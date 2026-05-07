# Universal Modularization Plan for Wireframe AI

## Overview

This document outlines a comprehensive plan to modularize Wireframe AI using a universal plugin architecture. The goal is to make every module in the system extensible at runtime, allowing dynamic composition of strategies, backends, and implementations without recompilation.

## Core Design Philosophy

**Every module = Core Logic + Pluggable Strategies**

Each module has a small, stable core that handles:

- NATS communication
- Plugin orchestration
- Lifecycle management
- Error handling and logging

All domain-specific logic is extracted into plugins that can be:

- Swapped at runtime via configuration
- Developed by third parties
- Composed in pipelines
- A/B tested

## Current Architecture Analysis

### Current Module Structure

The system currently has these monolithic modules:

1. **Context Module** (`modules/context/src/main.rs`)
    - Session management (SQLite)
    - Message storage with FTS5 indexing
    - Memory search (FTS5 full-text)
    - Memory persistence
    - Context enrichment
    - Environment filtering
2. **Orchestrator Module** (`modules/orchestrator/src/main.rs`)
    - Task decomposition (N identical copies)
    - Parallel fan-out execution
    - Result collection by correlation ID
    - Result synthesis
3. **Sandbox Module** (`modules/sandbox/src/main.rs`)
    - MCP server implementation
    - Tool implementations (shell, file, git)
    - Security policies (whitelist, path validation)
    - Resource limits (rlimit)
4. **Interface Module** (`kernel/interface/src/main.rs`)
    - CLI input handling
    - Output formatting
5. **Reasoning Adapters** (Python)
    - AI model integration
    - Tool selection
    - Reasoning strategies

### Current Communication Pattern

All modules communicate via NATS with queue groups for load balancing:

- **task.submitted** → Context module (queue: task_handler)
- **task.enriched** → Orchestrator or Reasoning Adapter (queue: task_handler)
- **agent.job** → Reasoning Adapters (queue: agent_worker)
- **agent.result** → Orchestrator (queue: orchestrator_collector)

## Universal Plugin System Design

### 1. Base Plugin Trait (SDK)

```rust
// sdk/agentic-sdk/src/plugin.rs

use async_trait::async_trait;
use serde_json::Value;

/// Base trait that all plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync {
	/// Unique plugin identifier (e.g., "memory-rag", "planner-tree")
	fn plugin_id(&self) -> &'static str;

	/// Plugin version for compatibility checking
	fn version(&self) -> &'static str;

	/// Human-readable description
	fn description(&self) -> &'static str;

	/// Initialize plugin with configuration
	async fn initialize(&mut self, config: &Value) -> Result<(), PluginError>;

	/// Health check - return true if plugin is operational
	async fn health_check(&self) -> Result<bool, PluginError>;

	/// Cleanup resources before shutdown
	async fn shutdown(&mut self) -> Result<(), PluginError>;
}

/// Plugin error types
#[derive(thiserror::Error, Debug)]
pub enum PluginError {
	#[error("Initialization failed: {0}")]
	InitializationFailed(String),

	#[error("Configuration error: {0}")]
	ConfigurationError(String),

	#[error("Execution error: {0}")]
	ExecutionError(String),

	#[error("Plugin not found: {0}")]
	NotFound(String),

	#[error("Incompatible version: expected {expected}, got {got}")]
	IncompatibleVersion { expected: String, got: String },
}
```

### 2. Module-Specific Plugin Traits

#### A. Context Module Plugins

```rust
// sdk/agentic-sdk/src/plugins/context.rs

/// Storage backend for sessions and messages
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

/// Memory retrieval backend
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

/// Context enrichment strategy
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

#### B. Orchestrator Module Plugins

```rust
// sdk/agentic-sdk/src/plugins/orchestrator.rs

/// Task decomposition strategy
#[async_trait]
pub trait TaskPlanner: Plugin {
	async fn decompose(&self, task: &TaskEnriched)
		-> Result<Vec<TaskDescription>, PlanningError>;
}

