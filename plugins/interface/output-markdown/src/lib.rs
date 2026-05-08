//! Markdown output formatter — formats results as markdown.

use agentic_sdk::message_types::TaskComplete;
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::interface::OutputFormatter;
use async_trait::async_trait;
use serde_json::Value;

/// Markdown output formatter.
pub struct MarkdownOutput {
    syntax_highlighting: bool,
}

impl MarkdownOutput {
    pub fn new() -> Self {
        Self {
            syntax_highlighting: false,
        }
    }

    pub fn with_syntax_highlighting(highlighting: bool) -> Self {
        Self {
            syntax_highlighting: highlighting,
        }
    }
}

impl Default for MarkdownOutput {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for MarkdownOutput {
    fn plugin_id(&self) -> &'static str {
        "output-markdown"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Markdown output formatter for interface"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        if let Some(highlighting) = config.get("syntax_highlighting").and_then(|v| v.as_bool()) {
            self.syntax_highlighting = highlighting;
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
impl OutputFormatter for MarkdownOutput {
    async fn format_result(
        &self,
        result: &TaskComplete,
    ) -> Result<String, agentic_sdk::plugins::interface::FormatError> {
        // For Markdown output, we preserve the result as-is
        // Syntax highlighting for terminal output would require ANSI codes
        // which is outside the scope of Markdown formatting
        // The syntax_highlighting flag is preserved for future use
        Ok(result.result.clone())
    }
}
