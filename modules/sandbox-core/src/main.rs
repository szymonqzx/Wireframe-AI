//! Wireframe-AI Sandbox Core — Minimal MCP Server
//!
//! Runs as an MCP server over stdio for tool execution.
//! Minimal implementation that can be expanded by plugins.
//!
//! Architecture:
//! - MCP server over stdio (JSON-RPC 2.0)
//! - Plugin system for tools, security, and resource limits
//! - Default tools: file operations, shell execution
//! - Extensible via plugin registration

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tracing::{info, warn};
use wireframe_ai_sandbox_core::{SandboxCore, WhitelistPolicy, UnixResourceLimiter};

/// Minimal MCP server implementation
struct McpServer {
    sandbox: Arc<SandboxCore>,
}

/// JSON-RPC 2.0 Request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    _jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: Option<Value>,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

/// Tool definition
#[derive(Debug, Serialize)]
struct Tool {
    name: String,
    description: String,
    input_schema: Value,
}

impl McpServer {
    fn new(sandbox: Arc<SandboxCore>) -> Self {
        Self { sandbox }
    }

    /// List available tools
    fn list_tools(&self) -> Vec<Tool> {
        vec![
            Tool {
                name: "file_read".to_string(),
                description: "Read a file from the sandbox".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "File path to read"}
                    },
                    "required": ["path"]
                }),
            },
            Tool {
                name: "file_write".to_string(),
                description: "Write content to a file in the sandbox".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "File path to write"},
                        "content": {"type": "string", "description": "Content to write"}
                    },
                    "required": ["path", "content"]
                }),
            },
            Tool {
                name: "shell_exec".to_string(),
                description: "Execute a shell command in the sandbox".to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": {"type": "string", "description": "Shell command to execute"}
                    },
                    "required": ["command"]
                }),
            },
        ]
    }

    /// Execute a tool call
    async fn execute_tool(&self, tool_name: &str, arguments: Value) -> Result<Value, String> {
        match tool_name {
            "file_read" => self.execute_file_read(arguments).await,
            "file_write" => self.execute_file_write(arguments).await,
            "shell_exec" => self.execute_shell_exec(arguments).await,
            _ => Err(format!("Unknown tool: {}", tool_name)),
        }
    }

    async fn execute_file_read(&self, arguments: Value) -> Result<Value, String> {
        let path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'path' parameter".to_string())?;

        let full_path = std::path::PathBuf::from(self.sandbox.sandbox_root()).join(path);

        // Basic security check: ensure path is within sandbox root
        if !full_path.starts_with(self.sandbox.sandbox_root()) {
            return Err("Path outside sandbox root".to_string());
        }

        match tokio::fs::read_to_string(&full_path).await {
            Ok(content) => Ok(serde_json::json!({
                "content": content,
                "path": path
            })),
            Err(e) => Err(format!("Failed to read file: {}", e)),
        }
    }

    async fn execute_file_write(&self, arguments: Value) -> Result<Value, String> {
        let path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'path' parameter".to_string())?;

        let content = arguments
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'content' parameter".to_string())?;

        let full_path = std::path::PathBuf::from(self.sandbox.sandbox_root()).join(path);

        // Basic security check: ensure path is within sandbox root
        if !full_path.starts_with(self.sandbox.sandbox_root()) {
            return Err("Path outside sandbox root".to_string());
        }

        // Ensure parent directory exists
        if let Some(parent) = full_path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                return Err(format!("Failed to create directory: {}", e));
            }
        }

        match tokio::fs::write(&full_path, content).await {
            Ok(_) => Ok(serde_json::json!({
                "success": true,
                "path": path
            })),
            Err(e) => Err(format!("Failed to write file: {}", e)),
        }
    }

    async fn execute_shell_exec(&self, arguments: Value) -> Result<Value, String> {
        let command = arguments
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'command' parameter".to_string())?;

        // Basic security: allowlist of safe commands
        let safe_commands = ["ls", "pwd", "echo", "cat", "grep", "find", "head", "tail"];
        let base_cmd = command.split_whitespace().next().unwrap_or("");

        if !safe_commands.contains(&base_cmd) {
            return Err(format!("Command '{}' not in allowlist", base_cmd));
        }

        // Execute command in sandbox root
        match tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .current_dir(self.sandbox.sandbox_root())
            .output()
            .await
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                Ok(serde_json::json!({
                    "stdout": stdout,
                    "stderr": stderr,
                    "exit_code": output.status.code()
                }))
            }
            Err(e) => Err(format!("Failed to execute command: {}", e)),
        }
    }

    /// Handle a JSON-RPC request
    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = match request.method.as_str() {
            "initialize" => {
                // MCP initialize handshake
                serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "serverInfo": {
                        "name": "wireframe-ai-sandbox-core",
                        "version": "0.1.0"
                    }
                })
            }
            "tools/list" => {
                // List available tools
                serde_json::json!({
                    "tools": self.list_tools()
                })
            }
            "tools/call" => {
                // Execute a tool
                if let Some(params) = request.params {
                    let tool_name = params.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);

                    match self.execute_tool(tool_name, arguments).await {
                        Ok(result) => serde_json::json!({
                            "content": [{"type": "text", "text": result}]
                        }),
                        Err(e) => {
                            return JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                result: None,
                                error: Some(JsonRpcError {
                                    code: -32603,
                                    message: e,
                                    data: None,
                                }),
                                id: request.id,
                            }
                        }
                    }
                } else {
                    serde_json::json!({"error": "Missing parameters"})
                }
            }
            _ => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: format!("Method not found: {}", request.method),
                        data: None,
                    }),
                    id: request.id,
                }
            }
        };

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id: request.id,
        }
    }

    /// Run the MCP server on stdio
    async fn run_stdio(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let mut reader = tokio::io::BufReader::new(stdin);
        let mut writer = stdout;

        loop {
            let mut line = String::new();
            let bytes_read = reader.read_line(&mut line).await?;
            if bytes_read == 0 {
                break; // EOF
            }

            let request: JsonRpcRequest = match serde_json::from_str(&line.trim()) {
                Ok(req) => req,
                Err(e) => {
                    eprintln!("Failed to parse JSON-RPC request: {}", e);
                    continue;
                }
            };

            let response = self.handle_request(request).await;
            let response_json = serde_json::to_string(&response)?;
            writer.write_all(response_json.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            writer.flush().await?;
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Get sandbox root from environment or use temp dir
    let sandbox_root = std::env::var("WIREFRAME_AI_SANDBOX_ROOT")
        .unwrap_or_else(|_| {
            std::env::temp_dir()
                .join("wireframe-ai-sandbox")
                .to_string_lossy()
                .to_string()
        });

    // Create sandbox root directory
    tokio::fs::create_dir_all(&sandbox_root).await?;
    info!("Sandbox root: {}", sandbox_root);

    // Create sandbox core
    let sandbox = Arc::new(SandboxCore::new(sandbox_root.clone()));

    // Load plugins from configuration if available
    let config_path = std::env::var("WIREFRAME_AI_SANDBOX_CONFIG").ok();
    if let Some(ref config_path) = config_path {
        info!("Loading plugins from config: {}", config_path);
        match sandbox.load_plugins_from_config(config_path).await {
            Ok(_) => info!("Plugins loaded successfully"),
            Err(e) => {
                warn!("Failed to load plugins from config: {}, using built-in tools", e);
                // Load built-in security and resource plugins as fallback
                let default_config = serde_json::json!({
                    "allowed_paths": [sandbox_root.clone()],
                    "max_execution_time_secs": 60,
                    "max_memory_mb": 512
                });
                sandbox.set_security(Arc::new(WhitelistPolicy::new(&default_config))).await;
                sandbox.set_resource_limiter(Arc::new(UnixResourceLimiter::new(&default_config))).await;
                info!("Loaded built-in security and resource plugins");
            }
        }
    } else {
        info!("No plugin config specified, using built-in tools");
        // Load built-in security and resource plugins
        let default_config = serde_json::json!({
            "allowed_paths": [sandbox_root.clone()],
            "max_execution_time_secs": 60,
            "max_memory_mb": 512
        });
        sandbox.set_security(Arc::new(WhitelistPolicy::new(&default_config))).await;
        sandbox.set_resource_limiter(Arc::new(UnixResourceLimiter::new(&default_config))).await;
        info!("Loaded built-in security and resource plugins");
    }

    // Create MCP server
    let server = McpServer::new(sandbox);

    // Start MCP server on stdio
    info!("Starting MCP server on stdio");
    server.run_stdio().await?;

    Ok(())
}
