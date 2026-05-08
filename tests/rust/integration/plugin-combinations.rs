//! Integration tests for plugin combinations.
//!
//! These tests verify that different plugin combinations work correctly
//! together within each module and across the entire system.

use agentic_sdk::{
    message_types::{ChatMessage, ContextPackage, TaskComplete, TaskSubmitted, TaskEnriched},
    plugin::{Plugin, PluginError},
    plugin_registry::PluginRegistry,
    plugins::{
        context::{EnrichmentStrategy, MemoryBackend, StorageBackend},
        interface::{InputMethod, OutputFormatter},
        orchestrator::{ExecutionStrategy, ResultSynthesizer, TaskPlanner},
        sandbox::{ResourceLimiter, SecurityPolicy, Tool},
    },
    ContextPackageBuilder, TaskSubmittedBuilder,
};
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;
use tokio;

// ============================================================================
// Mock Plugin Implementations for Testing
// ============================================================================

struct MockStorageBackend;

#[async_trait::async_trait]
impl Plugin for MockStorageBackend {
    fn plugin_id(&self) -> &'static str {
        "storage-mock"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Mock storage backend for testing"
    }

    async fn initialize(&mut self, _config: &serde_json::Value) -> Result<(), PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl StorageBackend for MockStorageBackend {
    async fn ensure_session<'a>(&'a self, _session_id: &'a str) -> Result<(), agentic_sdk::plugins::context::StorageError> {
        Ok(())
    }

    async fn store_message<'a>(
        &'a self,
        _session_id: &'a str,
        role: &'a str,
        content: &'a str,
    ) -> Result<(), agentic_sdk::plugins::context::StorageError> {
        // Simulate storing a message
        println!("Stored message: role={}, content={}", role, content);
        Ok(())
    }

    async fn load_session_history<'a>(
        &'a self,
        _session_id: &'a str,
        _limit: usize,
    ) -> Result<Vec<ChatMessage>, agentic_sdk::plugins::context::StorageError> {
        Ok(vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
                timestamp: Utc::now().timestamp(),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "Hi there!".to_string(),
                timestamp: Utc::now().timestamp(),
            },
        ])
    }
}

struct MockMemoryBackend;

#[async_trait::async_trait]
impl Plugin for MockMemoryBackend {
    fn plugin_id(&self) -> &'static str {
        "memory-mock"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Mock memory backend for testing"
    }

    async fn initialize(&mut self, _config: &serde_json::Value) -> Result<(), PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl MemoryBackend for MockMemoryBackend {
    async fn search(
        &self,
        _query: &str,
        _session_id: &str,
        _limit: usize,
    ) -> Result<Vec<agentic_sdk::message_types::MemoryChunk>, agentic_sdk::plugins::context::MemoryError> {
        Ok(vec![agentic_sdk::message_types::MemoryChunk {
            id: "chunk-1".to_string(),
            content: "Mock memory content".to_string(),
            source: "test".to_string(),
            relevance_score: 0.9,
        }])
    }

    async fn persist_chunk(
        &self,
        _session_id: &str,
        _content: &str,
        _source: &str,
    ) -> Result<(), agentic_sdk::plugins::context::MemoryError> {
        Ok(())
    }

    async fn load_chunks(
        &self,
        _session_id: &str,
        _limit: usize,
    ) -> Result<Vec<agentic_sdk::message_types::MemoryChunk>, agentic_sdk::plugins::context::MemoryError> {
        Ok(vec![])
    }
}

struct MockEnrichmentStrategy;

#[async_trait::async_trait]
impl Plugin for MockEnrichmentStrategy {
    fn plugin_id(&self) -> &'static str {
        "enrichment-mock"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Mock enrichment strategy for testing"
    }

    async fn initialize(&mut self, _config: &serde_json::Value) -> Result<(), PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl EnrichmentStrategy for MockEnrichmentStrategy {
    async fn enrich(
        &self,
        _task: &TaskSubmitted,
        base_context: &ContextPackage,
    ) -> Result<ContextPackage, agentic_sdk::plugins::context::EnrichmentError> {
        // Return enriched context with additional data
        let mut enriched = base_context.clone();
        enriched.safe_env.insert("TEST_VAR".to_string(), "test_value".to_string());
        Ok(enriched)
    }

    async fn on_complete(
        &self,
        _session_id: &str,
        _result: &TaskComplete,
    ) -> Result<(), agentic_sdk::plugins::context::EnrichmentError> {
        Ok(())
    }
}

