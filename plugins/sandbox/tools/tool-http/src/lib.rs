//! HTTP tool — makes HTTP requests from sandbox.

use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::sandbox::SandboxContext;
use agentic_sdk::plugins::sandbox::Tool;
use async_trait::async_trait;
use serde_json::Value;

/// HTTP tool for making requests.
pub struct HttpTool {
    timeout_seconds: u64,
}

impl HttpTool {
    pub fn new() -> Self {
        Self {
            timeout_seconds: 30,
        }
    }

    pub fn with_timeout(timeout: u64) -> Self {
        Self {
            timeout_seconds: timeout,
        }
    }
}

impl Default for HttpTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for HttpTool {
    fn plugin_id(&self) -> &'static str {
        "tool-http"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "HTTP tool for sandbox"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        if let Some(timeout) = config.get("timeout_seconds").and_then(|v| v.as_u64()) {
            self.timeout_seconds = timeout;
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
impl Tool for HttpTool {
    fn tool_name(&self) -> &'static str {
        "http"
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {"type": "string"},
                "method": {"type": "string", "enum": ["GET", "POST", "PUT", "DELETE"]}
            },
            "required": ["url"]
        })
    }

    async fn execute(
        &self,
        params: Value,
        _sandbox_context: &SandboxContext,
    ) -> Result<Value, agentic_sdk::plugins::sandbox::ToolError> {
        // Placeholder: simulate HTTP request
        // TODO: Implement actual HTTP requests with reqwest
        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        Ok(serde_json::json!({
            "status": "simulated",
            "url": url
        }))
    }
}
