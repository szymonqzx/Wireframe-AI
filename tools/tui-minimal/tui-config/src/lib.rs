//! TUI configuration management
//! 
//! Loads and manages TUI configuration from TOML files

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// TUI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiConfig {
    /// NATS server URL
    pub nats_url: String,
    
    /// Tick rate in milliseconds
    pub tick_rate_ms: u64,
    
    /// Provider configurations
    pub providers: Vec<ProviderConfig>,
    
    /// Current provider name
    pub current_provider: String,
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider name
    pub name: String,
    
    /// API key environment variable
    pub api_key_env: String,
    
    /// Model name
    pub model: String,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            nats_url: "nats://localhost:4222".to_string(),
            tick_rate_ms: 250,
            providers: vec![],
            current_provider: String::new(),
        }
    }
}

impl TuiConfig {
    /// Load configuration from file
    pub fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: TuiConfig = toml::from_str(&content)?;
        Ok(config)
    }
    
    /// Load configuration from default path (tui-config.toml in current directory)
    pub fn load_default() -> Result<Self> {
        let path = PathBuf::from("tui-config.toml");
        if path.exists() {
            Self::load_from_file(&path)
        } else {
            Ok(Self::default())
        }
    }
    
    /// Save configuration to file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = TuiConfig::default();
        assert_eq!(config.nats_url, "nats://localhost:4222");
        assert_eq!(config.tick_rate_ms, 250);
    }
    
    #[test]
    fn test_config_serialization() {
        let config = TuiConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: TuiConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.nats_url, config.nats_url);
    }
}
