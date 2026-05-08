//! Whitelist security policy — validates commands against an allowed command whitelist.

use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::sandbox::{FileOperation, SecurityError, SecurityPolicy};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashSet;
use tracing::warn;

/// Whitelist security policy that only allows specific commands.
pub struct WhitelistPolicy {
    allowed_commands: HashSet<String>,
    allow_network: bool,
    filesystem_policy: FilesystemPolicy,
}

#[derive(Debug, Clone, Copy)]
enum FilesystemPolicy {
    ReadOnly,
    Writable,
    SandboxWritable,
}

impl WhitelistPolicy {
    pub fn new() -> Self {
        Self {
            allowed_commands: Self::default_whitelist(),
            allow_network: false,
            filesystem_policy: FilesystemPolicy::SandboxWritable,
        }
    }

    pub fn with_allowed_commands(commands: Vec<String>) -> Self {
        Self {
            allowed_commands: commands.into_iter().collect(),
            allow_network: false,
            filesystem_policy: FilesystemPolicy::SandboxWritable,
        }
    }

    pub fn allow_network(mut self, allow: bool) -> Self {
        self.allow_network = allow;
        self
    }

    pub fn filesystem_policy(mut self, policy: &str) -> Self {
        self.filesystem_policy = match policy {
            "readonly" => FilesystemPolicy::ReadOnly,
            "writable" => FilesystemPolicy::Writable,
            "sandbox_writable" => FilesystemPolicy::SandboxWritable,
            _ => FilesystemPolicy::SandboxWritable,
        };
        self
    }

    fn default_whitelist() -> HashSet<String> {
        [
            "python",
            "python3",
            "pip",
            "pip3",
            "node",
            "npm",
            "npx",
            "yarn",
            "pnpm",
            "cargo",
            "rustc",
            "go",
            "gofmt",
            "java",
            "javac",
            "gcc",
            "g++",
            "clang",
            "clang++",
            "make",
            "cmake",
            "git",
            "ls",
            "dir",
            "cd",
            "cat",
            "type",
            "head",
            "tail",
            "grep",
            "find",
            "locate",
            "cp",
            "mv",
            "rm",
            "rmdir",
            "mkdir",
            "chmod",
            "chown",
            "tar",
            "zip",
            "unzip",
            "curl",
            "wget",
            "pytest",
            "jest",
            "mocha",
            "black",
            "ruff",
            "flake8",
            "mypy",
            "clang-format",
            "prettier",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }
}

impl Default for WhitelistPolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for WhitelistPolicy {
    fn plugin_id(&self) -> &'static str {
        "policy-whitelist"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Whitelist security policy for sandbox"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        if let Some(commands) = config.get("allowed_commands").and_then(|v| v.as_array()) {
            self.allowed_commands = commands
                .iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect();
        }

        if let Some(allow) = config.get("allow_network").and_then(|v| v.as_bool()) {
            self.allow_network = allow;
        }

        if let Some(policy) = config.get("filesystem_policy").and_then(|v| v.as_str()) {
            self.filesystem_policy = match policy {
                "readonly" => FilesystemPolicy::ReadOnly,
                "writable" => FilesystemPolicy::Writable,
                "sandbox_writable" => FilesystemPolicy::SandboxWritable,
                _ => FilesystemPolicy::SandboxWritable,
            };
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
impl SecurityPolicy for WhitelistPolicy {
    async fn validate_command(
        &self,
        command: &str,
        _working_dir: &str,
    ) -> Result<bool, SecurityError> {
        // Extract the executable basename
        let basename = command
            .split_whitespace()
            .next()
            .unwrap_or(command)
            .rsplit('/')
            .next()
            .unwrap_or(command)
            .rsplit('\\')
            .next()
            .unwrap_or(command);

        if !self.allowed_commands.contains(basename) {
            warn!(executable = %basename, command = %command, "command not in whitelist");
            return Err(SecurityError::CommandRejected(format!(
                "Command '{}' is not in the allowed whitelist",
                basename
            )));
        }

        Ok(true)
    }

    async fn validate_file_access(
        &self,
        _path: &str,
        operation: FileOperation,
    ) -> Result<bool, SecurityError> {
        match self.filesystem_policy {
            FilesystemPolicy::ReadOnly => {
                if matches!(operation, FileOperation::Write | FileOperation::Delete) {
                    return Err(SecurityError::FileAccessDenied(
                        "Filesystem is read-only".to_string(),
                    ));
                }
            }
            FilesystemPolicy::SandboxWritable => {
                // Allow all operations within sandbox (path validation handled elsewhere)
            }
            FilesystemPolicy::Writable => {
                // Allow all operations
            }
        }
        Ok(true)
    }

    async fn validate_network_access(&self, _url: &str) -> Result<bool, SecurityError> {
        if !self.allow_network {
            return Err(SecurityError::NetworkAccessDenied(
                "Network access is disabled".to_string(),
            ));
        }
        Ok(true)
    }
}