struct MockTaskPlanner;

#[async_trait::async_trait]
impl Plugin for MockTaskPlanner {
    fn plugin_id(&self) -> &'static str {
        "planner-mock"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Mock task planner for testing"
    }

    async fn initialize(&mut self, _config: &serde_json::Value) -> Result<(), PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl TaskPlanner for MockTaskPlanner {
    async fn decompose(
        &self,
        _task: &TaskEnriched,
    ) -> Result<Vec<agentic_sdk::plugins::orchestrator::TaskDescription>, agentic_sdk::plugins::orchestrator::PlanningError> {
        Ok(vec![
            agentic_sdk::plugins::orchestrator::TaskDescription {
                description: "Subtask 1".to_string(),
                dependencies: vec![],
                metadata: json!({}),
            },
            agentic_sdk::plugins::orchestrator::TaskDescription {
                description: "Subtask 2".to_string(),
                dependencies: vec![],
                metadata: json!({}),
            },
        ])
    }
}

struct MockExecutionStrategy;

#[async_trait::async_trait]
impl Plugin for MockExecutionStrategy {
    fn plugin_id(&self) -> &'static str {
        "execution-mock"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Mock execution strategy for testing"
    }

    async fn initialize(&mut self, _config: &serde_json::Value) -> Result<(), PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl ExecutionStrategy for MockExecutionStrategy {
    async fn dispatch_jobs(
        &self,
        _jobs: Vec<agentic_sdk::message_types::AgentJob>,
    ) -> Result<Vec<String>, agentic_sdk::plugins::orchestrator::ExecutionError> {
        Ok(vec!["job-1".to_string(), "job-2".to_string()])
    }

    async fn collect_results(
        &self,
        _correlation_parent: &str,
        _expected_count: usize,
    ) -> Result<Vec<agentic_sdk::message_types::AgentResult>, agentic_sdk::plugins::orchestrator::ExecutionError> {
        Ok(vec![])
    }
}

struct MockResultSynthesizer;

#[async_trait::async_trait]
impl Plugin for MockResultSynthesizer {
    fn plugin_id(&self) -> &'static str {
        "synthesizer-mock"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Mock result synthesizer for testing"
    }

    async fn initialize(&mut self, _config: &serde_json::Value) -> Result<(), PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl ResultSynthesizer for MockResultSynthesizer {
    async fn synthesize(
        &self,
        _results: Vec<agentic_sdk::message_types::AgentResult>,
        _original_task: &TaskEnriched,
    ) -> Result<TaskComplete, agentic_sdk::plugins::orchestrator::SynthesisError> {
        Ok(TaskComplete {
            session_id: "test-session".to_string(),
            correlation_id: "test-correlation".to_string(),
            result: "Synthesized result".to_string(),
            side_effects: vec![],
            warnings: vec![],
            completed_at: Utc::now().timestamp(),
        })
    }
}

struct MockTool;

#[async_trait::async_trait]
impl Plugin for MockTool {
    fn plugin_id(&self) -> &'static str {
        "tool-mock"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Mock tool for testing"
    }

    async fn initialize(&mut self, _config: &serde_json::Value) -> Result<(), PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl Tool for MockTool {
    fn tool_name(&self) -> &'static str {
        "mock"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "input": {"type": "string"}
            },
            "required": ["input"]
        })
    }

    async fn execute(
        &self,
        _params: serde_json::Value,
        _sandbox_context: &agentic_sdk::plugins::sandbox::SandboxContext,
    ) -> Result<serde_json::Value, agentic_sdk::plugins::sandbox::ToolError> {
        Ok(json!({ "result": "mock output" }))
    }
}

struct MockSecurityPolicy;

