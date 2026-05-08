//! Wireframe-AI Configuration Management
//!
//! Centralized configuration for all Wireframe-AI modules.
//! Supports environment variables, TOML files, hot reloading, and sensible defaults.

pub mod retry;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, event::EventKind};
use notify::event::Event as NotifyEvent;

/// Main configuration structure for Wireframe-AI modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireframeConfig {
    pub nats: NatsConfig,
    pub orchestrator: OrchestratorConfig,
    pub context: ContextConfig,
    pub interface: InterfaceConfig,
}

/// NATS connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsConfig {
    /// NATS server URL (default: nats://localhost:4222)
    pub url: String,
    /// Connection timeout in seconds (default: 10)
    pub connection_timeout_secs: u64,
    /// Maximum reconnect attempts (default: 5)
    pub max_reconnect_attempts: u32,
}

/// Orchestrator module configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    /// Number of concurrent agents to spawn per task (default: 3)
    pub concurrency: u32,
    /// Timeout for collecting agent results in seconds (default: 600)
    pub result_timeout_secs: u64,
}

/// Context module configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// Database path (default: ./wireframe_ai_context.db)
    pub db_path: String,
    /// Maximum session history messages to load (default: 50)
    pub max_session_history: usize,
    /// Maximum memory chunks to retrieve (default: 20)
    pub max_memory_chunks: usize,
    /// Maximum context tokens (default: 32768)
    pub max_context_tokens: usize,
}

/// Interface module configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceConfig {
    /// Default timeout for task completion in seconds (default: 300)
    pub default_timeout_secs: u64,
    /// Whether to show welcome banner (default: true)
    pub show_banner: bool,
}

/// Configuration manager with hot reload support
pub struct ConfigManager {
    config: Arc<RwLock<WireframeConfig>>,
    config_path: Option<String>,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(WireframeConfig::default())),
            config_path: None,
        }
    }

    /// Load configuration from environment variables
    pub async fn load_from_env(&self) -> Result<()> {
        let config = WireframeConfig::from_env()?;
        *self.config.write().await = config;
        Ok(())
    }

    /// Load configuration from a TOML file
    pub async fn load_from_file(&mut self, path: &str) -> Result<()> {
        let config = WireframeConfig::from_file(path)?;
        self.config_path = Some(path.to_string());
        *self.config.write().await = config;
        Ok(())
    }

    /// Get current configuration
    pub async fn get(&self) -> WireframeConfig {
        self.config.read().await.clone()
    }

    /// Enable hot reloading for configuration file
    pub async fn enable_hot_reload(&mut self) -> Result<()> {
        let config_path = self.config_path.clone().ok_or_else(|| {
            anyhow::anyhow!("No config file loaded for hot reload")
        })?;

        let config_path_clone = config_path.clone();
        let config_path_for_spawn = config_path.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut watcher = RecommendedWatcher::new(
                move |res: Result<NotifyEvent, _>| {
                    if let Ok(event) = res {
                        if matches!(event.kind, EventKind::Modify(_)) {
                            tracing::info!("Configuration file modified, reloading...");
                            if let Ok(new_config) = WireframeConfig::from_file(&config_path_clone) {
                                let mut guard = config.blocking_write();
                                *guard = new_config;
                                tracing::info!("Configuration reloaded successfully");
                            }
                        }
                    }
                },
                notify::Config::default(),
            ).expect("Failed to create file watcher");

            watcher.watch(Path::new(&config_path_for_spawn), RecursiveMode::NonRecursive)
                .expect("Failed to watch config file");

            // Keep the watcher alive
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });

        tracing::info!("Hot reload enabled for config file: {}", config_path);
        Ok(())
    }
}