/// Fan-out execution strategy
#[async_trait]
pub trait ExecutionStrategy: Plugin {
	async fn dispatch_jobs(&self, jobs: Vec<AgentJob>)
		-> Result<Vec<String>, ExecutionError>;
	async fn collect_results(
		&self,
		correlation_parent: &str,
		expected_count: usize,
	) -> Result<Vec<AgentResult>, ExecutionError>;
}

/// Result synthesis strategy
#[async_trait]
pub trait ResultSynthesizer: Plugin {
	async fn synthesize(
		&self,
		results: Vec<AgentResult>,
		original_task: &TaskEnriched,
	) -> Result<TaskComplete, SynthesisError>;
}
```

#### C. Sandbox Module Plugins

```rust
// sdk/agentic-sdk/src/plugins/sandbox.rs

/// Tool implementation
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

/// Security policy enforcement
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
	async fn validate_network_access(&self, url: &str)
		-> Result<bool, SecurityError>;
}

/// Resource limit enforcement
#[async_trait]
pub trait ResourceLimiter: Plugin {
	async fn check_cpu_limit(
		&self,
		current_usage: Duration,
	) -> Result<bool, ResourceError>;
	async fn check_memory_limit(
		&self,
		current_usage: usize,
	) -> Result<bool, ResourceError>;
	async fn enforce_timeout(
		&self,
		started_at: Instant,
		timeout: Duration,
	) -> Result<(), ResourceError>;
}
```

#### D. Interface Module Plugins

```rust
// sdk/agentic-sdk/src/plugins/interface.rs

/// Input method (CLI, web, API, etc.)
#[async_trait]
pub trait InputMethod: Plugin {
	async fn read_input(&self) -> Result<TaskSubmitted, InputError>;
}

/// Output formatter
#[async_trait]
pub trait OutputFormatter: Plugin {
	async fn format_result(&self, result: &TaskComplete)
		-> Result<String, FormatError>;
}

/// UI component (progress bars, rich output, etc.)
#[async_trait]
pub trait UIComponent: Plugin {
	async fn render(&self, state: &UIState) -> Result<(), UIError>;
}
```

#### E. Reasoning Adapter Plugins

```rust
// sdk/agentic-sdk/src/plugins/adapter.rs

/// AI model interface
#[async_trait]
pub trait AIModel: Plugin {
	async fn generate(&self, prompt: &str, context: &AgentJob)
		-> Result<String, ModelError>;
	async fn generate_structured(
		&self,
		prompt: &str,
		schema: &Value,
		context: &AgentJob,
	) -> Result<Value, ModelError>;
}

/// Tool selection strategy
#[async_trait]
pub trait ToolSelector: Plugin {
	async fn select_tools(
		&self,
		task: &str,
		available: &[ToolCapability],
	) -> Result<Vec<String>, SelectionError>;
}

/// Reasoning strategy (chain-of-thought, tree-of-thought, etc.)
#[async_trait]
pub trait ReasoningStrategy: Plugin {
	async fn reason(&self, context: &AgentJob)
		-> Result<AgentOutput, ReasoningError>;
}
```

### 3. Universal Plugin Registry

```rust
// sdk/agentic-sdk/src/plugin_registry.rs

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PluginRegistry {
	plugins: Arc<RwLock<HashMap<String, Box<dyn Plugin>>>>,
	module_configs: Arc<RwLock<HashMap<String, ModuleConfig>>>,
}

impl PluginRegistry {
	/// Create a new registry
	pub fn new() -> Self {
		Self {
			plugins: Arc::new(RwLock::new(HashMap::new())),
			module_configs: Arc::new(RwLock::new(HashMap::new())),
		}
	}

	/// Register a plugin
	pub async fn register(&self, plugin: Box<dyn Plugin>) -> Result<(), PluginError> {
		let id = plugin.plugin_id().to_string();
		let mut plugins = self.plugins.write().await;
		plugins.insert(id, plugin);
		Ok(())
	}