#[async_trait::async_trait]
impl Plugin for MockSecurityPolicy {
    fn plugin_id(&self) -> &'static str {
        "security-mock"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Mock security policy for testing"
    }

    async fn initialize(&mut self, _config: &serde_json::Value) -> Result<(), PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl SecurityPolicy for MockSecurityPolicy {
    async fn validate_command(
        &self,
        _command: &str,
        _working_dir: &str,
    ) -> Result<bool, agentic_sdk::plugins::sandbox::SecurityError> {
        Ok(true)
    }

    async fn validate_file_access(
        &self,
        _path: &str,
        _operation: agentic_sdk::plugins::sandbox::FileOperation,
    ) -> Result<bool, agentic_sdk::plugins::sandbox::SecurityError> {
        Ok(true)
    }

    async fn validate_network_access(&self, _url: &str) -> Result<bool, agentic_sdk::plugins::sandbox::SecurityError> {
        Ok(true)
    }
}

struct MockResourceLimiter;

#[async_trait::async_trait]
impl Plugin for MockResourceLimiter {
    fn plugin_id(&self) -> &'static str {
        "limits-mock"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Mock resource limiter for testing"
    }

    async fn initialize(&mut self, _config: &serde_json::Value) -> Result<(), PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl ResourceLimiter for MockResourceLimiter {
    async fn check_cpu_limit(
        &self,
        _current_usage: std::time::Duration,
    ) -> Result<bool, agentic_sdk::plugins::sandbox::ResourceError> {
        Ok(true)
    }

    async fn check_memory_limit(&self, _current_usage: usize) -> Result<bool, agentic_sdk::plugins::sandbox::ResourceError> {
        Ok(true)
    }

    async fn enforce_timeout(
        &self,
        _started_at: std::time::Instant,
        _timeout: std::time::Duration,
    ) -> Result<(), agentic_sdk::plugins::sandbox::ResourceError> {
        Ok(())
    }
}

struct MockInputMethod;

#[async_trait::async_trait]
impl Plugin for MockInputMethod {
    fn plugin_id(&self) -> &'static str {
        "input-mock"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Mock input method for testing"
    }

    async fn initialize(&mut self, _config: &serde_json::Value) -> Result<(), PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl InputMethod for MockInputMethod {
    async fn read_input(&self) -> Result<TaskSubmitted, agentic_sdk::plugins::interface::InputError> {
        Ok(TaskSubmittedBuilder::default()
            .session_id("test-session")
            .user_input("Test input")
            .submitted_at(Utc::now().timestamp())
            .build()
            .unwrap())
    }
}

struct MockOutputFormatter;

#[async_trait::async_trait]
impl Plugin for MockOutputFormatter {
    fn plugin_id(&self) -> &'static str {
        "output-mock"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Mock output formatter for testing"
    }

    async fn initialize(&mut self, _config: &serde_json::Value) -> Result<(), PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl OutputFormatter for MockOutputFormatter {
    async fn format_result(&self, result: &TaskComplete) -> Result<String, agentic_sdk::plugins::interface::FormatError> {
        Ok(format!("Formatted: {}", result.result))
    }
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_context_plugins_combination() {
    let registry = PluginRegistry::new();

    // Register context plugins
    let storage = Box::new(MockStorageBackend);
    let memory = Box::new(MockMemoryBackend);
    let enrichment = Box::new(MockEnrichmentStrategy);

    registry.register(storage).await.unwrap();
    registry.register(memory).await.unwrap();
    registry.register(enrichment).await.unwrap();

    // Verify all plugins are registered
    assert!(registry.is_registered("storage-mock").await);
    assert!(registry.is_registered("memory-mock").await);
    assert!(registry.is_registered("enrichment-mock").await);
    assert_eq!(registry.count().await, 3);

    // Test plugin retrieval
    let storage_plugin: Arc<MockStorageBackend> = registry.get("storage-mock").await.unwrap();
    assert_eq!(storage_plugin.plugin_id(), "storage-mock");

    // Test storage backend operations
    storage_plugin.ensure_session("test-session").await.unwrap();
    storage_plugin
        .store_message("test-session", "user", "Hello")
        .await
        .unwrap();
    let history = storage_plugin.load_session_history("test-session", 10).await.unwrap();
    assert_eq!(history.len(), 2);

    // Test memory backend operations
    let memory_plugin: Arc<MockMemoryBackend> = registry.get("memory-mock").await.unwrap();
    let chunks = memory_plugin.search("test query", "test-session", 5).await.unwrap();
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].content, "Mock memory content");

    // Test enrichment strategy
    let enrichment_plugin: Arc<MockEnrichmentStrategy> = registry.get("enrichment-mock").await.unwrap();
    let task = TaskSubmittedBuilder::default()
        .session_id("test-session")
        .user_input("Test")
        .submitted_at(Utc::now().timestamp())
        .build()
        .unwrap();
    let base_context = ContextPackageBuilder::default().build().unwrap();
    let enriched = enrichment_plugin.enrich(&task, &base_context).await.unwrap();
    assert!(enriched.safe_env.contains_key("TEST_VAR"));
}

