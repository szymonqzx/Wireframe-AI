//! Configuration loading and parsing for plugin system.

use notify::RecommendedWatcher;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Top-level plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub modules: HashMap<String, ModuleConfig>,
}

/// Configuration for a single module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfig {
    pub enabled: bool,
    pub plugins: ModulePlugins,
}

/// Plugin configuration within a module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulePlugins {
    pub storage: Option<PluginSpec>,
    pub memory: Option<PluginSpec>,
    pub enrichment_pipeline: Vec<EnrichmentStep>,
    pub planner: Option<PluginSpec>,
    pub execution: Option<PluginSpec>,
    pub synthesizer: Option<PluginSpec>,
    pub tools: Vec<PluginSpec>,
    pub security: Option<PluginSpec>,
    pub resources: Option<PluginSpec>,
    pub input: Option<PluginSpec>,
    pub output: Option<PluginSpec>,
    pub model: Option<PluginSpec>,
    pub reasoning: Option<PluginSpec>,
    pub tool_selector: Option<PluginSpec>,
}

/// Specification for a single plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSpec {
    pub plugin_id: String,
    #[serde(default)]
    pub config: Value,
    #[serde(default)]
    pub order: usize,
}

/// A step in an enrichment pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichmentStep {
    pub plugin_id: String,
    #[serde(default)]
    pub order: usize,
    #[serde(default)]
    pub config: Value,
}

impl PluginConfig {
    /// Parse configuration from YAML string.
    pub fn from_yaml(yaml: &str) -> Result<Self, ConfigError> {
        serde_yaml::from_str(yaml).map_err(ConfigError::ParseError)
    }

    /// Parse configuration from JSON string.
    pub fn from_json(json: &str) -> Result<Self, ConfigError> {
        serde_json::from_str(json).map_err(ConfigError::JsonParseError)
    }

    /// Load configuration from a file.
    pub fn from_file(path: &PathBuf) -> Result<Self, ConfigError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ConfigError::IoError(e.to_string()))?;

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| ConfigError::IoError("No file extension".to_string()))?;

        match extension {
            "yaml" | "yml" => Self::from_yaml(&content),
            "json" => Self::from_json(&content),
            _ => Err(ConfigError::UnsupportedFormat(extension.to_string())),
        }
    }

    /// Save configuration to a file.
    pub fn to_file(&self, path: &PathBuf) -> Result<(), ConfigError> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .ok_or_else(|| ConfigError::IoError("No file extension".to_string()))?;

        let content = match extension {
            "yaml" | "yml" => serde_yaml::to_string(self).map_err(ConfigError::ParseError)?,
            "json" => serde_json::to_string_pretty(self).map_err(ConfigError::JsonParseError)?,
            _ => return Err(ConfigError::UnsupportedFormat(extension.to_string())),
        };

        std::fs::write(path, content).map_err(|e| ConfigError::IoError(e.to_string()))?;

        Ok(())
    }
}

#[allow(clippy::derivable_impls)]
impl Default for ModulePlugins {
    fn default() -> Self {
        Self {
            storage: None,
            memory: None,
            enrichment_pipeline: Vec::new(),
            planner: None,
            execution: None,
            synthesizer: None,
            tools: Vec::new(),
            security: None,
            resources: None,
            input: None,
            output: None,
            model: None,
            reasoning: None,
            tool_selector: None,
        }
    }
}

/// Configuration error.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("YAML parse error: {0}")]
    ParseError(serde_yaml::Error),

    #[error("JSON parse error: {0}")]
    JsonParseError(serde_json::Error),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Environment variable error: {0}")]
    EnvVarError(String),

    #[error("Watcher error: {0}")]
    WatcherError(String),
}

impl PluginConfig {
    /// Expand environment variables in configuration values.
    ///
    /// Supports ${VAR} syntax for environment variable substitution.
    /// Missing variables are replaced with empty string.
    pub fn expand_env_vars(&mut self) -> Result<(), ConfigError> {
        let mut value =
            serde_json::to_value(&*self).map_err(|e| ConfigError::IoError(e.to_string()))?;
        Self::expand_env_vars_in_value(&mut value);
        *self = serde_json::from_value(value).map_err(|e| ConfigError::IoError(e.to_string()))?;
        Ok(())
    }

    fn expand_env_vars_in_value(value: &mut Value) {
        match value {
            Value::String(s) => {
                *s = Self::expand_env_vars_in_string(s);
            }
            Value::Object(map) => {
                for (_key, val) in map.iter_mut() {
                    Self::expand_env_vars_in_value(val);
                }
            }
            Value::Array(arr) => {
                for val in arr.iter_mut() {
                    Self::expand_env_vars_in_value(val);
                }
            }
            _ => {}
        }
    }