	/// Get a plugin by ID
	pub async fn get<T: Plugin + 'static>(&self, plugin_id: &str) -> Result<Arc<T>, PluginError> {
		let plugins = self.plugins.read().await;
		plugins
			.get(plugin_id)
			.and_then(|p| {
				// Downcast to specific type
				let any = p as &dyn std::any::Any;
				any.downcast_ref::<T>().map(|t| Arc::new(t.clone()))
			})
			.ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))
	}

	/// Load plugins from configuration file
	pub async fn load_from_config(&self, config_path: &str) -> Result<(), PluginError> {
		let config = std::fs::read_to_string(config_path)
			.map_err(|e| PluginError::ConfigurationError(e.to_string()))?;

		let config: Value = serde_json::from_str(&config)
			.map_err(|e| PluginError::ConfigurationError(e.to_string()))?;

		// Load plugins for each module
		if let Some(modules) = config.get("modules") {
			if let Some(modules_obj) = modules.as_object() {
				for (module_name, module_config) in modules_obj {
					self.load_module_plugins(module_name, module_config).await?;
				}
			}
		}

		Ok(())
	}

	/// Load plugins for a specific module
	async fn load_module_plugins(
		&self,
		module_name: &str,
		config: &Value,
	) -> Result<(), PluginError> {
		match module_name {
			"context" => self.load_context_plugins(config).await,
			"orchestrator" => self.load_orchestrator_plugins(config).await,
			"sandbox" => self.load_sandbox_plugins(config).await,
			"interface" => self.load_interface_plugins(config).await,
			"adapter" => self.load_adapter_plugins(config).await,
			_ => Err(PluginError::NotFound(format!("Unknown module: {}", module_name))),
		}
	}
}
```

### 4. Configuration Framework

```yaml
# wireframe-config.yaml

modules:
  context:
    enabled: true
    plugins:
      storage:
        plugin_id: "storage-sqlite"
        config:
          db_path: "./wireframe_ai.db"

      memory:
        plugin_id: "memory-rag"
        config:
          vector_db:
            type: "qdrant"
            url: "http://localhost:6333"
          embedding:
            model: "text-embedding-3-small"

      enrichment_pipeline:
        - plugin_id: "memory-rag"
          order: 1
          config:
            top_k: 20
        - plugin_id: "context-files"
          order: 2
          config:
            max_files: 10
        - plugin_id: "context-env"
          order: 3

  orchestrator:
    enabled: true
    plugins:
      planner:
        plugin_id: "planner-hierarchical"
        config:
          max_depth: 3
          branch_factor: 3

      execution:
        plugin_id: "execution-parallel"
        config:
          max_concurrency: 5
          timeout_secs: 600

      synthesizer:
        plugin_id: "synthesizer-merge"
        config:
          merge_strategy: "weighted"

  sandbox:
    enabled: true
    plugins:
      tools:
        - plugin_id: "tool-shell"
          config:
            allowed_commands: ["python", "node", "cargo"]
        - plugin_id: "tool-file"
          config:
            max_file_size: 10485760
        - plugin_id: "tool-git"

      security:
        plugin_id: "policy-strict"
        config:
          network_access: false
          filesystem_policy: "sandbox_writable"

      resources:
        plugin_id: "limits-default"
        config:
          cpu_limit_secs: 300
          memory_limit_mb: 1024

  interface:
    enabled: true
    plugins:
      input:
        plugin_id: "input-cli"
        config:
          prompt: "wireframe> "

      output:
        plugin_id: "output-markdown"
        config:
          syntax_highlighting: true

  adapter:
    enabled: true
    plugins:
      model:
        plugin_id: "model-openai"
        config:
          api_key: "${OPENAI_API_KEY}"
          model: "gpt-4o"
          temperature: 0.7

      reasoning:
        plugin_id: "reasoning-chain-of-thought"
        config:
          max_steps: 10

      tool_selector:
        plugin_id: "selector-semantic"
