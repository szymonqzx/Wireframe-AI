//! Cache Module — Example Wireframe AI module.
//!
//! Provides a simple key-value cache via NATS messages.
//! Subscribes to `cache.get`, `cache.set`, `cache.delete`, `cache.clear`.
//! Publishes responses to `cache.response`.
//!
//! ## Use case
//!
//! Shared cache for agent context, session state, or intermediate results.

use agentic_sdk::Module;
use std::collections::HashMap;

const MAX_CACHE_ENTRIES: usize = 10_000;
const MAX_KEY_LENGTH: usize = 256;

#[derive(Default)]
struct CacheModule {
    store: HashMap<String, (serde_json::Value, i64)>, // value + expiry timestamp
}

impl CacheModule {
    fn enforce_bounds(&mut self) {
        // Evict expired entries first
        let now = chrono::Utc::now().timestamp();
        self.store
            .retain(|_, (_, expiry)| *expiry == 0 || *expiry > now);

        // Evict arbitrary entries if still over limit
        while self.store.len() > MAX_CACHE_ENTRIES {
            if let Some(k) = self.store.keys().next().cloned() {
                self.store.remove(&k);
            } else {
                break;
            }
        }
    }
}

#[agentic_sdk::module(
    subscribes = ["cache.get", "cache.set", "cache.delete", "cache.clear"],
    publishes  = ["cache.response"],
    queue_group = "cache_handler"
)]
impl Module for CacheModule {
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {
        let now = chrono::Utc::now().timestamp();

        // Clean expired entries
        self.store
            .retain(|_, (_, expiry)| *expiry > now || *expiry == 0);

        let response = match env.topic.as_str() {
            "cache.get" => {
                let key = env
                    .payload
                    .get("key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let value = self.store.get(key).map(|(v, _)| v.clone());
                serde_json::json!({
                    "operation": "get",
                    "key": key,
                    "found": value.is_some(),
                    "value": value,
                    "timestamp": now,
                })
            }
            "cache.set" => {
                let key = env
                    .payload
                    .get("key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let value = env.payload.get("value").cloned().unwrap_or_default();
                let ttl_secs = env
                    .payload
                    .get("ttl_secs")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let expiry = if ttl_secs > 0 { now + ttl_secs } else { 0 };

                if key.is_empty() {
                    serde_json::json!({
                        "operation": "set",
                        "key": key,
                        "status": "error",
                        "error": "empty key",
                        "timestamp": now,
                    })
                } else if key.len() > MAX_KEY_LENGTH {
                    serde_json::json!({
                        "operation": "set",
                        "key": key,
                        "status": "error",
                        "error": "key too long",
                        "timestamp": now,
                    })
                } else {
                    self.store.insert(key.clone(), (value, expiry));
                    self.enforce_bounds();
                    tracing::info!(key, "cache entry set");
                    serde_json::json!({
                        "operation": "set",
                        "key": key,
                        "status": "ok",
                        "timestamp": now,
                    })
                }
            }
            "cache.delete" => {
                let key = env
                    .payload
                    .get("key")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let removed = self.store.remove(key).is_some();
                serde_json::json!({
                    "operation": "delete",
                    "key": key,
                    "removed": removed,
                    "timestamp": now,
                })
            }
            "cache.clear" => {
                let count = self.store.len();
                self.store.clear();
                serde_json::json!({
                    "operation": "clear",
                    "cleared_entries": count,
                    "timestamp": now,
                })
            }
            _ => serde_json::json!({ "error": "unknown operation" }),
        };

        vec![env.reply("cache.response", response)]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    CacheModule::default().run("nats://localhost:4222").await
}
