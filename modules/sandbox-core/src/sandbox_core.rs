//! Sandbox core — MCP server orchestration and plugin lifecycle management.

use agentic_sdk::plugins::sandbox::{Tool, SecurityPolicy, ResourceLimiter, SandboxContext};
use agentic_sdk::plugin::Plugin;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Plugin configuration structure.
#[derive(Debug, Deserialize, Serialize)]
pub struct PluginConfig {
    pub tools: Vec<ToolPluginConfig>,
    pub security: Vec<SecurityPluginConfig>,
    pub resources: Vec<ResourcePluginConfig>,
}

/// Tool plugin configuration.
#[derive(Debug, Deserialize, Serialize)]
pub struct ToolPluginConfig {
    pub name: String,
    pub config: serde_json::Value,
}

/// Security plugin configuration.
#[derive(Debug, Deserialize, Serialize)]
pub struct SecurityPluginConfig {
    pub name: String,
    pub config: serde_json::Value,
}

/// Resource plugin configuration.
#[derive(Debug, Deserialize, Serialize)]
pub struct ResourcePluginConfig {
    pub name: String,
    pub config: serde_json::Value,
}

/// Sandbox core manages plugin lifecycle and coordinates tool execution.
pub struct SandboxCore {
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
    security: Arc<RwLock<Option<Arc<dyn SecurityPolicy>>>>,
    resource_limiter: Arc<RwLock<Option<Arc<dyn ResourceLimiter>>>>,
    sandbox_root: String,
}

impl SandboxCore {
    /// Create a new sandbox core with the given sandbox root directory.
    pub fn new(sandbox_root: String) -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            security: Arc::new(RwLock::new(None)),
            resource_limiter: Arc::new(RwLock::new(None)),
            sandbox_root,
        }
    }

    /// Load plugins from configuration file.
    pub async fn load_plugins_from_config(&self, config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config_content = tokio::fs::read_to_string(config_path).await?;
        let config: PluginConfig = serde_yaml::from_str(&config_content)?;

        // Load tool plugins
        for tool_config in config.tools {
            if let Some(tool) = self.create_tool_plugin(&tool_config.name, &tool_config.config).await {
                self.register_tool(tool).await;
                tracing::info!("Loaded tool plugin: {}", tool_config.name);
            }
        }

        // Load security plugins
        for security_config in config.security {
            if let Some(security) = self.create_security_plugin(&security_config.name, &security_config.config).await {
                self.set_security(security).await;
                tracing::info!("Loaded security plugin: {}", security_config.name);
            }
        }

        // Load resource plugins
        for resource_config in config.resources {
            if let Some(limiter) = self.create_resource_plugin(&resource_config.name, &resource_config.config).await {
                self.set_resource_limiter(limiter).await;
                tracing::info!("Loaded resource plugin: {}", resource_config.name);
            }
        }

        Ok(())
    }

    /// Create a tool plugin by name (placeholder for dynamic loading).
    async fn create_tool_plugin(&self, name: &str, _config: &serde_json::Value) -> Option<Arc<dyn Tool>> {
        // For now, return None - this would be implemented with dynamic loading
        tracing::warn!("Tool plugin creation not implemented for: {}", name);
        None
    }

    /// Create a security plugin by name (placeholder for dynamic loading).
    async fn create_security_plugin(&self, name: &str, config: &serde_json::Value) -> Option<Arc<dyn SecurityPolicy>> {
        match name {
            "whitelist" => Some(Arc::new(WhitelistPolicy::new(config))),
            _ => {
                tracing::warn!("Security plugin not found: {}", name);
                None
            }
        }
    }

    /// Create a resource plugin by name (placeholder for dynamic loading).
    async fn create_resource_plugin(&self, name: &str, config: &serde_json::Value) -> Option<Arc<dyn ResourceLimiter>> {
        match name {
            "unix" => Some(Arc::new(UnixResourceLimiter::new(config))),
            _ => {
                tracing::warn!("Resource plugin not found: {}", name);
                None
            }
        }
    }

    /// Register a tool plugin.
    pub async fn register_tool(&self, tool: Arc<dyn Tool>) {
        let mut tools = self.tools.write().await;
        tools.insert(tool.tool_name().to_string(), tool);
    }

    /// Set the security policy plugin.
    pub async fn set_security(&self, security: Arc<dyn SecurityPolicy>) {
        *self.security.write().await = Some(security);
    }

    /// Set the resource limiter plugin.
    pub async fn set_resource_limiter(&self, limiter: Arc<dyn ResourceLimiter>) {
        *self.resource_limiter.write().await = Some(limiter);
    }

    /// Execute a tool with given parameters.
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        // Security check
        if let Some(security) = self.security.read().await.as_ref() {
            let context = SandboxContext {
                working_dir: self.sandbox_root.clone(),
                environment: vec![],
                allowed_paths: vec![self.sandbox_root.clone()],
            };

            // Validate file operations
            if tool_name == "file_read" {
                if let Some(path) = params.get("path").and_then(|v| v.as_str()) {
                    security.validate_file_access(path, agentic_sdk::plugins::sandbox::FileOperation::Read).await?;
                }
            } else if tool_name == "file_write" {
                if let Some(path) = params.get("path").and_then(|v| v.as_str()) {
                    security.validate_file_access(path, agentic_sdk::plugins::sandbox::FileOperation::Write).await?;
                }
            } else if tool_name == "shell_exec" {
                if let Some(command) = params.get("command").and_then(|v| v.as_str()) {
                    security.validate_command(command, &context.working_dir).await?;
                }
            }
        }

        // Execute tool
        let tools = self.tools.read().await;
        let tool = tools.get(tool_name).ok_or("Tool not found")?;

        let context = SandboxContext {
            working_dir: self.sandbox_root.clone(),
            environment: vec![],
            allowed_paths: vec![self.sandbox_root.clone()],
        };

        let result = tool.execute(params, &context).await.map_err(|e| {
            Box::<dyn std::error::Error>::from(e)
        })?;

        Ok(result)
    }

    /// Get the sandbox root directory.
    pub fn sandbox_root(&self) -> &str {
        &self.sandbox_root
    }

    /// Get the security policy.
    pub async fn security(&self) -> Option<Arc<dyn SecurityPolicy>> {
        self.security.read().await.clone()
    }

    /// Get the resource limiter.
    pub async fn resource_limiter(&self) -> Option<Arc<dyn ResourceLimiter>> {
        self.resource_limiter.read().await.clone()
    }
}