```

### 5. New Directory Structure

```
Wireframe-AI/
├── modules/                          # Core module skeletons
│   ├── context-core/                 # Context orchestration
│   ├── orchestrator-core/            # Orchestrator orchestration
│   ├── sandbox-core/                 # Sandbox orchestration
│   ├── interface-core/               # Interface orchestration
│   └── adapter-core/                 # Adapter orchestration
├── plugins/                          # Universal plugin ecosystem
│   ├── context/                      # Context-specific plugins
│   │   ├── storage-sqlite/
│   │   ├── storage-postgres/
│   │   ├── memory-fts5/
│   │   ├── memory-rag/
│   │   ├── memory-graph/
│   │   ├── enrichment-files/
│   │   └── enrichment-env/
│   ├── orchestrator/                 # Orchestrator-specific plugins
│   │   ├── planner-linear/
│   │   ├── planner-hierarchical/
│   │   ├── planner-recursive/
│   │   ├── execution-parallel/
│   │   ├── execution-sequential/
│   │   └── synthesizer-merge/
│   ├── sandbox/                      # Sandbox-specific plugins
│   │   ├── tools/
│   │   │   ├── tool-shell/
│   │   │   ├── tool-file/
│   │   │   ├── tool-git/
│   │   │   └── tool-http/
│   │   ├── security/
│   │   │   ├── policy-strict/
│   │   │   ├── policy-permissive/
│   │   │   └── policy-custom/
│   │   └── resources/
│   │       ├── limits-default/
│   │       └── limits-custom/
├── sdk/
│   ├── agentic-sdk/
│   │   ├── src/
│   │   │   ├── plugin.rs             # Base Plugin trait
│   │   │   ├── plugin_registry.rs    # Universal registry
│   │   │   ├── pipeline.rs           # Pipeline orchestration
│   │   │   ├── plugins/              # Module-specific traits
│   │   │   │   ├── mod.rs
│   │   │   │   ├── context.rs
│   │   │   │   ├── orchestrator.rs
│   │   │   │   ├── sandbox.rs
│   │   │   │   ├── interface.rs
│   │   │   │   └── adapter.rs
│   │   │   └── config.rs             # Configuration loading
│   └── agentic-sdk-macros/
├── configs/
│   ├── wireframe-default.yaml
│   ├── wireframe-dev.yaml
│   └── wireframe-production.yaml
└── legacy/                           # Old monolithic modules (deprecated)
    ├── modules/
    │   ├── context/
    │   ├── orchestrator/
    │   └── sandbox/
```

## Module Core Implementations

### Context Core Module

```rust
// modules/context-core/src/main.rs

use agentic_sdk::{Plugin, PluginRegistry};
use agentic_sdk::plugins::context::{StorageBackend, MemoryBackend, EnrichmentStrategy};

pub struct ContextCore {
	registry: PluginRegistry,
	storage: Option<Box<dyn StorageBackend>>,
	memory: Option<Box<dyn MemoryBackend>>,
	enrichment_pipeline: Vec<Box<dyn EnrichmentStrategy>>,
}

impl ContextCore {
	pub async fn new(config_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
		let registry = PluginRegistry::new();
		registry.load_from_config(config_path).await?;

		// Load plugins based on config
		let storage = registry.get::<dyn StorageBackend>("storage-sqlite").await?;
		let memory = registry.get::<dyn MemoryBackend>("memory-rag").await?;
		let enrichment = registry.get_pipeline::<dyn EnrichmentStrategy>("context").await?;

		Ok(Self {
			registry,
			storage: Some(storage),
			memory: Some(memory),
			enrichment_pipeline: enrichment,
		})
	}

	pub async fn handle_task(&mut self, task: TaskSubmitted) -> Result<TaskEnriched> {
		// Build base context
		let mut context = self.build_base_context(&task).await?;

		// Run enrichment pipeline
		for plugin in &self.enrichment_pipeline {
			context = plugin.enrich(&task, &context).await?;
		}

		Ok(TaskEnriched {
			session_id: task.session_id,
			correlation_id: uuid::Uuid::new_v4().to_string(),
			user_input: task.user_input,
			context,
			inferred_constraints: vec![],
			enriched_at: chrono::Utc::now().timestamp(),
		})
	}
}
```

### Orchestrator Core Module

```rust
// modules/orchestrator-core/src/main.rs