    fn expand_env_vars_in_string(s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '$' && chars.peek() == Some(&'{') {
                chars.next(); // consume '{'
                let mut var_name = String::new();

                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                    var_name.push(c);
                }

                if let Ok(var_value) = std::env::var(&var_name) {
                    result.push_str(&var_value);
                }
                // If env var is not found, replace with empty string
            } else {
                result.push(c);
            }
        }

        result
    }

    /// Validate configuration against a JSON schema.
    ///
    /// Requires the "schema-validation" feature to be enabled.
    #[cfg(feature = "schema-validation")]
    pub fn validate(&self, schema: &Value) -> Result<(), ConfigError> {
        use jsonschema::JSONSchema;

        let compiled_schema = JSONSchema::compile(schema).map_err(|e| {
            ConfigError::ValidationError(format!("Schema compilation failed: {}", e))
        })?;

        let config_value =
            serde_json::to_value(self).map_err(|e| ConfigError::IoError(e.to_string()))?;

        if let Some(errors) = compiled_schema.validate(&config_value).err() {
            let error_messages: Vec<String> = errors.map(|e| e.to_string()).collect();
            return Err(ConfigError::ValidationError(error_messages.join(", ")));
        }

        Ok(())
    }

    /// Validate configuration (no-op without schema-validation feature).
    #[cfg(not(feature = "schema-validation"))]
    pub fn validate(&self, _schema: &Value) -> Result<(), ConfigError> {
        // No-op when feature is not enabled
        Ok(())
    }
}

/// Hot-reload configuration watcher.
///
/// Watches a configuration file for changes and triggers callbacks.
pub struct ConfigWatcher {
    path: PathBuf,
    config: Arc<RwLock<PluginConfig>>,
    watcher: Option<RecommendedWatcher>,
}

impl ConfigWatcher {
    /// Create a new config watcher for the given path.
    pub fn new(path: PathBuf) -> Result<Self, ConfigError> {
        let config = PluginConfig::from_file(&path)?;
        Ok(Self {
            path,
            config: Arc::new(RwLock::new(config)),
            watcher: None,
        })
    }

    /// Get the current configuration.
    pub async fn get_config(&self) -> PluginConfig {
        self.config.read().await.clone()
    }

    /// Start watching for configuration changes.
    ///
    /// The callback is invoked when the configuration file changes.
    pub async fn watch<F>(&mut self, callback: F) -> Result<(), ConfigError>
    where
        F: Fn(PluginConfig) + Send + 'static,
    {
        use notify::event::EventKind;
        use notify::{RecursiveMode, Watcher};

        let path = self.path.clone();
        let config = self.config.clone();

        let mut watcher =
            notify::recommended_watcher(move |res: Result<notify::Event, _>| match res {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        if let Ok(new_config) = PluginConfig::from_file(&path) {
                            let rt = tokio::runtime::Handle::current();
                            rt.block_on(async {
                                *config.write().await = new_config.clone();
                                callback(new_config);
                            });
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Config watch error: {}", e);
                }
            })
            .map_err(|e| ConfigError::WatcherError(e.to_string()))?;

        watcher
            .watch(&self.path, RecursiveMode::NonRecursive)
            .map_err(|e| ConfigError::WatcherError(e.to_string()))?;

        self.watcher = Some(watcher);
        Ok(())
    }

    /// Stop watching for configuration changes.
    pub fn stop(&mut self) {
        self.watcher = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_config_watcher_new_valid_yaml() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.yaml");
        let yaml = r#"
modules:
  test:
    enabled: true
    plugins:
      enrichment_pipeline: []
      tools: []
"#;
        fs::write(&file_path, yaml).unwrap();

        let watcher = ConfigWatcher::new(file_path).unwrap();
        let config = watcher.get_config().await;
        assert!(config.modules.contains_key("test"));
    }

    #[tokio::test]
    async fn test_config_watcher_new_valid_json() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.json");
        let json = r#"{
            "modules": {
                "test": {
                    "enabled": true,
                    "plugins": {
                        "enrichment_pipeline": [],
                        "tools": []
                    }
                }
            }
        }"#;
        fs::write(&file_path, json).unwrap();

        let watcher = ConfigWatcher::new(file_path).unwrap();
        let config = watcher.get_config().await;
        assert!(config.modules.contains_key("test"));
    }

    #[test]
    fn test_config_watcher_new_non_existent_file() {
        let file_path = PathBuf::from("non_existent_file.yaml");
        let result = ConfigWatcher::new(file_path);
        assert!(result.is_err());
        match result.err().unwrap() {
            ConfigError::IoError(e) => assert!(e.contains("No such file or directory")),
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_config_watcher_new_unsupported_format() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.txt");
        fs::write(&file_path, "some content").unwrap();

        let result = ConfigWatcher::new(file_path);
        assert!(result.is_err());
        match result.err().unwrap() {
            ConfigError::UnsupportedFormat(ext) => assert_eq!(ext, "txt"),
            _ => panic!("Expected UnsupportedFormat"),
        }
    }

    #[test]
    fn test_config_watcher_new_invalid_content() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.yaml");
        fs::write(&file_path, "not a valid yaml: { [").unwrap();

        let result = ConfigWatcher::new(file_path);
        assert!(result.is_err());
        match result.err().unwrap() {
            ConfigError::ParseError(_) => (),
            _ => panic!("Expected ParseError"),
        }
    }

