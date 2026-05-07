//! File operations tool — read, write, and list files in the sandbox with path validation.

use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::sandbox::{Tool, ToolError, SandboxContext};
use async_trait::async_trait;
use serde_json::{json, Value};
use tokio::fs;
use tracing::error;

const MAX_PATH_LENGTH: usize = 4096;
const MAX_FILE_SIZE: usize = 10_485_760; // 10MB

/// File tool that handles read, write, and list operations.
pub struct FileTool {
    max_file_size: usize,
}

impl FileTool {
    pub fn new() -> Self {
        Self {
            max_file_size: MAX_FILE_SIZE,
        }
    }

    pub fn with_max_file_size(max_file_size: usize) -> Self {
        Self {
            max_file_size,
        }
    }
}

impl Default for FileTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for FileTool {
    fn plugin_id(&self) -> &'static str {
        "tool-file"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "File operations tool for sandbox"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        if let Some(max_size) = config.get("max_file_size").and_then(|v| v.as_u64()) {
            self.max_file_size = max_size as usize;
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
impl Tool for FileTool {
    fn tool_name(&self) -> &'static str {
        "file"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["read", "write", "list"],
                    "description": "File operation to perform"
                },
                "path": {
                    "type": "string",
                    "description": "File path relative to sandbox root"
                },
                "content": {
                    "type": "string",
                    "description": "Content to write (for write operation)"
                }
            },
            "required": ["operation", "path"]
        })
    }

    async fn execute(
        &self,
        params: Value,
        sandbox_context: &SandboxContext,
    ) -> Result<Value, ToolError> {
        let operation = params.get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing operation".to_string()))?;

        let path = params.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing path".to_string()))?;

        // Validate and resolve path
        let validated_path = self.validate_path(path, sandbox_context)?;

        match operation {
            "read" => self.read_file(&validated_path).await,
            "write" => {
                let content = params.get("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidParameters("Missing content".to_string()))?;
                self.write_file(&validated_path, content).await
            }
            "list" => self.list_directory(&validated_path).await,
            _ => Err(ToolError::InvalidParameters(format!("Unknown operation: {}", operation))),
        }
    }
}

impl FileTool {
    /// Validate a path to prevent directory traversal attacks.
    fn validate_path(&self, path: &str, _context: &SandboxContext) -> Result<String, ToolError> {
        if path.len() > MAX_PATH_LENGTH {
            return Err(ToolError::InvalidParameters(format!(
                "Path too long (max {} chars)",
                MAX_PATH_LENGTH
            )));
        }

        if path.is_empty() {
            return Err(ToolError::InvalidParameters("Empty path".to_string()));
        }

        // Normalize the path
        let normalized = path
            .replace('\\', "/")
            .split('/')
            .filter(|part| !part.is_empty() && *part != ".")
            .collect::<Vec<_>>()
            .join("/");

        if normalized.contains("..") {
            return Err(ToolError::PermissionDenied("Path traversal not allowed".to_string()));
        }

        if path.starts_with('/') || path.starts_with('\\') {
            return Err(ToolError::PermissionDenied("Absolute paths not allowed".to_string()));
        }

        Ok(normalized)
    }

    /// Read a file from the sandbox.
    async fn read_file(&self, path: &str) -> Result<Value, ToolError> {
        let content = fs::read_to_string(path).await.map_err(|e| {
            error!(error = ?e, "failed to read file");
            ToolError::ExecutionFailed(format!("Failed to read file: {}", e))
        })?;

        Ok(json!({ "content": content }))
    }

    /// Write content to a file in the sandbox.
    async fn write_file(&self, path: &str, content: &str) -> Result<Value, ToolError> {
        if content.len() > self.max_file_size {
            return Err(ToolError::InvalidParameters(format!(
                "File too large (max {} bytes)",
                self.max_file_size
            )));
        }

        // Create parent directories if needed
        if let Some(parent) = std::path::Path::new(path).parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                error!(error = ?e, "failed to create directories");
                ToolError::ExecutionFailed(format!("Failed to create directories: {}", e))
            })?;
        }

        fs::write(path, content).await.map_err(|e| {
            error!(error = ?e, "failed to write file");
            ToolError::ExecutionFailed(format!("Failed to write file: {}", e))
        })?;

        Ok(json!({
            "success": true,
            "bytes_written": content.len()
        }))
    }

    /// List files in a directory.
    async fn list_directory(&self, path: &str) -> Result<Value, ToolError> {
        let mut entries = fs::read_dir(path).await.map_err(|e| {
            error!(error = ?e, "failed to list directory");
            ToolError::ExecutionFailed(format!("Failed to list directory: {}", e))
        })?;

        let mut result = Vec::new();
        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            error!(error = ?e, "failed to read entry");
            ToolError::ExecutionFailed(format!("Failed to read entry: {}", e))
        })? {
            let name = entry.file_name().to_string_lossy().to_string();
            let kind = if entry.file_type().await.map(|ft| ft.is_dir()).unwrap_or(false) {
                format!("{}/", name)
            } else {
                name
            };
            result.push(kind);
        }

        Ok(json!(result))
    }
}