// ============================================================================
// Built-in Security Policy: Whitelist
// ============================================================================

pub struct WhitelistPolicy {
    allowed_paths: Vec<String>,
    allowed_commands: Vec<String>,
}

impl WhitelistPolicy {
    pub fn new(config: &serde_json::Value) -> Self {
        let allowed_paths = config
            .get("allowed_paths")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let allowed_commands = config
            .get("allowed_commands")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_else(|| vec![
                "ls".to_string(),
                "pwd".to_string(),
                "echo".to_string(),
                "cat".to_string(),
                "grep".to_string(),
                "find".to_string(),
                "head".to_string(),
                "tail".to_string(),
            ]);

        Self {
            allowed_paths,
            allowed_commands,
        }
    }
}

#[async_trait::async_trait]
impl Plugin for WhitelistPolicy {
    fn plugin_id(&self) -> &'static str {
        "whitelist-policy"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn description(&self) -> &'static str {
        "Whitelist-based security policy"
    }

    async fn initialize(&mut self, _config: &serde_json::Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, agentic_sdk::plugin::PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SecurityPolicy for WhitelistPolicy {
    async fn validate_command(
        &self,
        command: &str,
        _working_dir: &str,
    ) -> Result<bool, agentic_sdk::plugins::sandbox::SecurityError> {
        let base_cmd = command.split_whitespace().next().unwrap_or("");

        if !self.allowed_commands.contains(&base_cmd.to_string()) {
            return Err(agentic_sdk::plugins::sandbox::SecurityError::CommandRejected(format!(
                "Command '{}' not in whitelist",
                base_cmd
            )));
        }

        Ok(true)
    }

    async fn validate_file_access(
        &self,
        path: &str,
        _operation: agentic_sdk::plugins::sandbox::FileOperation,
    ) -> Result<bool, agentic_sdk::plugins::sandbox::SecurityError> {
        // Check for path traversal
        if path.contains("..") {
            return Err(agentic_sdk::plugins::sandbox::SecurityError::FileAccessDenied(
                "Path traversal not allowed".to_string(),
            ));
        }

        // Check if path is in allowed paths
        let is_allowed = self.allowed_paths.is_empty()
            || self.allowed_paths.iter().any(|allowed| path.starts_with(allowed));

        if !is_allowed {
            return Err(agentic_sdk::plugins::sandbox::SecurityError::FileAccessDenied(format!(
                "Path '{}' not in whitelist",
                path
            )));
        }

        Ok(true)
    }

    async fn validate_network_access(&self, _url: &str) -> Result<bool, agentic_sdk::plugins::sandbox::SecurityError> {
        // Default: deny all network access
        Err(agentic_sdk::plugins::sandbox::SecurityError::NetworkAccessDenied(
            "Network access not allowed".to_string(),
        ))
    }
}

// ============================================================================
// Built-in Resource Limiter: Unix
// ============================================================================

pub struct UnixResourceLimiter {
    _max_execution_time_secs: u64,
    max_memory_mb: u64,
    _max_file_size_mb: u64,
}

impl UnixResourceLimiter {
    pub fn new(config: &serde_json::Value) -> Self {
        let max_execution_time_secs = config
            .get("max_execution_time_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(60);

        let max_memory_mb = config
            .get("max_memory_mb")
            .and_then(|v| v.as_u64())
            .unwrap_or(512);

        let max_file_size_mb = config
            .get("max_file_size_mb")
            .and_then(|v| v.as_u64())
            .unwrap_or(10);

        Self {
            _max_execution_time_secs: max_execution_time_secs,
            max_memory_mb,
            _max_file_size_mb: max_file_size_mb,
        }
    }
}

#[async_trait::async_trait]
impl Plugin for UnixResourceLimiter {
    fn plugin_id(&self) -> &'static str {
        "unix-resource-limiter"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn description(&self) -> &'static str {
        "Unix-specific resource limiter"
    }

    async fn initialize(&mut self, _config: &serde_json::Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, agentic_sdk::plugin::PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl ResourceLimiter for UnixResourceLimiter {
    async fn check_cpu_limit(
        &self,
        current_usage: std::time::Duration,
    ) -> Result<bool, agentic_sdk::plugins::sandbox::ResourceError> {
        // Placeholder: would check actual CPU usage
        let _ = current_usage;
        Ok(true)
    }

    async fn check_memory_limit(&self, current_usage: usize) -> Result<bool, agentic_sdk::plugins::sandbox::ResourceError> {
        // Convert MB to bytes
        let max_bytes = self.max_memory_mb * 1024 * 1024;
        if current_usage > max_bytes as usize {
            return Err(agentic_sdk::plugins::sandbox::ResourceError::MemoryLimitExceeded);
        }
        Ok(true)
    }

    async fn enforce_timeout(
        &self,
        started_at: std::time::Instant,
        timeout: std::time::Duration,
    ) -> Result<(), agentic_sdk::plugins::sandbox::ResourceError> {
        let elapsed = started_at.elapsed();
        if elapsed > timeout {
            return Err(agentic_sdk::plugins::sandbox::ResourceError::TimeoutExceeded);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_whitelist_policy_creation() {
        let config = serde_json::json!({
            "allowed_paths": ["/tmp"],
            "allowed_commands": ["ls", "pwd"],
            "allowed_network": []
        });

        let policy = WhitelistPolicy::new(&config);
        assert_eq!(policy.plugin_id(), "whitelist-policy");
        assert_eq!(policy.version(), "0.1.0");
    }

    #[tokio::test]
    async fn test_whitelist_policy_validate_command_allowed() {
        let config = serde_json::json!({
            "allowed_paths": ["/tmp"],
            "allowed_commands": ["ls", "pwd", "echo"],
            "allowed_network": []
        });

        let policy = WhitelistPolicy::new(&config);
        let result = policy.validate_command("ls", "/tmp").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_whitelist_policy_validate_command_denied() {
        let config = serde_json::json!({
            "allowed_paths": ["/tmp"],
            "allowed_commands": ["ls", "pwd"],
            "allowed_network": []
        });

        let policy = WhitelistPolicy::new(&config);
        let result = policy.validate_command("rm", "/tmp").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_whitelist_policy_validate_file_access_allowed() {
        let config = serde_json::json!({
            "allowed_paths": ["/tmp", "/var/tmp"],
            "allowed_commands": [],
            "allowed_network": []
        });

        let policy = WhitelistPolicy::new(&config);
        let result = policy.validate_file_access("/tmp/test.txt", agentic_sdk::plugins::sandbox::FileOperation::Read).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_whitelist_policy_validate_file_access_denied() {
        let config = serde_json::json!({
            "allowed_paths": ["/tmp"],
            "allowed_commands": [],
            "allowed_network": []
        });

        let policy = WhitelistPolicy::new(&config);
        let result = policy.validate_file_access("/etc/passwd", agentic_sdk::plugins::sandbox::FileOperation::Read).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_whitelist_policy_validate_network_denied() {
        let config = serde_json::json!({
            "allowed_paths": [],
            "allowed_commands": [],
            "allowed_network": []
        });

        let policy = WhitelistPolicy::new(&config);
        let result = policy.validate_network_access("http://example.com").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unix_resource_limiter_creation() {
        let config = serde_json::json!({
            "max_execution_time_secs": 60,
            "max_memory_mb": 512,
            "max_file_size_mb": 10
        });

        let limiter = UnixResourceLimiter::new(&config);
        assert_eq!(limiter.plugin_id(), "unix-resource-limiter");
        assert_eq!(limiter.version(), "0.1.0");
    }

    #[tokio::test]
    async fn test_unix_resource_limiter_check_cpu_limit() {
        let config = serde_json::json!({
            "max_execution_time_secs": 60,
            "max_memory_mb": 512,
            "max_file_size_mb": 10
        });

        let limiter = UnixResourceLimiter::new(&config);
        let result = limiter.check_cpu_limit(Duration::from_secs(10)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unix_resource_limiter_check_memory_limit_within() {
        let config = serde_json::json!({
            "max_execution_time_secs": 60,
            "max_memory_mb": 512,
            "max_file_size_mb": 10
        });

        let limiter = UnixResourceLimiter::new(&config);
        let result = limiter.check_memory_limit(100 * 1024 * 1024).await; // 100 MB
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unix_resource_limiter_check_memory_limit_exceeded() {
        let config = serde_json::json!({
            "max_execution_time_secs": 60,
            "max_memory_mb": 512,
            "max_file_size_mb": 10
        });

        let limiter = UnixResourceLimiter::new(&config);
        let result = limiter.check_memory_limit(1024 * 1024 * 1024).await; // 1 GB
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unix_resource_limiter_enforce_timeout_within() {
        let config = serde_json::json!({
            "max_execution_time_secs": 60,
            "max_memory_mb": 512,
            "max_file_size_mb": 10
        });

        let limiter = UnixResourceLimiter::new(&config);
        let started_at = std::time::Instant::now();
        let timeout = Duration::from_secs(10);

        tokio::time::sleep(Duration::from_millis(100)).await;
        let result = limiter.enforce_timeout(started_at, timeout).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unix_resource_limiter_enforce_timeout_exceeded() {
        let config = serde_json::json!({
            "max_execution_time_secs": 60,
            "max_memory_mb": 512,
            "max_file_size_mb": 10
        });

        let limiter = UnixResourceLimiter::new(&config);
        let started_at = std::time::Instant::now();
        let timeout = Duration::from_millis(10);

        tokio::time::sleep(Duration::from_millis(50)).await;
        let result = limiter.enforce_timeout(started_at, timeout).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_plugin_config_deserialization() {
        let config_json = r#"{
            "tools": [
                {
                    "name": "builtin-file",
                    "config": {"max_file_size_mb": 10}
                }
            ],
            "security": [
                {
                    "name": "whitelist",
                    "config": {
                        "allowed_paths": ["/tmp"],
                        "allowed_commands": ["ls"],
                        "allowed_network": []
                    }
                }
            ],
            "resources": [
                {
                    "name": "unix",
                    "config": {
                        "max_execution_time_secs": 60,
                        "max_memory_mb": 512,
                        "max_file_size_mb": 10
                    }
                }
            ]
        }"#;

        let config: PluginConfig = serde_json::from_str(config_json).unwrap();
        assert_eq!(config.tools.len(), 1);
        assert_eq!(config.security.len(), 1);
        assert_eq!(config.resources.len(), 1);
        assert_eq!(config.tools[0].name, "builtin-file");
        assert_eq!(config.security[0].name, "whitelist");
        assert_eq!(config.resources[0].name, "unix");
    }

    #[tokio::test]
    async fn test_sandbox_core_creation() {
        let sandbox_root = std::env::temp_dir().join("sandbox-test").to_string_lossy().to_string();
        let sandbox = SandboxCore::new(sandbox_root.clone());

        assert_eq!(sandbox.sandbox_root(), sandbox_root);
    }

    #[tokio::test]
    async fn test_sandbox_core_register_tool() {
        let sandbox_root = std::env::temp_dir().join("sandbox-test-tools").to_string_lossy().to_string();
        let sandbox = SandboxCore::new(sandbox_root);

        // Create a mock tool for testing
        struct MockTool;
        #[async_trait::async_trait]
        impl Plugin for MockTool {
            fn plugin_id(&self) -> &'static str { "mock-tool" }
            fn version(&self) -> &'static str { "1.0.0" }
            fn description(&self) -> &'static str { "Mock tool for testing" }
            async fn initialize(&mut self, _config: &serde_json::Value) -> Result<(), agentic_sdk::plugin::PluginError> { Ok(()) }
            async fn health_check(&self) -> Result<bool, agentic_sdk::plugin::PluginError> { Ok(true) }
            async fn shutdown(&mut self) -> Result<(), agentic_sdk::plugin::PluginError> { Ok(()) }
        }

        #[async_trait::async_trait]
        impl Tool for MockTool {
            fn tool_name(&self) -> &'static str { "mock" }
            fn input_schema(&self) -> serde_json::Value {
                serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                })
            }
            async fn execute(&self, _params: serde_json::Value, _context: &SandboxContext) -> Result<serde_json::Value, agentic_sdk::plugins::sandbox::ToolError> {
                Ok(serde_json::json!({"result": "mock result"}))
            }
        }

        let tool = Arc::new(MockTool);
        sandbox.register_tool(tool).await;

        // Verify tool is registered
        let tools = sandbox.tools.read().await;
        assert!(tools.contains_key("mock"));
    }

    #[tokio::test]
    async fn test_sandbox_core_set_security() {
        let sandbox_root = std::env::temp_dir().join("sandbox-test-security").to_string_lossy().to_string();
        let sandbox = SandboxCore::new(sandbox_root);

        let config = serde_json::json!({
            "allowed_paths": ["/tmp"],
            "allowed_commands": ["ls"],
            "allowed_network": []
        });

        let security = Arc::new(WhitelistPolicy::new(&config));
        sandbox.set_security(security.clone()).await;

        let retrieved_security = sandbox.security().await;
        assert!(retrieved_security.is_some());
    }

    #[tokio::test]
    async fn test_sandbox_core_set_resource_limiter() {
        let sandbox_root = std::env::temp_dir().join("sandbox-test-resources").to_string_lossy().to_string();
        let sandbox = SandboxCore::new(sandbox_root);

        let config = serde_json::json!({
            "max_execution_time_secs": 60,
            "max_memory_mb": 512,
            "max_file_size_mb": 10
        });

        let limiter = Arc::new(UnixResourceLimiter::new(&config));
        sandbox.set_resource_limiter(limiter.clone()).await;

        let retrieved_limiter = sandbox.resource_limiter().await;
        assert!(retrieved_limiter.is_some());
    }

    #[tokio::test]
    async fn test_sandbox_core_execute_tool_without_security() {
        let sandbox_root = std::env::temp_dir().join("sandbox-test-exec-no-sec").to_string_lossy().to_string();
        let sandbox = SandboxCore::new(sandbox_root);

        // Should fail because no tool is registered
        let result = sandbox.execute_tool("nonexistent", serde_json::json!({})).await;
        assert!(result.is_err());
    }
}