#[tokio::test]
async fn test_orchestrator_plugins_combination() {
    let registry = PluginRegistry::new();

    // Register orchestrator plugins
    let planner = Box::new(MockTaskPlanner);
    let execution = Box::new(MockExecutionStrategy);
    let synthesizer = Box::new(MockResultSynthesizer);

    registry.register(planner).await.unwrap();
    registry.register(execution).await.unwrap();
    registry.register(synthesizer).await.unwrap();

    // Verify all plugins are registered
    assert!(registry.is_registered("planner-mock").await);
    assert!(registry.is_registered("execution-mock").await);
    assert!(registry.is_registered("synthesizer-mock").await);
    assert_eq!(registry.count().await, 3);

    // Test task planner
    let planner_plugin: Arc<MockTaskPlanner> = registry.get("planner-mock").await.unwrap();
    let task = TaskEnriched {
        session_id: "test-session".to_string(),
        correlation_id: "test-correlation".to_string(),
        user_input: "Test task".to_string(),
        context: ContextPackageBuilder::default().build().unwrap(),
        inferred_constraints: vec![],
        enriched_at: Utc::now().timestamp(),
    };
    let subtasks = planner_plugin.decompose(&task).await.unwrap();
    assert_eq!(subtasks.len(), 2);

    // Test execution strategy
    let execution_plugin: Arc<MockExecutionStrategy> = registry.get("execution-mock").await.unwrap();
    let job_ids = execution_plugin.dispatch_jobs(vec![]).await.unwrap();
    assert_eq!(job_ids.len(), 2);

    // Test result synthesizer
    let synthesizer_plugin: Arc<MockResultSynthesizer> = registry.get("synthesizer-mock").await.unwrap();
    let result = synthesizer_plugin.synthesize(vec![], &task).await.unwrap();
    assert_eq!(result.result, "Synthesized result");
}

#[tokio::test]
async fn test_sandbox_plugins_combination() {
    let registry = PluginRegistry::new();

    // Register sandbox plugins
    let tool = Box::new(MockTool);
    let security = Box::new(MockSecurityPolicy);
    let limits = Box::new(MockResourceLimiter);

    registry.register(tool).await.unwrap();
    registry.register(security).await.unwrap();
    registry.register(limits).await.unwrap();

    // Verify all plugins are registered
    assert!(registry.is_registered("tool-mock").await);
    assert!(registry.is_registered("security-mock").await);
    assert!(registry.is_registered("limits-mock").await);
    assert_eq!(registry.count().await, 3);

    // Test tool execution
    let tool_plugin: Arc<MockTool> = registry.get("tool-mock").await.unwrap();
    assert_eq!(tool_plugin.tool_name(), "mock");
    let schema = tool_plugin.input_schema();
    assert!(schema.is_object());
    let context = agentic_sdk::plugins::sandbox::SandboxContext {
        working_dir: "/tmp".to_string(),
        environment: vec![],
        allowed_paths: vec![],
    };
    let result = tool_plugin.execute(json!({"input": "test"}), &context).await.unwrap();
    assert_eq!(result["result"], "mock output");

    // Test security policy
    let security_plugin: Arc<MockSecurityPolicy> = registry.get("security-mock").await.unwrap();
    assert!(security_plugin.validate_command("ls", "/tmp").await.unwrap());
    assert!(security_plugin.validate_file_access("/tmp/test.txt", agentic_sdk::plugins::sandbox::FileOperation::Read).await.unwrap());
    assert!(security_plugin.validate_network_access("https://example.com").await.unwrap());

    // Test resource limiter
    let limits_plugin: Arc<MockResourceLimiter> = registry.get("limits-mock").await.unwrap();
    assert!(limits_plugin.check_cpu_limit(std::time::Duration::from_secs(10)).await.unwrap());
    assert!(limits_plugin.check_memory_limit(1024).await.unwrap());
    limits_plugin
        .enforce_timeout(std::time::Instant::now(), std::time::Duration::from_secs(30))
        .await
        .unwrap();
}

