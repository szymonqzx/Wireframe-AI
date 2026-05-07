//! Interface core — CLI orchestration and plugin lifecycle management.

use agentic_sdk::message_types::{TaskComplete, TaskSubmitted};
use agentic_sdk::plugins::interface::{InputError, InputMethod, FormatError, OutputFormatter};
use agentic_sdk::plugin::Plugin;
use agentic_sdk::PluginRegistry;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Default CLI input implementation.
struct DefaultCliInput;

#[async_trait::async_trait]
impl Plugin for DefaultCliInput {
    fn plugin_id(&self) -> &'static str {
        "default-cli-input"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn description(&self) -> &'static str {
        "Default CLI input method reading from stdin"
    }

    async fn initialize(&mut self, _config: &Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, agentic_sdk::plugin::PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl InputMethod for DefaultCliInput {
    async fn read_input(&self) -> Result<TaskSubmitted, InputError> {
        println!("Enter your request (or 'quit' to exit):");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)
            .map_err(|e| InputError::ReadFailed(e.to_string()))?;

        let input = input.trim();
        if input.is_empty() || input.eq_ignore_ascii_case("quit") {
            return Err(InputError::Interrupted);
        }

        Ok(TaskSubmitted {
            session_id: uuid::Uuid::new_v4().to_string(),
            user_input: input.to_string(),
            submitted_at: chrono::Utc::now().timestamp(),
        })
    }
}

/// Default CLI output implementation.
struct DefaultCliOutput;

#[async_trait::async_trait]
impl Plugin for DefaultCliOutput {
    fn plugin_id(&self) -> &'static str {
        "default-cli-output"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn description(&self) -> &'static str {
        "Default CLI output formatter"
    }

    async fn initialize(&mut self, _config: &Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, agentic_sdk::plugin::PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl OutputFormatter for DefaultCliOutput {
    async fn format_result(&self, result: &TaskComplete) -> Result<String, FormatError> {
        Ok(format!("Result: {}", result.result))
    }
}

/// Interface core manages plugin lifecycle and coordinates CLI I/O.
pub struct InterfaceCore {
    registry: PluginRegistry,
    input: Arc<RwLock<Option<Arc<dyn InputMethod>>>>,
    output: Arc<RwLock<Option<Arc<dyn OutputFormatter>>>>,
}

impl InterfaceCore {
    /// Create a new interface core with default plugins.
    pub fn new() -> Self {
        Self {
            registry: PluginRegistry::new(),
            input: Arc::new(RwLock::new(None)),
            output: Arc::new(RwLock::new(None)),
        }
    }
}

impl Default for InterfaceCore {
    fn default() -> Self {
        Self::new()
    }
}

impl InterfaceCore {
    /// Set the input plugin directly.
    pub async fn set_input(&self, input: Arc<dyn InputMethod>) {
        *self.input.write().await = Some(input);
    }

    /// Set the output plugin directly.
    pub async fn set_output(&self, output: Arc<dyn OutputFormatter>) {
        *self.output.write().await = Some(output);
    }

    /// Ensure default input plugin is set.
    pub async fn ensure_default_input(&self) {
        if self.input.read().await.is_none() {
            self.set_input(Arc::new(DefaultCliInput)).await;
        }
    }

    /// Ensure default output plugin is set.
    pub async fn ensure_default_output(&self) {
        if self.output.read().await.is_none() {
            self.set_output(Arc::new(DefaultCliOutput)).await;
        }
    }

    /// Read input using the configured input method.
    pub async fn read_input(&self) -> Result<TaskSubmitted, InputError> {
        // Ensure we have an input plugin
        self.ensure_default_input().await;

        let input = self.input.read().await;
        let input = input.as_ref().ok_or_else(|| InputError::ReadFailed("No input plugin configured".to_string()))?;
        input.read_input().await
    }

    /// Format result using the configured output formatter.
    pub async fn format_result(
        &self,
        result: &TaskComplete,
    ) -> Result<String, FormatError> {
        // Ensure we have an output plugin
        self.ensure_default_output().await;

        let output = self.output.read().await;
        let output = output.as_ref().ok_or_else(|| FormatError::FormattingFailed("No output plugin configured".to_string()))?;
        output.format_result(result).await
    }

    /// Get the plugin registry.
    pub fn registry(&self) -> &PluginRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agentic_sdk::message_types::TaskComplete;

    #[tokio::test]
    async fn test_interface_core_creation() {
        let core = InterfaceCore::new();
        assert!(core.input.read().await.is_none());
        assert!(core.output.read().await.is_none());
    }

    #[tokio::test]
    async fn test_interface_core_default_plugins() {
        let core = InterfaceCore::new();
        core.ensure_default_input().await;
        core.ensure_default_output().await;

        assert!(core.input.read().await.is_some());
        assert!(core.output.read().await.is_some());
    }

    #[tokio::test]
    async fn test_default_cli_input_plugin() {
        let input = Arc::new(DefaultCliInput);

        // Test plugin trait
        assert_eq!(input.plugin_id(), "default-cli-input");
        assert_eq!(input.version(), "0.1.0");
        assert_eq!(input.description(), "Default CLI input method reading from stdin");

        // Test initialization
        let mut input_clone = DefaultCliInput;
        let config = serde_json::json!({});
        assert!(input_clone.initialize(&config).await.is_ok());

        // Test health check
        assert!(input.health_check().await.is_ok());

        // Test shutdown
        assert!(input_clone.shutdown().await.is_ok());
    }

    #[tokio::test]
    async fn test_default_cli_output_plugin() {
        let output = Arc::new(DefaultCliOutput);

        // Test plugin trait
        assert_eq!(output.plugin_id(), "default-cli-output");
        assert_eq!(output.version(), "0.1.0");
        assert_eq!(output.description(), "Default CLI output formatter");

        // Test initialization
        let mut output_clone = DefaultCliOutput;
        let config = serde_json::json!({});
        assert!(output_clone.initialize(&config).await.is_ok());

        // Test health check
        assert!(output.health_check().await.is_ok());

        // Test shutdown
        assert!(output_clone.shutdown().await.is_ok());
    }

    #[tokio::test]
    async fn test_default_cli_output_format() {
        let output = Arc::new(DefaultCliOutput);
        let complete = TaskComplete {
            session_id: "test-session".to_string(),
            correlation_id: uuid::Uuid::new_v4().to_string(),
            result: "test result".to_string(),
            side_effects: vec![],
            warnings: vec![],
            completed_at: chrono::Utc::now().timestamp(),
        };

        let formatted = output.format_result(&complete).await.unwrap();
        assert!(formatted.contains("test result"));
    }

    #[tokio::test]
    async fn test_interface_core_set_input() {
        let core = InterfaceCore::new();
        let input = Arc::new(DefaultCliInput);
        core.set_input(input).await;

        assert!(core.input.read().await.is_some());
    }

    #[tokio::test]
    async fn test_interface_core_set_output() {
        let core = InterfaceCore::new();
        let output = Arc::new(DefaultCliOutput);
        core.set_output(output).await;

        assert!(core.output.read().await.is_some());
    }

    #[tokio::test]
    async fn test_interface_core_format_result() {
        let core = InterfaceCore::new();
        let complete = TaskComplete {
            session_id: "test-session".to_string(),
            correlation_id: uuid::Uuid::new_v4().to_string(),
            result: "test result".to_string(),
            side_effects: vec![],
            warnings: vec![],
            completed_at: chrono::Utc::now().timestamp(),
        };

        let formatted = core.format_result(&complete).await.unwrap();
        assert!(formatted.contains("test result"));
    }
}
