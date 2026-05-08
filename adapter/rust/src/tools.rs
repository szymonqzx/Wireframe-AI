//! Tool execution handlers for the adapter.
//!
//! Provides modular tool execution with consistent error handling and response formatting.

use serde_json::Value;
use std::env;
use std::path::PathBuf;
use tracing::warn;

use crate::security::{
    sanitize_string, validate_path, validate_path_for_write, validate_shell_command,
};
use wireframe_provider_core::ToolDefinition;

/// Tool names enum to prevent magic string typos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolName {
    ShellExec,
    FileRead,
    FileWrite,
    FileList,
    ReadSource,
    WriteSource,
    CompileSelf,
    RestartSelf,
    SwitchModule,
}

impl ToolName {
    pub fn as_str(&self) -> &str {
        match self {
            ToolName::ShellExec => "shell_exec",
            ToolName::FileRead => "file_read",
            ToolName::FileWrite => "file_write",
            ToolName::FileList => "file_list",
            ToolName::ReadSource => "read_source",
            ToolName::WriteSource => "write_source",
            ToolName::CompileSelf => "compile_self",
            ToolName::RestartSelf => "restart_self",
            ToolName::SwitchModule => "switch_module",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "shell_exec" => Some(ToolName::ShellExec),
            "file_read" => Some(ToolName::FileRead),
            "file_write" => Some(ToolName::FileWrite),
            "file_list" => Some(ToolName::FileList),
            "read_source" => Some(ToolName::ReadSource),
            "write_source" => Some(ToolName::WriteSource),
            "compile_self" => Some(ToolName::CompileSelf),
            "restart_self" => Some(ToolName::RestartSelf),
            "switch_module" => Some(ToolName::SwitchModule),
            _ => None,
        }
    }
}

/// Tool execution context.
pub struct ToolContext<'a> {
    pub allowed_base_dir: Option<&'a PathBuf>,
    pub execution_mode: ExecutionMode,
}

/// Execution mode for tool execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    Direct,
    Sandbox,
}

/// Tool execution result with consistent formatting.
pub fn success_response(data: Value) -> Value {
    let mut response = serde_json::Map::new();
    response.insert("success".to_string(), Value::Bool(true));
    response.insert("data".to_string(), data);
    Value::Object(response)
}

pub fn error_response(message: &str) -> Value {
    let mut response = serde_json::Map::new();
    response.insert("success".to_string(), Value::Bool(false));
    response.insert("error".to_string(), Value::String(message.to_string()));
    Value::Object(response)
}

pub fn missing_param_response(param: &str) -> Value {
    error_response(&format!("Missing '{}' parameter", param))
}

/// Detect the appropriate shell and flag for the current platform.
fn detect_platform_shell() -> (String, String) {
    // Explicit override takes precedence
    if let Ok(explicit) = env::var("WIREFRAME_AI_SHELL") {
        let shell = explicit.trim().to_string();
        if !shell.is_empty() {
            // Assume POSIX-style shell with -c flag
            return (shell, "-c".to_string());
        }
    }

    if cfg!(target_os = "windows") {
        // COMSPEC is the official Windows shell environment variable
        if let Ok(comspec) = env::var("COMSPEC") {
            let shell = comspec.trim().to_string();
            if !shell.is_empty() {
                // PowerShell uses -Command, cmd uses /C
                let flag = if shell.to_lowercase().contains("powershell")
                    || shell.to_lowercase().contains("pwsh")
                {
                    "-Command"
                } else {
                    "/C"
                };
                return (shell, flag.to_string());
            }
        }
        // Default to cmd with /C flag
        ("cmd".to_string(), "/C".to_string())
    } else {
        // Unix-like systems
        if let Ok(shell_env) = env::var("SHELL") {
            let shell = shell_env.trim().to_string();
            if !shell.is_empty() {
                return (shell, "-c".to_string());
            }
        }
        // Default to sh with -c flag
        ("sh".to_string(), "-c".to_string())
    }
}

/// Execute shell command.
pub async fn execute_shell(
    command: &str,
    working_dir: Option<&str>,
    _ctx: &ToolContext<'_>,
) -> Value {
    if command.is_empty() {
        return missing_param_response("command");
    }

    match validate_shell_command(command) {
        Ok(validated) => {
            let (shell, flag) = detect_platform_shell();
            let mut cmd = tokio::process::Command::new(&shell);
            cmd.args([&flag, &validated]);

            if let Some(dir) = working_dir {
                cmd.current_dir(dir);
            }

            match cmd.output().await {
                Ok(output) => {
                    serde_json::json!({
                        "success": output.status.success(),
                        "stdout": String::from_utf8_lossy(&output.stdout).to_string(),
                        "stderr": String::from_utf8_lossy(&output.stderr).to_string(),
                        "exit_code": output.status.code().unwrap_or(-1)
                    })
                }
                Err(e) => error_response(&format!("Command execution failed: {}", e)),
            }
        }
        Err(e) => {
            warn!("Command validation failed: {}", e);
            error_response(&format!("Command validation failed: {}", e))
        }
    }
}

