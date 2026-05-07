//! Template for a Sandbox Tool Plugin
//!
//! This template provides a starting point for implementing a custom
//! tool for the Wireframe-AI Sandbox module.
//!
//! To use this template:
//! 1. Copy this file to your plugin directory
//! 2. Replace "MyTool" with your tool name
//! 3. Implement the Tool trait methods
//! 4. Add your specific tool logic
//! 5. Register the plugin in your module

use agentic_sdk::plugins::sandbox::{Tool, ToolError, SandboxContext};
use agentic_sdk::plugin::{Plugin, PluginError};
use async_trait::async_trait;
use serde_json::{json, Value};

/// Your custom tool implementation
pub struct MyTool {
    // Add your tool-specific fields here
    // Example:
    // timeout: u64,
    // max_retries: usize,
    // api_endpoint: String,
}

impl MyTool {
    /// Create a new instance of your tool
    pub fn new(/* Add your constructor parameters */) -> Self {
        Self {
            // Initialize your fields
            // timeout: 30,
            // max_retries: 3,
            // api_endpoint: "https://api.example.com".to_string(),
        }
    }
}

#[async_trait]
impl Plugin for MyTool {
    fn plugin_id(&self) -> &'static str {
        "tool-my-custom" // Replace with your tool ID
    }

    fn version(&self) -> &'static str {
        "1.0.0" // Update with your version
    }

    fn description(&self) -> &'static str {
        "My custom tool for Wireframe-AI" // Update with your description
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), PluginError> {
        // Parse configuration and initialize your tool
        // Example:
        // if let Some(timeout) = config.get("timeout").and_then(|v| v.as_u64()) {
        //     self.timeout = timeout;
        // }
        //
        // if let Some(api_endpoint) = config.get("api_endpoint").and_then(|v| v.as_str()) {
        //     self.api_endpoint = api_endpoint.to_string();
        // }

        println!("MyTool initialized with config: {}", config);
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        // Check if your tool is healthy
        // Example:
        // let client = reqwest::Client::new();
        // let response = client
        //     .get(&format!("{}/health", self.api_endpoint))
        //     .send()
        //     .await;
        //
        // match response {
        //     Ok(resp) => Ok(resp.status().is_success()),
        //     Err(e) => Err(PluginError::HealthCheckFailed(e.to_string())),
        // }

        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        // Cleanup resources
        println!("MyTool shutdown complete");
        Ok(())
    }
}

#[async_trait]
impl Tool for MyTool {
    fn tool_name(&self) -> &'static str {
        "my_tool" // Replace with your tool name
    }

    fn input_schema(&self) -> Value {
        // Define the JSON schema for your tool's input parameters
        json!({
            "type": "object",
            "properties": {
                "param1": {
                    "type": "string",
                    "description": "First parameter description"
                },
                "param2": {
                    "type": "integer",
                    "description": "Second parameter description",
                    "default": 10
                },
                "param3": {
                    "type": "boolean",
                    "description": "Third parameter description",
                    "default": false
                }
            },
            "required": ["param1"],
            "additionalProperties": false
        })
    }

    async fn execute(
        &self,
        params: Value,
        sandbox_context: &SandboxContext,
    ) -> Result<Value, ToolError> {
        // Execute your tool with the given parameters
        println!("Executing MyTool with params: {}", params);
        println!("Sandbox context: {:?}", sandbox_context);

        // Validate parameters
        let param1 = params
            .get("param1")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing required param1".to_string()))?;

        let param2 = params
            .get("param2")
            .and_then(|v| v.as_i64())
            .unwrap_or(10);

        let param3 = params
            .get("param3")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Example 1: Simple computation
        let result = if param3 {
            format!("{} with param2={} and param3=true", param1, param2)
        } else {
            format!("{} with param2={}", param1, param2)
        };

        // Example 2: HTTP request
        // let client = reqwest::Client::new();
        // let response = client
        //     .post(&format!("{}/execute", self.api_endpoint))
        //     .json(&params)
        //     .timeout(Duration::from_secs(self.timeout))
        //     .send()
        //     .await
        //     .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
        //
        // let result = response
        //     .json::<Value>()
        //     .await
        //     .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        // Example 3: File operation (respecting sandbox context)
        // let file_path = sandbox_context.working_dir.join(param1);
        // let content = std::fs::read_to_string(&file_path)
        //     .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        // Example 4: Shell command (respecting sandbox context)
        // let output = std::process::Command::new(param1)
        //     .current_dir(&sandbox_context.working_dir)
        //     .output()
        //     .map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;

        println!("Tool execution result: {}", result);

        Ok(json!({
            "success": true,
            "result": result,
            "param1": param1,
            "param2": param2,
            "param3": param3
        }))
    }
}