use agentic_sdk::plugins::orchestrator::{TaskPlanner, ExecutionStrategy, ResultSynthesizer};

pub struct OrchestratorCore {
	registry: PluginRegistry,
	planner: Option<Box<dyn TaskPlanner>>,
	execution: Option<Box<dyn ExecutionStrategy>>,
	synthesizer: Option<Box<dyn ResultSynthesizer>>,
}

impl OrchestratorCore {
	pub async fn handle_task(&mut self, task: TaskEnriched) -> Result<TaskComplete> {
		// Decompose task
		let sub_tasks = self
			.planner
			.as_ref()
			.ok_or("No planner configured")?
			.decompose(&task)
			.await?;

		// Dispatch jobs
		let job_ids = self
			.execution
			.as_ref()
			.ok_or("No execution strategy configured")?
			.dispatch_jobs(sub_tasks)
			.await?;

		// Collect results
		let results = self
			.execution
			.as_ref()
			.ok_or("No execution strategy configured")?
			.collect_results(&task.correlation_id, job_ids.len())
			.await?;

		// Synthesize final result
		let complete = self
			.synthesizer
			.as_ref()
			.ok_or("No synthesizer configured")?
			.synthesize(results, &task)
			.await?;

		Ok(complete)
	}
}
```

## Plugin Implementation Examples

### RAG Memory Plugin

```rust
// plugins/context/memory-rag/src/lib.rs

use agentic_sdk::{Plugin, PluginError};
use agentic_sdk::plugins::context::MemoryBackend;

pub struct RagMemoryPlugin {
	vector_db: QdrantClient,
	embedding_model: EmbeddingModel,
}

#[async_trait]
impl Plugin for RagMemoryPlugin {
	fn plugin_id(&self) -> &'static str { "memory-rag" }
	fn version(&self) -> &'static str { "1.0.0" }
	fn description(&self) -> &'static str { "RAG-based memory with vector search" }

	async fn initialize(&mut self, config: &Value) -> Result<(), PluginError> {
		let db_url = config["vector_db"]["url"].as_str().unwrap();
		self.vector_db = QdrantClient::connect(db_url).await?;
		Ok(())
	}

	async fn health_check(&self) -> Result<bool, PluginError> {
		Ok(self.vector_db.ping().await.is_ok())
	}

	async fn shutdown(&mut self) -> Result<(), PluginError> {
		self.vector_db.close().await?;
		Ok(())
	}
}

#[async_trait]
impl MemoryBackend for RagMemoryPlugin {
	async fn search(
		&self,
		query: &str,
		session_id: &str,
		limit: usize,
	) -> Result<Vec<MemoryChunk>, MemoryError> {
		// Generate embedding
		let embedding = self.embedding_model.embed(query).await?;

		// Vector search
		let results = self.vector_db.search(&embedding, limit).await?;

		// Convert to MemoryChunk
		Ok(results.into_iter().map(|r| MemoryChunk {
			id: r.id,
			content: r.payload["content"].as_str().unwrap().to_string(),
			source: r.payload["source"].as_str().unwrap().to_string(),
			relevance_score: r.score,
		}).collect())
	}

	async fn persist_chunk(
		&self,
		session_id: &str,
		content: &str,
		source: &str,
	) -> Result<(), MemoryError> {
		let embedding = self.embedding_model.embed(content).await?;
		self.vector_db.insert(session_id, content, source, embedding).await?;
		Ok(())
	}

	async fn load_chunks(
		&self,
		session_id: &str,
		limit: usize,
	) -> Result<Vec<MemoryChunk>, MemoryError> {
		let results = self.vector_db.load_session(session_id, limit).await?;
		Ok(results.into_iter().map(|r| MemoryChunk {
			id: r.id,
			content: r.payload["content"].as_str().unwrap().to_string(),
			source: r.payload["source"].as_str().unwrap().to_string(),
			relevance_score: 1.0,
		}).collect())
	}
}
```

### Hierarchical Planner Plugin

```rust
// plugins/orchestrator/planner-hierarchical/src/lib.rs