/// Read file content.
pub async fn read_file(path: &str, ctx: &ToolContext<'_>) -> Value {
    if path.is_empty() {
        return missing_param_response("path");
    }

    let sanitized = sanitize_string(path);
    match validate_path(
        sanitized.as_ref(),
        ctx.allowed_base_dir.map(|p| p.as_path()),
    ) {
        Ok(validated_path) => match tokio::fs::read_to_string(&validated_path).await {
            Ok(content) => {
                serde_json::json!({
                    "success": true,
                    "content": content,
                    "path": validated_path.to_string_lossy().to_string()
                })
            }
            Err(e) => error_response(&format!("Failed to read file: {}", e)),
        },
        Err(e) => {
            warn!("Path validation failed: {}", e);
            error_response(&format!("Path validation failed: {}", e))
        }
    }
}

/// Write content to file.
pub async fn write_file(path: &str, content: &str, ctx: &ToolContext<'_>) -> Value {
    if path.is_empty() {
        return missing_param_response("path");
    }
    if content.is_empty() {
        return missing_param_response("content");
    }

    let sanitized_path = sanitize_string(path);
    let sanitized_content = sanitize_string(content);
    match validate_path_for_write(
        sanitized_path.as_ref(),
        ctx.allowed_base_dir.map(|p| p.as_path()),
    ) {
        Ok(validated_path) => {
            match tokio::fs::write(&validated_path, sanitized_content.as_ref()).await {
                Ok(_) => {
                    serde_json::json!({
                        "success": true,
                        "path": validated_path.to_string_lossy().to_string()
                    })
                }
                Err(e) => error_response(&format!("Failed to write file: {}", e)),
            }
        }
        Err(e) => {
            warn!("Path validation failed: {}", e);
            error_response(&format!("Path validation failed: {}", e))
        }
    }
}

/// List directory contents.
pub async fn list_directory(path: &str, ctx: &ToolContext<'_>) -> Value {
    if path.is_empty() {
        return missing_param_response("path");
    }

    let sanitized = sanitize_string(path);
    match validate_path(
        sanitized.as_ref(),
        ctx.allowed_base_dir.map(|p| p.as_path()),
    ) {
        Ok(validated_path) => match tokio::fs::read_dir(&validated_path).await {
            Ok(mut entries) => {
                let mut files = Vec::new();
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let metadata = entry.metadata().await;
                    let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                    files.push(serde_json::json!({
                        "name": entry.file_name().to_string_lossy().to_string(),
                        "is_directory": is_dir
                    }));
                }
                serde_json::json!({
                    "success": true,
                    "path": validated_path.to_string_lossy().to_string(),
                    "files": files
                })
            }
            Err(e) => error_response(&format!("Failed to list directory: {}", e)),
        },
        Err(e) => {
            warn!("Path validation failed: {}", e);
            error_response(&format!("Path validation failed: {}", e))
        }
    }
}

/// Build tool definitions for the provider.
pub fn build_tool_definitions() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: ToolName::ShellExec.as_str().to_string(),
            description: "Execute a shell command in the sandbox environment".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "Shell command to execute"
                    },
                    "working_dir": {
                        "type": "string",
                        "description": "Working directory relative to sandbox root"
                    }
                },
                "required": ["command"]
            }),
        },
        ToolDefinition {
            name: ToolName::FileRead.as_str().to_string(),
            description: "Read the contents of a file in the sandbox".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path relative to sandbox root"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: ToolName::FileWrite.as_str().to_string(),
            description: "Write content to a file in the sandbox".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path relative to sandbox root"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write to the file"
                    }
                },
                "required": ["path", "content"]
            }),
        },
        ToolDefinition {
            name: ToolName::FileList.as_str().to_string(),
            description: "List files and directories in a sandbox path".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path relative to sandbox root"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: ToolName::ReadSource.as_str().to_string(),
            description: "Read source code file from the module's source directory (selfdev)".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path relative to source root"
                    }
                },
                "required": ["path"]
            }),
        },
        ToolDefinition {
            name: ToolName::WriteSource.as_str().to_string(),
            description: "Write content to a source code file in the module's source directory (selfdev)".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "File path relative to source root"
                    },
                    "content": {
                        "type": "string",
                        "description": "Content to write to the file"
                    }
                },
                "required": ["path", "content"]
            }),
        },
        ToolDefinition {
            name: ToolName::CompileSelf.as_str().to_string(),
            description: "Compile the module with the current source code changes (selfdev)".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        },
        ToolDefinition {
            name: ToolName::RestartSelf.as_str().to_string(),
            description: "Restart the module with the newly compiled binary (selfdev). This will terminate the current process and start a new one.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "auto_restart": {
                        "type": "boolean",
                        "description": "Whether to automatically restart after compilation"
                    }
                },
                "required": []
            }),
        },
        ToolDefinition {
            name: ToolName::SwitchModule.as_str().to_string(),
            description: "Switch to a different module at runtime. This will stop the current module and start a new one.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "new_module": {
                        "type": "string",
                        "description": "ID of the module to switch to (e.g., 'community-adapter-x')"
                    },
                    "force": {
                        "type": "boolean",
                        "description": "Skip compatibility checks if true"
                    }
                },
                "required": ["new_module"]
            }),
        },
    ]
}