<<<<<<< HEAD
    #[test]
    fn test_plugin_config_from_file_valid_yaml() {
=======
    #[tokio::test]
    async fn test_config_watcher_stop() {
        use std::fs;
        use tempfile::tempdir;
>>>>>>> origin/jules-configwatcher-stop-test-4029460018877860288
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.yaml");
        let yaml = r#"
modules:
  test:
    enabled: true
    plugins:
      enrichment_pipeline: []
      tools: []
"#;
        fs::write(&file_path, yaml).unwrap();

<<<<<<< HEAD
        let config = PluginConfig::from_file(&file_path).unwrap();
        assert!(config.modules.contains_key("test"));
    }

    #[test]
    fn test_plugin_config_from_file_valid_json() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.json");
        let json = r#"{
            "modules": {
                "test": {
                    "enabled": true,
                    "plugins": {
                        "enrichment_pipeline": [],
                        "tools": []
                    }
                }
            }
        }"#;
        fs::write(&file_path, json).unwrap();

        let config = PluginConfig::from_file(&file_path).unwrap();
        assert!(config.modules.contains_key("test"));
    }

    #[test]
    fn test_plugin_config_from_file_non_existent() {
        let file_path = PathBuf::from("non_existent_file.yaml");
        let result = PluginConfig::from_file(&file_path);
        assert!(result.is_err());
        match result.err().unwrap() {
            ConfigError::IoError(e) => assert!(e.contains("No such file or directory")),
            _ => panic!("Expected IoError"),
        }
    }

    #[test]
    fn test_plugin_config_from_file_unsupported_format() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.txt");
        fs::write(&file_path, "some content").unwrap();

        let result = PluginConfig::from_file(&file_path);
        assert!(result.is_err());
        match result.err().unwrap() {
            ConfigError::UnsupportedFormat(ext) => assert_eq!(ext, "txt"),
            _ => panic!("Expected UnsupportedFormat"),
        }
    }

    #[test]
    fn test_plugin_config_from_file_invalid_content() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config.yaml");
        fs::write(&file_path, "not a valid yaml: { [").unwrap();

        let result = PluginConfig::from_file(&file_path);
        assert!(result.is_err());
        match result.err().unwrap() {
            ConfigError::ParseError(_) => (),
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_plugin_config_to_file_yaml() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config_out.yaml");

        let config = PluginConfig {
            modules: {
                let mut map = HashMap::new();
                map.insert(
                    "test".to_string(),
                    ModuleConfig {
                        enabled: true,
                        plugins: ModulePlugins::default(),
                    },
                );
                map
            },
        };

        let result = config.to_file(&file_path);
        assert!(result.is_ok());

        // Verify we can read it back
        let read_config = PluginConfig::from_file(&file_path).unwrap();
        assert!(read_config.modules.contains_key("test"));
    }

    #[test]
    fn test_plugin_config_to_file_json() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config_out.json");

        let config = PluginConfig {
            modules: {
                let mut map = HashMap::new();
                map.insert(
                    "test".to_string(),
                    ModuleConfig {
                        enabled: true,
                        plugins: ModulePlugins::default(),
                    },
                );
                map
            },
        };

        let result = config.to_file(&file_path);
        assert!(result.is_ok());

        // Verify we can read it back
        let read_config = PluginConfig::from_file(&file_path).unwrap();
        assert!(read_config.modules.contains_key("test"));
    }

    #[test]
    fn test_plugin_config_to_file_unsupported_format() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("config_out.txt");

        let config = PluginConfig {
            modules: HashMap::new(),
        };

        let result = config.to_file(&file_path);
        assert!(result.is_err());
        match result.err().unwrap() {
            ConfigError::UnsupportedFormat(ext) => assert_eq!(ext, "txt"),
            _ => panic!("Expected UnsupportedFormat"),
        }

        #[tokio::test]
        async fn test_config_watcher_stop() {
            let dir = tempdir().unwrap();
            let file_path = dir.path().join("config.yaml");

            // Create a dummy config file
            std::fs::write(&file_path, "test: true").unwrap();

            let mut watcher = ConfigWatcher::new(file_path).unwrap();

            // Watcher is initially None until watch() is called
            assert!(watcher.watcher.is_none());

            // Start watching - we just pass a dummy callback
            let _ = watcher.watch(|_| {}).await.unwrap();

            // Watcher should be Some now
            assert!(watcher.watcher.is_some());

            // Stop watching
            watcher.stop();

            // Watcher should be None again
            assert!(watcher.watcher.is_none());
        }
    }
}