use agentic_sdk::plugins::orchestrator::TaskPlanner;

pub struct HierarchicalPlanner {
	max_depth: usize,
	branch_factor: usize,
}

#[async_trait]
impl Plugin for HierarchicalPlanner {
	fn plugin_id(&self) -> &'static str { "planner-hierarchical" }
	fn version(&self) -> &'static str { "1.0.0" }
	fn description(&self) -> &'static str { "Hierarchical task decomposition" }

	async fn initialize(&mut self, config: &Value) -> Result<(), PluginError> {
		self.max_depth = config["max_depth"].as_u64().unwrap() as usize;
		self.branch_factor = config["branch_factor"].as_u64().unwrap() as usize;
		Ok(())
	}

	async fn health_check(&self) -> Result<bool, PluginError> { Ok(true) }
	async fn shutdown(&mut self) -> Result<(), PluginError> { Ok(()) }
}

#[async_trait]
impl TaskPlanner for HierarchicalPlanner {
	async fn decompose(
		&self,
		task: &TaskEnriched,
	) -> Result<Vec<TaskDescription>, PlanningError> {
		let prompt = format!(
			"Decompose this task into subtasks (max depth {}, {} branches per level):\n{}",
			self.max_depth, self.branch_factor, task.user_input
		);

		let decomposition = self.call_planning_model(&prompt).await?;
		self.parse_decomposition(decomposition).await
	}
}
```

## Migration Strategy

### Phase 1: SDK Foundation (Week 1-2)

**Goal**: Build the plugin infrastructure without touching existing modules

1. **Create plugin traits in SDK**
    - Add `plugin.rs` with base `Plugin` trait
    - Create `plugins/` module with module-specific traits
    - Implement `plugin_registry.rs` for universal plugin management
    - Add `config.rs` for configuration loading/parsing
    - Write comprehensive unit tests
2. **Build plugin loading system**
    - Support compile-time plugin registration
    - Implement dynamic loading (dlopen/LoadLibrary)
    - Add plugin discovery from configuration files
    - Create plugin lifecycle management
3. **Documentation**
    - Plugin development guide
    - Trait documentation with examples
    - Best practices for plugin design

### Phase 2: Context Module Migration (Week 3-4)

**Goal**: Extract context module logic into plugins while keeping it functional

1. **Create context-core module**
    - Extract NATS handling from current context
    - Implement plugin orchestration
    - Keep existing SQLite/FTS5 as default plugins
2. **Extract storage plugin**
    - Move SQLite logic to `plugins/context/storage-sqlite`
    - Implement `StorageBackend` trait
    - Add tests
3. **Extract memory plugin**
    - Move FTS5 logic to `plugins/context/memory-fts5`
    - Implement `MemoryBackend` trait
    - Add tests
4. **Extract enrichment plugins**
    - Move env filtering to `plugins/context/enrichment-env`
    - Implement `EnrichmentStrategy` trait
    - Add tests
5. **Backward compatibility**
    - Keep monolithic context module in `legacy/`

### Phase 3: Orchestrator Module Migration (Week 5-6)

**Goal**: Apply same pattern to orchestrator

1. **Create orchestrator-core module**
    - Extract NATS handling
    - Implement plugin orchestration
    - Keep current parallel fan-out as default plugin
2. **Extract planner plugin**
    - Move current N-copy logic to `plugins/orchestrator/planner-linear`
    - Implement `TaskPlanner` trait
    - Add tests
3. **Extract execution plugin**
    - Move parallel dispatch logic to `plugins/orchestrator/execution-parallel`
    - Implement `ExecutionStrategy` trait
    - Add tests
4. **Extract synthesizer plugin**
    - Move result merging logic to `plugins/orchestrator/synthesizer-merge`
    - Implement `ResultSynthesizer` trait
    - Add tests

### Phase 4: Sandbox Module Migration (Week 7-8)

**Goal**: Modularize sandbox tool and security implementations

1. **Create sandbox-core module**
    - Extract MCP server setup
    - Implement plugin orchestration
    - Keep current tools as default plugins
2. **Extract tool plugins**
    - Move shell_exec to `plugins/sandbox/tools/tool-shell`
    - Move file operations to `plugins/sandbox/tools/tool-file`
    - Implement `Tool` trait
    - Add tests
3. **Extract security plugin**
    - Move whitelist logic to `plugins/sandbox/security/policy-whitelist`
    - Implement `SecurityPolicy` trait
    - Add tests
4. **Extract resource limiter plugin**
    - Move rlimit logic to `plugins/sandbox/resources/limits-unix`
    - Implement `ResourceLimiter` trait
    - Add tests

### Phase 5: Interface & Adapter Modules (Week 9-10)

**Goal**: Apply pattern to remaining modules

1. **Interface module**
    - Create interface-core
    - Extract CLI input to plugin
    - Extract markdown output to plugin
    - Add web interface plugin (optional)
2. **Adapter module**
    - Create adapter-core
    - Extract OpenAI model to plugin
    - Extract tool selection to plugin
    - Add support for other models (Anthropic, local)

### Phase 6: New Plugin Development (Week 11-12)

**Goal**: Add new capabilities through plugins

1. **Orchestrator plugins**
    - Implement hierarchical planner
    - Add recursive planner
    - Create sequential execution strategy
2. **Sandbox plugins**
    - Add HTTP tool plugin
    - Implement Docker sandbox plugin
    - Create custom security policies
3. **Adapter plugins**
    - Add Anthropic model plugin
    - Implement local LLaMA plugin
    - Create tree-of-thought reasoning plugin

### Phase 7: Testing & Documentation (Week 13-14) ✅ COMPLETED

**Goal**: Ensure reliability and usability

1. **Integration testing**
    - Test all plugin combinations
    - Performance benchmarks
    - Load testing
2. **Documentation**
    - Complete plugin development guide
    - Configuration examples
    - API reference
3. **Examples**
    - Sample configurations
    - Plugin templates
    - Tutorial walkthroughs

### Phase 8: Advanced Features & Optimization (Week 15-16) 🔄 IN PROGRESS

**Goal**: Implement monitoring infrastructure and create production deployment documentation

**Scope (Streamlined):**
1. **Monitoring and Tracing** - Metrics collection and distributed tracing infrastructure
2. **Performance Documentation** - Performance optimization guide
3. **Production Documentation** - Production deployment guide and configuration
4. **Load Testing** - Load testing script for performance validation
5. **Documentation Updates** - Update README and plan with new features

**Note:** Advanced plugin implementations (RAG memory, LLM planner, Docker sandbox, PostgreSQL storage, web interface) deferred to future phases.

## Configuration Examples

### Minimal Configuration (Current Behavior)

```yaml
modules:
  context:
    plugins:
      storage: { plugin_id: "storage-sqlite" }
      memory: { plugin_id: "memory-fts5" }
      enrichment_pipeline:
        - plugin_id: "enrichment-env"

  orchestrator:
    plugins:
      planner: { plugin_id: "planner-linear" }
      execution: { plugin_id: "execution-parallel" }
      synthesizer: { plugin_id: "synthesizer-merge" }

  sandbox:
    plugins:
      tools:
        - plugin_id: "tool-shell"
        - plugin_id: "tool-file"
      security: { plugin_id: "policy-whitelist" }
