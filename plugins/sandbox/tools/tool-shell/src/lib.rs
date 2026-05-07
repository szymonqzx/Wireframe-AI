//! Shell execution tool — runs commands in the sandbox with validation and resource limits.

use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::sandbox::{Tool, ToolError, SandboxContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use shell_words::split;
use tokio::process::Command;
use tracing::error;

const MAX_COMMAND_LENGTH: usize = 1000;
const MAX_ARGS: usize = 50;
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Shell tool that executes commands in the sandbox.
pub struct ShellTool {
    timeout_secs: u64,
}

impl ShellTool {
    pub fn new() -> Self {
        Self {
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }

    pub fn with_timeout(timeout_secs: u64) -> Self {
        Self {
            timeout_secs,
        }
    }
}

impl Default for ShellTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for ShellTool {
    fn plugin_id(&self) -> &'static str {
        "tool-shell"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Shell execution tool for sandbox"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        if let Some(timeout) = config.get("timeout_secs").and_then(|v| v.as_u64()) {
            self.timeout_secs = timeout;
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
impl Tool for ShellTool {
    fn tool_name(&self) -> &'static str {
        "shell"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Command to run as a single string"
                },
                "working_dir": {
                    "type": "string",
                    "description": "Working directory relative to sandbox root"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Timeout in seconds (default 30)"
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(
        &self,
        params: Value,
        sandbox_context: &SandboxContext,
    ) -> Result<Value, ToolError> {
        let command = params.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing command".to_string()))?;

        let working_dir = params.get("working_dir")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let timeout_secs = params.get("timeout_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(self.timeout_secs);

        // Validate command length
        if command.len() > MAX_COMMAND_LENGTH {
            return Err(ToolError::InvalidParameters(format!(
                "Command too long (max {} chars)",
                MAX_COMMAND_LENGTH
            )));
        }

        // Parse command into executable and arguments
        let parts = split(command).map_err(|e| {
            error!(error = ?e, "failed to parse command");
            ToolError::InvalidParameters(format!("Failed to parse command: {}", e))
        })?;

        if parts.is_empty() {
            return Err(ToolError::InvalidParameters("Empty command after parsing".to_string()));
        }

        if parts.len() > MAX_ARGS {
            return Err(ToolError::InvalidParameters(format!(
                "Too many arguments (max {})",
                MAX_ARGS
            )));
        }

        // Check for shell metacharacters in arguments
        let shell_chars = ['|', '&', ';', '$', '`', '(', ')', '<', '>', '\\', '\n', '\r'];
        for part in &parts[1..] {
            for &c in &shell_chars {
                if part.contains(c) {
                    return Err(ToolError::InvalidParameters(format!(
                        "Shell metacharacter '{}' not allowed in arguments",
                        c
                    )));
                }
            }
        }

        let executable = &parts[0];
        let args = &parts[1..];

        // Resolve working directory
        let dir = if working_dir.is_empty() {
            sandbox_context.working_dir.clone()
        } else {
            format!("{}/{}", sandbox_context.working_dir, working_dir)
        };

        // Execute command
        let mut cmd = Command::new(executable.as_str());
        cmd.args(args);
        cmd.current_dir(&dir);

        let output = match tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            cmd.output()
        ).await {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => {
                error!(error = ?e, "command failed");
                return Err(ToolError::ExecutionFailed(format!("exec error: {}", e)));
            }
            Err(_) => {
                return Err(ToolError::Timeout);
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);

        Ok(json!({
            "success": output.status.success(),
            "stdout": stdout,
            "stderr": stderr,
            "exit_code": exit_code,
        }))
    }
}