// ============================================================================
// Example Configuration
// ============================================================================

/*
Add this to your module's configuration file:

plugins:
  tools:
    - plugin_id: "tool-my-custom"
      config:
        timeout: 30
        max_retries: 3
        api_endpoint: "https://api.example.com"
*/

// ============================================================================
// Example Usage in Module
// ============================================================================

/*
use agentic_sdk::PluginRegistry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = PluginRegistry::new();

    // Create and register your tool plugin
    let tool = Box::new(MyTool::new(/* params */));
    registry.register(tool).await?;

    // Retrieve and use the plugin
    let tool_plugin: Arc<MyTool> = registry.get("tool-my-custom").await?;

    let context = SandboxContext {
        working_dir: "/tmp/wireframe".to_string(),
        environment: vec![("TEST_VAR".to_string(), "test_value".to_string())],
        allowed_paths: vec!["/tmp/wireframe".to_string()],
    };

    let params = json!({
        "param1": "test_value",
        "param2": 20,
        "param3": true
    });

    let result = tool_plugin.execute(params, &context).await?;
    println!("Tool result: {}", result);

    Ok(())
}
*/

// ============================================================================
// Advanced: Tool with Retry Logic
// ============================================================================

/*
If your tool needs retry logic, you can add this helper method:

impl MyTool {
    async fn execute_with_retry(&self, params: &Value, context: &SandboxContext) -> Result<Value, ToolError> {
        let mut last_error = None;
        
        for attempt in 0..self.max_retries {
            match self.execute(params.clone(), context).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries - 1 {
                        tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| ToolError::ExecutionFailed("Max retries exceeded".to_string())))
    }
}
*/

// ============================================================================
// Advanced: Tool with Streaming Output
// ============================================================================

/*
If your tool supports streaming, you can modify the execute method to return a stream:

use futures::Stream;

async fn execute_stream(
    &self,
    params: Value,
    sandbox_context: &SandboxContext,
) -> Result<impl Stream<Item = Result<Value, ToolError>>, ToolError> {
    // Implement streaming logic
    // Return a stream of partial results
}
*/

// ============================================================================
// Testing
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_initialization() {
        let mut tool = MyTool::new();
        let config = json!({ "timeout": 30 });
        assert!(tool.initialize(&config).await.is_ok());
    }

    #[tokio::test]
    async fn test_tool_name() {
        let tool = MyTool::new();
        assert_eq!(tool.tool_name(), "my_tool");
    }

    #[tokio::test]
    async fn test_input_schema() {
        let tool = MyTool::new();
        let schema = tool.input_schema();
        assert!(schema.is_object());
        assert!(schema["properties"]["param1"].is_object());
    }

    #[tokio::test]
    async fn test_execute_with_valid_params() {
        let tool = MyTool::new();
        let context = SandboxContext {
            working_dir: "/tmp".to_string(),
            environment: vec![],
            allowed_paths: vec![],
        };

        let params = json!({
            "param1": "test",
            "param2": 20,
            "param3": true
        });

        let result = tool.execute(params, &context).await.unwrap();
        assert!(result["success"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_execute_with_missing_required_param() {
        let tool = MyTool::new();
        let context = SandboxContext {
            working_dir: "/tmp".to_string(),
            environment: vec![],
            allowed_paths: vec![],
        };

        let params = json!({
            "param2": 20
        });

        let result = tool.execute(params, &context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_with_default_params() {
        let tool = MyTool::new();
        let context = SandboxContext {
            working_dir: "/tmp".to_string(),
            environment: vec![],
            allowed_paths: vec![],
        };

        let params = json!({
            "param1": "test"
        });

        let result = tool.execute(params, &context).await.unwrap();
        assert_eq!(result["param2"].as_i64().unwrap(), 10);
        assert_eq!(result["param3"].as_bool().unwrap(), false);
    }
}