```

### RAG-Enabled Configuration

```yaml
modules:
  context:
    plugins:
      storage: { plugin_id: "storage-sqlite" }
      memory:
        plugin_id: "memory-rag"
        config:
          vector_db: { type: "qdrant", url: "http://localhost:6333" }
          embedding: { model: "text-embedding-3-small" }
      enrichment_pipeline:
        - plugin_id: "memory-rag"
          config: { top_k: 20, min_score: 0.7 }
        - plugin_id: "enrichment-files"
          config: { max_files: 10 }
        - plugin_id: "enrichment-env"
```

### Advanced Orchestrator Configuration

```yaml
modules:
  orchestrator:
    plugins:
      planner:
        plugin_id: "planner-hierarchical"
        config:
          max_depth: 3
          branch_factor: 3
          use_llm_planning: true
      execution:
        plugin_id: "execution-adaptive"
        config:
          initial_concurrency: 3
          max_concurrency: 10
          scale_up_threshold: 0.8
      synthesizer:
        plugin_id: "synthesizer-llm"
        config:
          model: "gpt-4o"
          merge_strategy: "semantic"
```

### Strict Sandbox Configuration

```yaml
modules:
  sandbox:
    plugins:
      tools:
        - plugin_id: "tool-shell"
          config:
            allowed_commands: ["python", "node"]
        - plugin_id: "tool-file"
          config:
            read_only: true
            max_file_size: 1048576
      security:
        plugin_id: "policy-strict"
        config:
          network_access: false
          filesystem_policy: "readonly"
          allow_subprocess: false
      resources:
        plugin_id: "limits-hard"
        config:
          cpu_limit_secs: 60
          memory_limit_mb: 512
