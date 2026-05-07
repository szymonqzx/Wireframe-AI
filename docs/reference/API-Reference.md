# Wireframe-AI API Reference

Complete API reference for the Wireframe-AI SDK.

## Table of Contents

- [Core Plugin Trait](#core-plugin-trait)
- [Context Plugin Traits](#context-plugin-traits)
- [Orchestrator Plugin Traits](#orchestrator-plugin-traits)
- [Sandbox Plugin Traits](#sandbox-plugin-traits)
- [Interface Plugin Traits](#interface-plugin-traits)
- [Message Types](#message-types)
- [PluginRegistry API](#pluginregistry-api)
- [Envelope API](#envelope-api)

## Core Plugin Trait

All plugins implement the base `Plugin` trait.

### Trait Definition

```rust
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Plugin: Send + Sync + Any {
    /// Returns the unique plugin ID.
    fn plugin_id(&self) -> &'static str;

    /// Returns the version of this plugin.
    fn version(&self) -> &'static str;

    /// Returns a description of this plugin.
    fn description(&self) -> &'static str;

    /// Initializes the plugin with the provided configuration.
    async fn initialize(&mut self, config: &Value) -> Result<(), PluginError>;

    /// Performs a health check on the plugin.
    async fn health_check(&self) -> Result<bool, PluginError>;

    /// Shuts down the plugin and cleans up resources.
    async fn shutdown(&mut self) -> Result<(), PluginError>;
}
```

### Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `plugin_id()` | `&'static str` | Unique identifier for the plugin |
| `version()` | `&'static str` | Semantic version of the plugin |
| `description()` | `&'static str` | Human-readable description |
| `initialize()` | `Result<(), PluginError>` | Initialize plugin with configuration |
| `health_check()` | `Result<bool, PluginError>` | Check if plugin is healthy |
| `shutdown()` | `Result<(), PluginError>` | Cleanup plugin resources |

### PluginError

```rust
pub enum PluginError {
    InitializationFailed(String),
    ConfigurationError(String),
    ExecutionError(String),
    NotFound(String),
    IncompatibleVersion { expected: String, got: String },
    HealthCheckFailed(String),
    ShutdownFailed(String),
}
```

## Context Plugin Traits

### StorageBackend

Persist sessions and messages to a database.

```rust
use agentic_sdk::plugins::context::{StorageBackend, StorageError};
use agentic_sdk::message_types::ChatMessage;

#[async_trait]
pub trait StorageBackend: Plugin {
    async fn ensure_session(&self, session_id: &str) -> Result<(), StorageError>;
    async fn store_message(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> Result<(), StorageError>;
    async fn load_session_history(
        &self,
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<ChatMessage>, StorageError>;
}
```

#### Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `ensure_session()` | `session_id: &str` | `Result<(), StorageError>` | Ensure session exists in storage |
| `store_message()` | `session_id, role, content` | `Result<(), StorageError>` | Store a message in a session |
| `load_session_history()` | `session_id, limit` | `Result<Vec<ChatMessage>>` | Load session history with limit |

#### StorageError

```rust
pub enum StorageError {
    DatabaseError(String),
    SessionNotFound(String),
    SerializationError(String),
}
```

### MemoryBackend

Search and persist memory chunks.

```rust
use agentic_sdk::plugins::context::{MemoryBackend, MemoryError};
use agentic_sdk::message_types::MemoryChunk;

#[async_trait]
pub trait MemoryBackend: Plugin {
    async fn search(
        &self,
        query: &str,
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<MemoryChunk>, MemoryError>;
    
    async fn persist_chunk(
        &self,
        session_id: &str,
        content: &str,
        source: &str,
    ) -> Result<(), MemoryError>;
    
    async fn load_chunks(
        &self,
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<MemoryChunk>, MemoryError>;
}
```

#### Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `search()` | `query, session_id, limit` | `Result<Vec<MemoryChunk>>` | Search memory for relevant chunks |
| `persist_chunk()` | `session_id, content, source` | `Result<(), MemoryError>` | Persist a memory chunk |
| `load_chunks()` | `session_id, limit` | `Result<Vec<MemoryChunk>>` | Load memory chunks for a session |

#### MemoryError

```rust
pub enum MemoryError {
    SearchFailed(String),
    PersistenceFailed(String),
    EmbeddingFailed(String),
    VectorDbError(String),
}
```

### EnrichmentStrategy

Add context to tasks (memory retrieval, file context, environment variables).

```rust
use agentic_sdk::plugins::context::{EnrichmentStrategy, EnrichmentError};
use agentic_sdk::message_types::{TaskSubmitted, TaskComplete, ContextPackage};

#[async_trait]
pub trait EnrichmentStrategy: Plugin {
    async fn enrich(
        &self,
        task: &TaskSubmitted,
        base_context: &ContextPackage,
    ) -> Result<ContextPackage, EnrichmentError>;
    
    async fn on_complete(
        &self,
        session_id: &str,
        result: &TaskComplete,
    ) -> Result<(), EnrichmentError>;
}
```

#### Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `enrich()` | `task, base_context` | `Result<ContextPackage>` | Enrich a task with additional context |
| `on_complete()` | `session_id, result` | `Result<(), EnrichmentError>` | Post-processing when task completes |

#### EnrichmentError

```rust
pub enum EnrichmentError {
    MemoryRetrievalFailed(String),
    FileContextFailed(String),
    EnvironmentContextFailed(String),
}
```

## Orchestrator Plugin Traits

### TaskPlanner

Decompose tasks into subtasks.

```rust
use agentic_sdk::plugins::orchestrator::{TaskPlanner, PlanningError, TaskDescription};
use agentic_sdk::message_types::TaskEnriched;

#[async_trait]
pub trait TaskPlanner: Plugin {
    async fn decompose(&self, task: &TaskEnriched) -> Result<Vec<TaskDescription>, PlanningError>;
}
```

#### Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `decompose()` | `task` | `Result<Vec<TaskDescription>>` | Decompose a task into subtasks |

#### TaskDescription

```rust
pub struct TaskDescription {
    pub description: String,
    pub dependencies: Vec<String>,
    pub metadata: Value,
}
```

#### PlanningError

```rust
pub enum PlanningError {
    DecompositionFailed(String),
    LlmPlanningFailed(String),
    InvalidTaskStructure(String),
}
```

### ExecutionStrategy

Dispatch jobs and collect results.

```rust
use agentic_sdk::plugins::orchestrator::{ExecutionStrategy, ExecutionError};
use agentic_sdk::message_types::{AgentJob, AgentResult};

#[async_trait]
pub trait ExecutionStrategy: Plugin {
    async fn dispatch_jobs(&self, jobs: Vec<AgentJob>) -> Result<Vec<String>, ExecutionError>;
    async fn collect_results(
        &self,
        correlation_parent: &str,
        expected_count: usize,
    ) -> Result<Vec<AgentResult>, ExecutionError>;
}
```

#### Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `dispatch_jobs()` | `jobs` | `Result<Vec<String>>` | Dispatch jobs to workers |
| `collect_results()` | `correlation_parent, expected_count` | `Result<Vec<AgentResult>>` | Collect results by correlation ID |

#### ExecutionError

```rust
pub enum ExecutionError {
    DispatchFailed(String),
    CollectionFailed(String),
    Timeout,
    CorrelationMismatch,
}
```

### ResultSynthesizer

Merge agent results into final answer.

```rust
use agentic_sdk::plugins::orchestrator::{ResultSynthesizer, SynthesisError};
use agentic_sdk::message_types::{AgentResult, TaskComplete, TaskEnriched};

#[async_trait]
pub trait ResultSynthesizer: Plugin {
    async fn synthesize(
        &self,
        results: Vec<AgentResult>,
        original_task: &TaskEnriched,
    ) -> Result<TaskComplete, SynthesisError>;
}
```

#### Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `synthesize()` | `results, original_task` | `Result<TaskComplete>` | Synthesize results into final task completion |

#### SynthesisError

```rust
pub enum SynthesisError {
    SynthesisFailed(String),
    LlmSynthesisFailed(String),
    NoResults,
}
```

## Sandbox Plugin Traits

### Tool

Executable tools (shell, file, git, HTTP, etc.).

```rust
use agentic_sdk::plugins::sandbox::{Tool, ToolError, SandboxContext};
use serde_json::Value;

#[async_trait]
pub trait Tool: Plugin {
    fn tool_name(&self) -> &'static str;
    fn input_schema(&self) -> Value;
    
    async fn execute(
        &self,
        params: Value,
        sandbox_context: &SandboxContext,
    ) -> Result<Value, ToolError>;
}
```

#### Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `tool_name()` | `&'static str` | Tool name (e.g., "shell", "file") |
| `input_schema()` | `Value` | JSON schema for tool input validation |
| `execute()` | `Result<Value, ToolError>` | Execute the tool with given parameters |

#### SandboxContext

```rust
pub struct SandboxContext {
    pub working_dir: String,
    pub environment: Vec<(String, String)>,
    pub allowed_paths: Vec<String>,
}
```

#### ToolError

```rust
pub enum ToolError {
    ExecutionFailed(String),
    InvalidParameters(String),
    PermissionDenied(String),
    Timeout,
}
```

### SecurityPolicy

Validate commands, file access, and network access.

```rust
use agentic_sdk::plugins::sandbox::{SecurityPolicy, SecurityError, FileOperation};

#[async_trait]
pub trait SecurityPolicy: Plugin {
    async fn validate_command(
        &self,
        command: &str,
        working_dir: &str,
    ) -> Result<bool, SecurityError>;
    
    async fn validate_file_access(
        &self,
        path: &str,
        operation: FileOperation,
    ) -> Result<bool, SecurityError>;
    
    async fn validate_network_access(&self, url: &str) -> Result<bool, SecurityError>;
}
```

#### Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `validate_command()` | `command, working_dir` | `Result<bool, SecurityError>` | Validate a shell command |
| `validate_file_access()` | `path, operation` | `Result<bool, SecurityError>` | Validate file system access |
| `validate_network_access()` | `url` | `Result<bool, SecurityError>` | Validate network access |

#### FileOperation

```rust
pub enum FileOperation {
    Read,
    Write,
    Delete,
    Execute,
}
```

#### SecurityError

```rust
pub enum SecurityError {
    CommandRejected(String),
    FileAccessDenied(String),
    NetworkAccessDenied(String),
    PolicyViolation(String),
}
```

### ResourceLimiter

Enforce CPU, memory, and timeout limits.

```rust
use agentic_sdk::plugins::sandbox::{ResourceLimiter, ResourceError};
use std::time::{Duration, Instant};

#[async_trait]
pub trait ResourceLimiter: Plugin {
    async fn check_cpu_limit(
        &self,
        current_usage: Duration,
    ) -> Result<bool, ResourceError>;
    
    async fn check_memory_limit(&self, current_usage: usize) -> Result<bool, ResourceError>;
    
    async fn enforce_timeout(
        &self,
        started_at: Instant,
        timeout: Duration,
    ) -> Result<(), ResourceError>;
}
```

#### Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `check_cpu_limit()` | `current_usage` | `Result<bool, ResourceError>` | Check CPU time limit |
| `check_memory_limit()` | `current_usage` | `Result<bool, ResourceError>` | Check memory limit |
| `enforce_timeout()` | `started_at, timeout` | `Result<(), ResourceError>` | Enforce timeout |

#### ResourceError

```rust
pub enum ResourceError {
    CpuLimitExceeded,
    MemoryLimitExceeded,
    TimeoutExceeded,
    MonitoringFailed(String),
}
```

## Interface Plugin Traits

### InputMethod

Receive user input (CLI, web, API, etc.).

```rust
use agentic_sdk::plugins::interface::{InputMethod, InputError};
use agentic_sdk::message_types::TaskSubmitted;

#[async_trait]
pub trait InputMethod: Plugin {
    async fn read_input(&self) -> Result<TaskSubmitted, InputError>;
}
```

#### Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `read_input()` | `Result<TaskSubmitted, InputError>` | Read input from the user |

#### InputError

```rust
pub enum InputError {
    ReadFailed(String),
    ParseFailed(String),
    Interrupted,
}
```

### OutputFormatter

Format results for display (markdown, JSON, HTML, etc.).

```rust
use agentic_sdk::plugins::interface::{OutputFormatter, FormatError};
use agentic_sdk::message_types::TaskComplete;

#[async_trait]
pub trait OutputFormatter: Plugin {
    async fn format_result(&self, result: &TaskComplete) -> Result<String, FormatError>;
}
```

#### Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `format_result()` | `result` | `Result<String, FormatError>` | Format a task completion result |

#### FormatError

```rust
pub enum FormatError {
    FormattingFailed(String),
    SerializationFailed(String),
}
```

### UIComponent

Interactive UI elements (progress bars, rich output, etc.).

```rust
use agentic_sdk::plugins::interface::{UIComponent, UIError, UIState};

#[async_trait]
pub trait UIComponent: Plugin {
    async fn render(&self, state: &UIState) -> Result<(), UIError>;
}
```

#### Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `render()` | `state` | `Result<(), UIError>` | Render the UI component with current state |

#### UIState

```rust
pub struct UIState {
    pub progress: f64,
    pub status: String,
    pub messages: Vec<String>,
}
```

#### UIError

```rust
pub enum UIError {
    RenderFailed(String),
    InvalidState(String),
}
```

## Message Types

### Task Flow Messages

#### TaskSubmitted

User request as submitted by the Interface module.

```rust
pub struct TaskSubmitted {
    pub session_id: String,
    pub user_input: String,
    pub submitted_at: i64,
}
```

#### TaskEnriched

Task after Context module enrichment.

```rust
pub struct TaskEnriched {
    pub session_id: String,
    pub correlation_id: String,
    pub user_input: String,
    pub context: ContextPackage,
    pub inferred_constraints: Vec<String>,
    pub enriched_at: i64,
}
```

#### TaskComplete

Final result returned to Interface for display to user.

```rust
pub struct TaskComplete {
    pub session_id: String,
    pub correlation_id: String,
    pub result: String,
    pub side_effects: Vec<SideEffect>,
    pub warnings: Vec<String>,
    pub completed_at: i64,
}
```

### Agent Job & Result

#### AgentJob

Self-contained unit of work dispatched to a Reasoning Adapter.

```rust
pub struct AgentJob {
    pub job_id: String,
    pub correlation_parent: String,
    pub task: TaskDescription,
    pub context: ContextPackage,
    pub available_tool_capabilities: Vec<ToolCapability>,
    pub constraints: ExecutionConstraints,
    pub model_config: ModelConfig,
    pub metadata: JobMetadata,
    pub adapter_hints: Option<Value>,
    pub schema_version: u32,
}
```

#### AgentResult

What an adapter sends back after finishing a job.

```rust
pub struct AgentResult {
    pub job_id: String,
    pub correlation_parent: String,
    pub output: AgentOutput,
    pub tool_invocations: Vec<ToolInvocation>,
    pub errors: Vec<AdapterError>,
    pub usage: Option<UsageMetrics>,
    pub completed_at: i64,
}
```

### Supporting Types

#### ContextPackage

```rust
pub struct ContextPackage {
    pub memory_chunks: Vec<MemoryChunk>,
    pub session_history: Vec<ChatMessage>,
    pub readonly_files: Vec<FileSnapshot>,
    pub safe_env: HashMap<String, String>,
    pub working_dir: PathBuf,
    pub max_context_tokens: usize,
}
```

#### MemoryChunk

```rust
pub struct MemoryChunk {
    pub id: String,
    pub content: String,
    pub source: String,
    pub relevance_score: f32,
}
```

#### ChatMessage

```rust
pub struct ChatMessage {
    pub role: String,  // "user", "assistant", "system"
    pub content: String,
    pub timestamp: i64,
}
```

#### FileSnapshot

```rust
pub struct FileSnapshot {
    pub path: PathBuf,
    pub content: String,
    pub size_bytes: usize,
    pub last_modified: i64,
}
```

#### ExecutionConstraints

```rust
pub struct ExecutionConstraints {
    pub timeout_seconds: Option<u32>,
    pub max_completion_tokens: Option<usize>,
    pub network_access: NetworkPolicy,
    pub filesystem_policy: FilesystemPolicy,
    pub allow_subprocess: bool,
    pub execution_mode: Option<String>,
}
```

#### NetworkPolicy

```rust
pub enum NetworkPolicy {
    None,
    OutboundOnly,
    Full,
}
```

#### FilesystemPolicy

```rust
pub enum FilesystemPolicy {
    Readonly,
    SandboxWritable,
    IsolatedVM,
}
```

#### ModelConfig

```rust
pub struct ModelConfig {
    pub provider: String,
    pub model_name: String,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub extra: HashMap<String, Value>,
}
```

#### AgentOutput

```rust
pub struct AgentOutput {
    pub text: Option<String>,
    pub structured: Option<Value>,
    pub files_written: Vec<PathBuf>,
    pub commands_run: Vec<String>,
}
```

#### ToolCapability

```rust
pub struct ToolCapability {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub required_credentials: Vec<CredentialRef>,
    pub rate_limit: Option<RateLimit>,
}
```

#### SideEffect

```rust
pub struct SideEffect {
    pub kind: String,
    pub description: String,
    pub path: Option<PathBuf>,
}
```

## PluginRegistry API

Universal plugin registry for dynamic plugin management.

```rust
pub struct PluginRegistry {
    plugins: Arc<RwLock<HashMap<String, Arc<dyn Plugin>>>>,
}
```

### Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `new()` | - | `PluginRegistry` | Create a new empty registry |
| `register()` | `plugin: Box<dyn Plugin>` | `Result<(), PluginError>` | Register a plugin |
| `get<T>()` | `plugin_id: &str` | `Result<Arc<T>, PluginError>` | Get a plugin by ID and downcast |
| `is_registered()` | `plugin_id: &str` | `bool` | Check if a plugin is registered |
| `count()` | - | `usize` | Get the count of registered plugins |
| `list_plugins()` | - | `Vec<String>` | List all registered plugin IDs |
| `unregister()` | `plugin_id: &str` | `Result<(), PluginError>` | Unregister a plugin |
| `clear()` | - | `()` | Clear all plugins |
| `load_from_config()` | `config_path: PathBuf` | `Result<(), ConfigError>` | Load plugins from configuration file |

### Example Usage

```rust
use agentic_sdk::{PluginRegistry, plugin::Plugin};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = PluginRegistry::new();
    
    // Register a plugin
    let plugin = Box::new(MyPlugin::new());
    registry.register(plugin).await?;
    
    // Check if registered
    if registry.is_registered("my-plugin").await {
        println!("Plugin is registered");
    }
    
    // List all plugins
    let plugins = registry.list_plugins().await;
    println!("Registered plugins: {:?}", plugins);
    
    Ok(())
}
```

## Envelope API

Universal message wrapper for NATS communication.

```rust
pub struct Envelope<T> {
    pub topic: String,
    pub correlation_id: String,
    pub timestamp: i64,
    pub payload: T,
}
```

### Methods

| Method | Parameters | Return Type | Description |
|--------|------------|-------------|-------------|
| `new()` | `topic, payload` | `Envelope<T>` | Create a new envelope |
| `reply()` | `topic, payload` | `Envelope<T>` | Create a reply envelope |
| `with_correlation()` | `correlation_id` | `Envelope<T>` | Set correlation ID |

### Example Usage

```rust
use agentic_sdk::Envelope;

let envelope = Envelope::new("task.submitted", task_data);
let reply = envelope.reply("task.enriched", enriched_data);
```

## Next Steps

- See `docs/Plugin-Development-Guide.md` for plugin development guide
- See `docs/Configuration-Examples.md` for configuration examples
- See SDK source code in `sdk/agentic-sdk/src/` for implementation details
