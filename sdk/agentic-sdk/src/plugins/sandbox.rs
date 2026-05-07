//! Plugin traits for the Sandbox module.

use crate::plugin::Plugin;
use async_trait::async_trait;
use serde_json::Value;
use thiserror::Error;

/// Tool implementation.
///
/// Implementations provide executable tools (shell, file, git, HTTP, etc.)
/// that can be invoked by reasoning adapters.
#[async_trait]
pub trait Tool: Plugin {
    /// Tool name (e.g., "shell", "file", "git").
    fn tool_name(&self) -> &'static str;

    /// JSON schema for tool input validation.
    fn input_schema(&self) -> Value;

    /// Execute the tool with given parameters.
    async fn execute(
        &self,
        params: Value,
        sandbox_context: &SandboxContext,
    ) -> Result<Value, ToolError>;
}

/// Security policy enforcement.
///
/// Implementations validate commands, file access, and network access
/// according to security policies (whitelist, strict, permissive, etc.).
#[async_trait]
pub trait SecurityPolicy: Plugin {
    /// Validate a shell command.
    async fn validate_command(
        &self,
        command: &str,
        working_dir: &str,
    ) -> Result<bool, SecurityError>;

    /// Validate file system access.
    async fn validate_file_access(
        &self,
        path: &str,
        operation: FileOperation,
    ) -> Result<bool, SecurityError>;

    /// Validate network access.
    async fn validate_network_access(&self, url: &str) -> Result<bool, SecurityError>;
}

/// Resource limit enforcement.
///
/// Implementations enforce CPU, memory, and timeout limits
/// to prevent resource exhaustion.
#[async_trait]
pub trait ResourceLimiter: Plugin {
    /// Check CPU time limit.
    async fn check_cpu_limit(
        &self,
        current_usage: std::time::Duration,
    ) -> Result<bool, ResourceError>;

    /// Check memory limit.
    async fn check_memory_limit(&self, current_usage: usize) -> Result<bool, ResourceError>;

    /// Enforce timeout.
    async fn enforce_timeout(
        &self,
        started_at: std::time::Instant,
        timeout: std::time::Duration,
    ) -> Result<(), ResourceError>;
}

// ============================================================================
// Data Types
// ============================================================================

/// Context provided to tool execution.
#[derive(Debug, Clone)]
pub struct SandboxContext {
    pub working_dir: String,
    pub environment: Vec<(String, String)>,
    pub allowed_paths: Vec<String>,
}

/// File system operation type.
#[derive(Debug, Clone, Copy)]
pub enum FileOperation {
    Read,
    Write,
    Delete,
    Execute,
}

// ============================================================================
// Error Types
// ============================================================================

/// Tool execution error.
#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Timeout")]
    Timeout,
}

/// Security policy error.
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Command rejected: {0}")]
    CommandRejected(String),

    #[error("File access denied: {0}")]
    FileAccessDenied(String),

    #[error("Network access denied: {0}")]
    NetworkAccessDenied(String),

    #[error("Policy violation: {0}")]
    PolicyViolation(String),
}

/// Resource limit error.
#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("CPU limit exceeded")]
    CpuLimitExceeded,

    #[error("Memory limit exceeded")]
    MemoryLimitExceeded,

    #[error("Timeout exceeded")]
    TimeoutExceeded,

    #[error("Resource monitoring failed: {0}")]
    MonitoringFailed(String),
}