```

## Plugin Development Example

Creating a new plugin is straightforward:

```rust
// plugins/context/memory-custom/src/lib.rs

use agentic_sdk::{Plugin, PluginError};
use agentic_sdk::plugins::context::MemoryBackend;

pub struct CustomMemoryPlugin {
	// Your custom state
}

#[async_trait]
impl Plugin for CustomMemoryPlugin {
	fn plugin_id(&self) -> &'static str { "memory-custom" }
	fn version(&self) -> &'static str { "1.0.0" }
	fn description(&self) -> &'static str { "My custom memory backend" }

	async fn initialize(&mut self, config: &Value) -> Result<(), PluginError> {
		// Initialize from config
		Ok(())
	}

	async fn health_check(&self) -> Result<bool, PluginError> { Ok(true) }
	async fn shutdown(&mut self) -> Result<(), PluginError> { Ok(()) }
}

#[async_trait]
impl MemoryBackend for CustomMemoryPlugin {
	async fn search(
		&self,
		query: &str,
		session_id: &str,
		limit: usize,
	) -> Result<Vec<MemoryChunk>, MemoryError> {
		// Your custom search logic
		Ok(vec![])
	}

	async fn persist_chunk(
		&self,
		session_id: &str,
		content: &str,
		source: &str,
	) -> Result<(), MemoryError> {
		// Your custom persistence logic
		Ok(())
	}

	async fn load_chunks(
		&self,
		session_id: &str,
		limit: usize,
	) -> Result<Vec<MemoryChunk>, MemoryError> {
		// Your custom loading logic
		Ok(vec![])
	}
}
```

Then register it in config:

```yaml
modules:
  context:
    plugins:
      memory:
        plugin_id: "memory-custom"
        config:
          # Your custom config
```

## Key Benefits

### 1. Runtime Flexibility

- Swap any module's implementation without recompilation
- Mix and match plugins across modules
- A/B test different strategies

### 2. Third-Party Ecosystem

- Community can develop plugins for any module
- Standardized plugin marketplace
- Plugin versioning and dependency management

### 3. Performance Optimization

- Disable unused plugins
- Per-module resource allocation
- Parallel plugin execution where possible

### 4. Security & Isolation

- Plugins can run in separate processes
- Per-plugin security policies
- Sandboxed plugin execution

## Summary

This universal plugin architecture transforms Wireframe AI from a set of monolithic modules into a highly extensible platform where every component can be customized, extended, or replaced at runtime. The design maintains the existing NATS-based communication pattern while adding a powerful plugin system that enables:

- **Runtime composition** of module behavior
- **Third-party extensibility** through standardized plugin interfaces
- **Performance optimization** through selective plugin loading
- **Security isolation** through sandboxed plugin execution

The 16-week implementation plan provides a clear path from the current architecture to the fully modularized system, with each phase building on the previous one and maintaining system functionality throughout the transition.