impl Default for WireframeConfig {
    fn default() -> Self {
        Self {
            nats: NatsConfig {
                url: env::var("WIREFRAME_AI_NATS_URL")
                    .unwrap_or_else(|_| "nats://localhost:4222".to_string()),
                connection_timeout_secs: 10,
                max_reconnect_attempts: 5,
            },
            orchestrator: OrchestratorConfig {
                concurrency: env::var("WIREFRAME_AI_CONCURRENCY")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(3),
                result_timeout_secs: 600,
            },
            context: ContextConfig {
                db_path: env::var("WIREFRAME_AI_CONTEXT_DB")
                    .unwrap_or_else(|_| "./wireframe_ai_context.db".to_string()),
                max_session_history: 50,
                max_memory_chunks: 20,
                max_context_tokens: 32768,
            },
            interface: InterfaceConfig {
                default_timeout_secs: 300,
                show_banner: true,
            },
        }
    }
}

impl WireframeConfig {
    /// Load configuration from environment variables with defaults
    pub fn from_env() -> Result<Self> {
        Ok(Self::default())
    }

    /// Load configuration from a TOML file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path))?;
        let mut config: WireframeConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path))?;

        // Override with environment variables if set
        if let Ok(url) = env::var("WIREFRAME_AI_NATS_URL") {
            config.nats.url = url;
        }
        if let Ok(concurrency) = env::var("WIREFRAME_AI_CONCURRENCY") {
            config.orchestrator.concurrency = concurrency
                .parse()
                .context("Invalid WIREFRAME_AI_CONCURRENCY: must be a positive integer")?;
        }
        if let Ok(db_path) = env::var("WIREFRAME_AI_CONTEXT_DB") {
            config.context.db_path = db_path;
        }

        Ok(config)
    }

    /// Get NATS URL
    pub fn nats_url(&self) -> &str {
        &self.nats.url
    }

    /// Get orchestrator concurrency
    pub fn concurrency(&self) -> u32 {
        self.orchestrator.concurrency
    }

    /// Get context database path
    pub fn context_db_path(&self) -> &str {
        &self.context.db_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    #[serial]
    fn test_default_config() {
        // Ensure no env var overrides affect the test
        env::remove_var("WIREFRAME_AI_NATS_URL");
        env::remove_var("WIREFRAME_AI_CONCURRENCY");
        env::remove_var("WIREFRAME_AI_CONTEXT_DB");

        let config = WireframeConfig::default();
        assert_eq!(config.nats.url, "nats://localhost:4222");
        assert_eq!(config.orchestrator.concurrency, 3);
        assert_eq!(config.context.db_path, "./wireframe_ai_context.db");
    }

    #[test]
    fn test_config_manager_new() {
        let manager = ConfigManager::new();
        // Since we can't easily access private fields in integration tests,
        // but this is a unit test (mod tests inside lib.rs), we can.
        assert!(manager.config_path.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn test_config_manager_get() {
        // Ensure no env var overrides affect the test
        env::remove_var("WIREFRAME_AI_NATS_URL");

        let manager = ConfigManager::new();
        let config = manager.get().await;
        assert_eq!(config.nats.url, "nats://localhost:4222");
    }

    #[test]
    #[serial]
    fn test_config_from_file() -> Result<()> {
        // Ensure no env var overrides affect the test
        env::remove_var("WIREFRAME_AI_NATS_URL");
        env::remove_var("WIREFRAME_AI_CONCURRENCY");
        env::remove_var("WIREFRAME_AI_CONTEXT_DB");

        let mut file = NamedTempFile::new()?;
        let toml_content = r#"
[nats]
url = "nats://test-host:4222"
connection_timeout_secs = 10
max_reconnect_attempts = 5

[orchestrator]
concurrency = 5
result_timeout_secs = 600

[context]
db_path = "./test.db"
max_session_history = 50
max_memory_chunks = 20
max_context_tokens = 32768

[interface]
default_timeout_secs = 300
show_banner = true
"#;
        writeln!(file, "{}", toml_content)?;
        let path = file.path().to_str().unwrap();

        let config = WireframeConfig::from_file(path)?;

        assert_eq!(config.nats.url, "nats://test-host:4222");
        assert_eq!(config.orchestrator.concurrency, 5);
        assert_eq!(config.context.db_path, "./test.db");

        Ok(())
    }
}
