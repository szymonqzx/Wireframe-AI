//! MCP Stdio Client — lightweight JSON-RPC client for sandbox communication

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::process::{Child, ChildStdin, ChildStdout, Command as TokioCommand};
use tracing::info;

/// Lightweight MCP JSON-RPC stdio client for communicating with the sandbox.
///
/// Spawns the `wireframe-ai-sandbox-core` binary as a subprocess and communicates
/// via JSON-RPC 2.0 over stdin/stdout.
pub struct McpStdioClient {
    stdin: ChildStdin,
    stdout_reader: tokio::io::BufReader<ChildStdout>,
    _child: Child,
    request_id: std::sync::atomic::AtomicU64,
}

impl McpStdioClient {
    /// Spawn the sandbox binary and perform MCP initialization handshake.
    pub async fn spawn_sandbox(binary_path: Option<&Path>) -> Result<Self> {
        let bin = binary_path
            .map(|p| p.to_path_buf())
            .or_else(|| {
                // Try to find the sandbox binary in the same directory as current exe
                std::env::current_exe().ok().and_then(|exe| {
                    exe.parent()
                        .map(|dir| dir.join("wireframe-ai-sandbox-core"))
                })
            })
            .unwrap_or_else(|| PathBuf::from("wireframe-ai-sandbox-core"));

        info!("Spawning sandbox MCP server: {}", bin.display());

        let mut child = TokioCommand::new(&bin)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .with_context(|| format!("Failed to spawn sandbox binary: {}", bin.display()))?;

        let stdin = child
            .stdin
            .take()
            .context("Failed to capture sandbox stdin")?;
        let stdout = child
            .stdout
            .take()
            .context("Failed to capture sandbox stdout")?;

        let mut client = McpStdioClient {
            stdin,
            stdout_reader: tokio::io::BufReader::new(stdout),
            _child: child,
            request_id: std::sync::atomic::AtomicU64::new(1),
        };

        // MCP initialize handshake
        client.initialize().await?;

        info!("MCP sandbox client initialized successfully");
        Ok(client)
    }

    /// Perform the MCP initialize/initialized handshake.
    async fn initialize(&mut self) -> Result<()> {
        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 0,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "wireframe-adapter-rust",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        });

        let response = self.send_request(&init_request).await?;

        // Check for error
        if response.get("error").is_some() {
            return Err(anyhow::anyhow!(
                "MCP initialize failed: {}",
                serde_json::to_string_pretty(&response).unwrap_or_default()
            ));
        }

        // Send initialized notification
        let initialized = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        self.send_notification(&initialized).await?;

        Ok(())
    }

    /// Send a JSON-RPC request and wait for the response.
    async fn send_request(&mut self, request: &serde_json::Value) -> Result<serde_json::Value> {
        let line = serde_json::to_string(request)?;
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;

        let mut response_line = String::new();
        let bytes_read = self.stdout_reader.read_line(&mut response_line).await?;
        if bytes_read == 0 {
            return Err(anyhow::anyhow!(
                "Sandbox process closed stdout before responding"
            ));
        }

        let response: serde_json::Value = serde_json::from_str(&response_line)
            .with_context(|| format!("Invalid JSON from sandbox: {}", response_line.trim()))?;

        Ok(response)
    }

    /// Send a JSON-RPC notification (no response expected).
    async fn send_notification(&mut self, notification: &serde_json::Value) -> Result<()> {
        let line = serde_json::to_string(notification)?;
        self.stdin.write_all(line.as_bytes()).await?;
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;
        Ok(())
    }

    /// Call an MCP tool and return the result.
    pub async fn call_tool(
        &mut self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let id = self
            .request_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments
            }
        });

        let response = self.send_request(&request).await?;

        // Extract result from MCP response
        if let Some(error) = response.get("error") {
            return Err(anyhow::anyhow!(
                "MCP tool call error: {}",
                serde_json::to_string_pretty(error).unwrap_or_default()
            ));
        }

        let result = response
            .get("result")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        Ok(result)
    }
}
