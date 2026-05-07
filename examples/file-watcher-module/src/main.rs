//! File Watcher Module — Example Wireframe AI module.
//!
//! Subscribes to `file.watch.request` and publishes `file.changed` events
//! when watched files are modified. Demonstrates stateful modules with
//! background tasks.
//!
//! ## Use case
//!
//! Trigger agent workflows when source files change.

use agentic_sdk::{Envelope, Module};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

struct FileWatcherModule {
    watched_files: Arc<Mutex<HashSet<PathBuf>>>,
    file_states: Arc<Mutex<std::collections::HashMap<PathBuf, u64>>>,
}

impl FileWatcherModule {
    fn new() -> Self {
        Self {
            watched_files: Arc::new(Mutex::new(HashSet::new())),
            file_states: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    fn start_watcher(&self, client: async_nats::Client) {
        let watched = self.watched_files.clone();
        let states = self.file_states.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(5));
            loop {
                ticker.tick().await;
                let files = watched.lock().await.clone();
                for path in files {
                    match tokio::fs::metadata(&path).await {
                        Ok(meta) => {
                            let modified = meta
                                .modified()
                                .ok()
                                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                .map(|d| d.as_secs());

                            if let Some(new_mtime) = modified {
                                let mut states_guard = states.lock().await;
                                if let Some(old_mtime) = states_guard.get(&path) {
                                    if *old_mtime != new_mtime {
                                        tracing::info!(path = %path.display(), "file changed");
                                        let event = serde_json::json!({
                                            "path": path.to_string_lossy(),
                                            "event": "modified",
                                            "mtime": new_mtime,
                                            "timestamp": chrono::Utc::now().timestamp(),
                                        });
                                        let env = Envelope::new("file.changed", event, None);
                                        if let Ok(data) = serde_json::to_vec(&env) {
                                            let _ =
                                                client.publish("file.changed", data.into()).await;
                                        }
                                    }
                                }
                                states_guard.insert(path, new_mtime);
                            }
                        }
                        Err(e) => {
                            tracing::warn!(path = %path.display(), error = %e, "failed to check file");
                        }
                    }
                }
            }
        });
    }
}

#[agentic_sdk::module(
    subscribes = ["file.watch.request", "file.unwatch.request"],
    publishes  = ["file.changed", "file.watch.confirmed"],
    queue_group = "file_watcher"
)]
impl Module for FileWatcherModule {
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {
        match env.topic.as_str() {
            "file.watch.request" => {
                let path_str = env
                    .payload
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let path = PathBuf::from(path_str);

                if !path_str.is_empty() {
                    self.watched_files.lock().await.insert(path.clone());
                    tracing::info!(path = %path.display(), "watching file");

                    let confirmed = serde_json::json!({
                        "path": path_str,
                        "status": "watching",
                        "timestamp": chrono::Utc::now().timestamp(),
                    });
                    return vec![env.reply("file.watch.confirmed", confirmed)];
                }
            }
            "file.unwatch.request" => {
                let path_str = env
                    .payload
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let path = PathBuf::from(path_str);
                self.watched_files.lock().await.remove(&path);
                tracing::info!(path = %path.display(), "unwatched file");
            }
            _ => {}
        }

        vec![]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let module = FileWatcherModule::new();

    let client = async_nats::connect("nats://localhost:4222").await?;
    agentic_sdk::announce_online(
        &client,
        "file-watcher-module",
        "0.1.0",
        &["file.watch.request", "file.unwatch.request"],
        &["file.changed", "file.watch.confirmed"],
    )
    .await?;

    module.start_watcher(client.clone());

    // Manual run loop for this module
    let watched = module.watched_files.clone();

    let mut sub = client
        .queue_subscribe("file.watch.request", "file_watcher".to_string())
        .await?;
    let mut sub2 = client
        .queue_subscribe("file.unwatch.request", "file_watcher".to_string())
        .await?;

    use futures::StreamExt;

    loop {
        tokio::select! {
            msg = sub.next() => {
                match msg {
                    Some(msg) => {
                        if let Ok(env) = serde_json::from_slice::<Envelope<serde_json::Value>>(&msg.payload) {
                            let path_str = env.payload.get("path").and_then(|v| v.as_str()).unwrap_or("");
                            let path = PathBuf::from(path_str);
                            if is_safe_path(&path) {
                                watched.lock().await.insert(path);
                                let confirmed = serde_json::json!({
                                    "path": path_str,
                                    "status": "watching",
                                    "timestamp": chrono::Utc::now().timestamp(),
                                });
                                let reply = env.reply("file.watch.confirmed", confirmed);
                                if let Ok(data) = serde_json::to_vec(&reply) {
                                    let _ = client.publish(reply.topic, data.into()).await;
                                }
                            } else {
                                tracing::warn!(path = %path_str, "rejected unsafe watch path");
                            }
                        }
                    }
                    None => break,
                }
            }
            msg = sub2.next() => {
                match msg {
                    Some(msg) => {
                        if let Ok(env) = serde_json::from_slice::<Envelope<serde_json::Value>>(&msg.payload) {
                            let path_str = env.payload.get("path").and_then(|v| v.as_str()).unwrap_or("");
                            let path = PathBuf::from(path_str);
                            if is_safe_path(&path) {
                                watched.lock().await.remove(&path);
                            }
                        }
                    }
                    None => break,
                }
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("shutting down file-watcher-module");
                break;
            }
        }
    }

    agentic_sdk::announce_offline(&client, "file-watcher-module", "0.1.0")
        .await
        .ok();
    Ok(())
}

/// Validate that a watch path is safe to access.
/// Rejects paths that attempt directory traversal or target sensitive files.
fn is_safe_path(path: &std::path::Path) -> bool {
    use std::path::Component;

    if path.as_os_str().is_empty() {
        return false;
    }

    // Check for suspicious file extensions/patterns
    let path_str = path.to_string_lossy();
    let lower = path_str.to_lowercase();
    let sensitive_patterns = [
        ".env",
        ".ssh",
        ".aws",
        ".gnupg",
        ".docker",
        "id_rsa",
        "id_dsa",
        "id_ecdsa",
        "id_ed25519",
        ".pgpass",
        ".netrc",
        "credentials",
        ".gitconfig",
        "cookie",
        "token",
        "secret",
        "password",
        "key",
    ];
    for pat in &sensitive_patterns {
        if lower.contains(pat) {
            return false;
        }
    }

    // Check for directory traversal attempts
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => {
                // Absolute paths are rejected for security
                return false;
            }
            Component::ParentDir => {
                // Directory traversal is rejected
                return false;
            }
            Component::Normal(_) => {}
            Component::CurDir => {}
        }
    }

    true
}
