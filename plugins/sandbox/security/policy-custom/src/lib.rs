//! Custom security policy — configurable security rules.

use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::sandbox::FileOperation;
use agentic_sdk::plugins::sandbox::SecurityPolicy;
use async_trait::async_trait;
use serde_json::Value;

/// Custom security policy with configurable rules.
pub struct CustomPolicy {
    allowed_domains: Vec<String>,
    block_network: bool,
}

impl CustomPolicy {
    pub fn new() -> Self {
        Self {
            allowed_domains: vec![],
            block_network: false,
        }
    }

    pub fn with_allowed_domains(domains: Vec<String>) -> Self {
        Self {
            allowed_domains: domains,
            block_network: false,
        }
    }

    pub fn with_network_blocked(blocked: bool) -> Self {
        Self {
            allowed_domains: vec![],
            block_network: blocked,
        }
    }
}

impl Default for CustomPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for CustomPolicy {
    fn plugin_id(&self) -> &'static str {
        "policy-custom"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Custom security policy for sandbox"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        if let Some(domains) = config.get("allowed_domains").and_then(|v| v.as_array()) {
            self.allowed_domains = domains
                .iter()
                .filter_map(|d| d.as_str().map(|s| s.to_string()))
                .collect();
        }
        if let Some(blocked) = config.get("block_network").and_then(|v| v.as_bool()) {
            self.block_network = blocked;
        }
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, agentic_sdk::plugin::PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }
}

#[async_trait]
impl SecurityPolicy for CustomPolicy {
    async fn validate_command(
        &self,
        command: &str,
        _working_dir: &str,
    ) -> Result<bool, agentic_sdk::plugins::sandbox::SecurityError> {
        // Block dangerous commands
        if command.contains("rm -rf") || command.contains("sudo") {
            return Ok(false);
        }
        Ok(true)
    }

    async fn validate_file_access(
        &self,
        _path: &str,
        _operation: FileOperation,
    ) -> Result<bool, agentic_sdk::plugins::sandbox::SecurityError> {
        // Allow all file access for now
        Ok(true)
    }

    async fn validate_network_access(
        &self,
        url: &str,
    ) -> Result<bool, agentic_sdk::plugins::sandbox::SecurityError> {
        // Check if network operations are blocked
        if self.block_network {
            return Ok(false);
        }

        // Check domain whitelist if configured
        if !self.allowed_domains.is_empty() {
            for domain in &self.allowed_domains {
                if url.contains(domain) {
                    return Ok(true);
                }
            }
            return Ok(false);
        }

        Ok(true)
    }
}
