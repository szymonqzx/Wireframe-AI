//! HTTP tool — makes HTTP requests from sandbox.

use agentic_sdk::plugin::{Plugin, PluginError};
use agentic_sdk::plugins::sandbox::{SandboxContext, Tool, ToolError};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::{json, Value};
use std::str::FromStr;
use std::time::Duration;

/// HTTP tool for making requests.
pub struct HttpTool {
    timeout_seconds: u64,
    client: reqwest::Client,
}

impl HttpTool {
    pub fn new() -> Self {
        Self::with_timeout(30)
    }

    pub fn with_timeout(timeout: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout))
            .build()
            .expect("Failed to create HTTP client");
        Self {
            timeout_seconds: timeout,
            client,
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

    async fn initialize(&mut self, config: &Value) -> Result<(), PluginError> {
        if let Some(timeout) = config.get("timeout_seconds").and_then(|v| v.as_u64()) {
            if timeout != self.timeout_seconds {
                self.timeout_seconds = timeout;
                self.client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(timeout))
                    .build()
                    .map_err(|e| {
                        PluginError::InitializationFailed(format!(
                            "Failed to create HTTP client: {}",
                            e
                        ))
                    })?;
            }
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
                "url": {"type": "string", "description": "The URL to send the request to"},
                "method": {
                    "type": "string",
                    "enum": ["GET", "POST", "PUT", "DELETE"],
                    "default": "GET",
                    "description": "The HTTP method to use"
                },
                "headers": {
                    "type": "object",
                    "additionalProperties": {"type": "string"},
                    "description": "HTTP headers to include in the request"
                },
                "body": {
                    "type": "string",
                    "description": "The request body to send"
                }
            },
            "required": ["url"]
        })
    }

    async fn execute(
        &self,
        params: Value,
        _sandbox_context: &SandboxContext,
    ) -> Result<Value, ToolError> {
        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing URL".to_string()))?;

        let method_str = params
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");

        let method = reqwest::Method::from_str(method_str).map_err(|_| {
            ToolError::InvalidParameters(format!("Invalid HTTP method: {}", method_str))
        })?;

        let mut request_builder = self.client.request(method, url);

        if let Some(headers_value) = params.get("headers").and_then(|v| v.as_object()) {
            let mut headers = HeaderMap::new();
            for (k, v) in headers_value {
                if let Some(v_str) = v.as_str() {
                    let header_name = HeaderName::from_str(k).map_err(|_| {
                        ToolError::InvalidParameters(format!("Invalid header name: {}", k))
                    })?;
                    let header_value = HeaderValue::from_str(v_str).map_err(|_| {
                        ToolError::InvalidParameters(format!("Invalid header value for {}: {}", k, v_str))
                    })?;
                    headers.insert(header_name, header_value);
                }
            }
            request_builder = request_builder.headers(headers);
        }

        if let Some(body) = params.get("body").and_then(|v| v.as_str()) {
            request_builder = request_builder.body(body.to_string());
        }

        let response = request_builder.send().await.map_err(|e| {
            ToolError::ExecutionFailed(format!("HTTP request failed: {}", e))
        })?;

        let status = response.status().as_u16();
        let mut response_headers = serde_json::Map::new();
        for (name, value) in response.headers().iter() {
            if let Ok(v) = value.to_str() {
                response_headers.insert(name.to_string(), json!(v));
            }
        }

        let body = response.text().await.map_err(|e| {
            ToolError::ExecutionFailed(format!("Failed to read response body: {}", e))
        })?;

        Ok(json!({
            "status": status,
            "headers": response_headers,
            "body": body
        }))
    }
}