#[tokio::test]
async fn test_interface_plugins_combination() {
    let registry = PluginRegistry::new();

    // Register interface plugins
    let input = Box::new(MockInputMethod);
    let output = Box::new(MockOutputFormatter);

    registry.register(input).await.unwrap();
    registry.register(output).await.unwrap();

    // Verify all plugins are registered
    assert!(registry.is_registered("input-mock").await);
    assert!(registry.is_registered("output-mock").await);
    assert_eq!(registry.count().await, 2);

    // Test input method
    let input_plugin: Arc<MockInputMethod> = registry.get("input-mock").await.unwrap();
    let task = input_plugin.read_input().await.unwrap();
    assert_eq!(task.user_input, "Test input");

    // Test output formatter
    let output_plugin: Arc<MockOutputFormatter> = registry.get("output-mock").await.unwrap();
    let complete = TaskComplete {
        session_id: "test-session".to_string(),
        correlation_id: "test-correlation".to_string(),
        result: "Test result".to_string(),
        side_effects: vec![],
        warnings: vec![],
        completed_at: Utc::now().timestamp(),
    };
    let formatted = output_plugin.format_result(&complete).await.unwrap();
    assert_eq!(formatted, "Formatted: Test result");
}

#[tokio::test]
async fn test_full_system_integration() {
    let registry = PluginRegistry::new();

    // Register all plugins from all modules
    registry.register(Box::new(MockStorageBackend)).await.unwrap();
    registry.register(Box::new(MockMemoryBackend)).await.unwrap();
    registry.register(Box::new(MockEnrichmentStrategy)).await.unwrap();
    registry.register(Box::new(MockTaskPlanner)).await.unwrap();
    registry.register(Box::new(MockExecutionStrategy)).await.unwrap();
    registry.register(Box::new(MockResultSynthesizer)).await.unwrap();
    registry.register(Box::new(MockTool)).await.unwrap();
    registry.register(Box::new(MockSecurityPolicy)).await.unwrap();
    registry.register(Box::new(MockResourceLimiter)).await.unwrap();
    registry.register(Box::new(MockInputMethod)).await.unwrap();
    registry.register(Box::new(MockOutputFormatter)).await.unwrap();

    // Verify all plugins are registered
    assert_eq!(registry.count().await, 11);

    // List all plugins
    let plugin_ids = registry.list_plugins().await;
    assert_eq!(plugin_ids.len(), 11);
    assert!(plugin_ids.contains(&"storage-mock".to_string()));
    assert!(plugin_ids.contains(&"memory-mock".to_string()));
    assert!(plugin_ids.contains(&"enrichment-mock".to_string()));
    assert!(plugin_ids.contains(&"planner-mock".to_string()));
    assert!(plugin_ids.contains(&"execution-mock".to_string()));
    assert!(plugin_ids.contains(&"synthesizer-mock".to_string()));
    assert!(plugin_ids.contains(&"tool-mock".to_string()));
    assert!(plugin_ids.contains(&"security-mock".to_string()));
    assert!(plugin_ids.contains(&"limits-mock".to_string()));
    assert!(plugin_ids.contains(&"input-mock".to_string()));
    assert!(plugin_ids.contains(&"output-mock".to_string()));

    // Test health checks for all plugins
    for plugin_id in &plugin_ids {
        let plugin = registry.get::<dyn Plugin>(plugin_id).await.unwrap();
        let healthy = plugin.health_check().await.unwrap();
        assert!(healthy, "Plugin {} should be healthy", plugin_id);
    }

    // Test cleanup
    registry.clear().await;
    assert_eq!(registry.count().await, 0);
}
