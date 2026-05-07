# Wireframe-AI Plugin Development Guide

Develop plugins for Wireframe-AI modules to extend system capabilities with custom implementations.

## Overview

Wireframe-AI uses a plugin architecture where each module (Context, Orchestrator, Sandbox, Interface) loads plugins through trait implementations. Plugins are dynamically loaded at runtime and managed by the `PluginRegistry`.

### Plugin Lifecycle

1. **Initialization**: Plugin is loaded and configured via `initialize()`
2. **Health Check**: Periodic health checks via `health_check()`
3. **Execution**: Plugin-specific trait methods are called
4. **Shutdown**: Plugin cleanup via `shutdown()`

### Module Plugin Types

| Module | Plugin Traits | Purpose |
|--------|--------------|---------|
| Context | `StorageBackend`, `MemoryBackend`, `EnrichmentStrategy` | Session storage, memory retrieval, context enrichment |
| Orchestrator | `TaskPlanner`, `ExecutionStrategy`, `ResultSynthesizer` | Task decomposition, execution dispatch, result synthesis |
| Sandbox | `Tool`, `SecurityPolicy`, `ResourceLimiter` | Tool execution, security validation, resource limits |
| Interface | `InputMethod`, `OutputFormatter`, `UIComponent` | User input, output formatting, UI rendering |

## Core Plugin Trait

All plugins implement the base `Plugin` trait:

```rust
use agentic_sdk::plugin::Plugin;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Plugin: Send + Sync + Any {
    fn plugin_id(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn description(&self) -> &'static str;
    
    async fn initialize(&mut self, config: &Value) -> Result<(), PluginError>;
    async fn health_check(&self) -> Result<bool, PluginError>;
    async fn shutdown(&mut self) -> Result<(), PluginError>;
}
```

## Context Module Plugins

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

**Example: SQLite Storage**

```rust
use agentic_sdk::plugins::context::{StorageBackend, StorageError};
use agentic_sdk::plugin::{Plugin, PluginError};
use async_trait::async_trait;
use serde_json::Value;
use sqlx::SqlitePool;

pub struct SQLiteStorage {
    pool: SqlitePool,
}

impl SQLiteStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Plugin for SQLiteStorage {
    fn plugin_id(&self) -> &'static str {
        "storage-sqlite"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "SQLite storage backend for sessions and messages"
    }

    async fn initialize(&mut self, _config: &Value) -> Result<(), PluginError> {
        // Initialize database schema
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PluginError::InitializationFailed(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| PluginError::InitializationFailed(e.to_string()))?;

        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| true)
            .map_err(|e| PluginError::HealthCheckFailed(e.to_string()))
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        self.pool.close().await;
        Ok(())
    }
}

#[async_trait]
impl StorageBackend for SQLiteStorage {
    async fn ensure_session(&self, session_id: &str) -> Result<(), StorageError> {
        sqlx::query(
            "INSERT OR IGNORE INTO sessions (id, created_at) VALUES (?, ?)",
        )
        .bind(session_id)
        .bind(chrono::Utc::now().timestamp())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn store_message(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> Result<(), StorageError> {
        sqlx::query(
            "INSERT INTO messages (id, session_id, role, content, timestamp) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(session_id)
        .bind(role)
        .bind(content)
        .bind(chrono::Utc::now().timestamp())
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn load_session_history(
        &self,
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<ChatMessage>, StorageError> {
        let rows = sqlx::query_as::<_, (String, String, i64)>(
            "SELECT role, content, timestamp FROM messages WHERE session_id = ? ORDER BY timestamp DESC LIMIT ?",
        )
        .bind(session_id)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        let messages = rows
            .into_iter()
            .rev()
            .map(|(role, content, timestamp)| ChatMessage {
                role,
                content,
                timestamp,
            })
            .collect();

        Ok(messages)
    }
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

## Orchestrator Module Plugins

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

**Example: Hierarchical Planner**

```rust
use agentic_sdk::plugins::orchestrator::{TaskPlanner, PlanningError, TaskDescription};
use agentic_sdk::plugin::{Plugin, PluginError};
use agentic_sdk::message_types::TaskEnriched;
use async_trait::async_trait;
use serde_json::Value;

pub struct HierarchicalPlanner;

#[async_trait]
impl Plugin for HierarchicalPlanner {
    fn plugin_id(&self) -> &'static str {
        "planner-hierarchical"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Hierarchical task decomposition planner"
    }

    async fn initialize(&mut self, _config: &Value) -> Result<(), PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait]
impl TaskPlanner for HierarchicalPlanner {
    async fn decompose(&self, task: &TaskEnriched) -> Result<Vec<TaskDescription>, PlanningError> {
        // Simple decomposition: split by "and" for demonstration
        let subtasks: Vec<TaskDescription> = task
            .user_input
            .split(" and ")
            .map(|s| TaskDescription {
                description: s.trim().to_string(),
                dependencies: vec![],
                metadata: Value::Null,
            })
            .collect();

        if subtasks.is_empty() {
            return Err(PlanningError::DecompositionFailed(
                "No subtasks found".to_string(),
            ));
        }

        Ok(subtasks)
    }
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

## Sandbox Module Plugins

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

**Example: HTTP Tool**

```rust
use agentic_sdk::plugins::sandbox::{Tool, ToolError, SandboxContext};
use agentic_sdk::plugin::{Plugin, PluginError};
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct HttpTool;

#[async_trait]
impl Plugin for HttpTool {
    fn plugin_id(&self) -> &'static str {
        "tool-http"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "HTTP request tool for web API calls"
    }

    async fn initialize(&mut self, _config: &Value) -> Result<(), PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait]
impl Tool for HttpTool {
    fn tool_name(&self) -> &'static str {
        "http"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": {"type": "string"},
                "method": {"type": "string", "enum": ["GET", "POST", "PUT", "DELETE"]},
                "headers": {"type": "object"},
                "body": {"type": "string"}
            },
            "required": ["url", "method"]
        })
    }

    async fn execute(
        &self,
        params: Value,
        _sandbox_context: &SandboxContext,
    ) -> Result<Value, ToolError> {
        let url = params
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing url".to_string()))?;

        let method = params
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing method".to_string()))?;

        // Execute HTTP request (simplified)
        let client = reqwest::Client::new();
        let response = match method {
            "GET" => client.get(url).send().await,
            "POST" => client.post(url).send().await,
            "PUT" => client.put(url).send().await,
            "DELETE" => client.delete(url).send().await,
            _ => return Err(ToolError::InvalidParameters("Invalid method".to_string())),
        };

        match response {
            Ok(resp) => {
                let status = resp.status().as_u16();
                let body = resp.text().await.map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
                Ok(json!({ "status": status, "body": body }))
            }
            Err(e) => Err(ToolError::ExecutionFailed(e.to_string())),
        }
    }
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

## Interface Module Plugins

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

**Example: CLI Input**

```rust
use agentic_sdk::plugins::interface::{InputMethod, InputError};
use agentic_sdk::plugin::{Plugin, PluginError};
use agentic_sdk::message_types::TaskSubmitted;
use async_trait::async_trait;
use serde_json::Value;

pub struct CliInput;

#[async_trait]
impl Plugin for CliInput {
    fn plugin_id(&self) -> &'static str {
        "input-cli"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "CLI input method for interactive sessions"
    }

    async fn initialize(&mut self, _config: &Value) -> Result<(), PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait]
impl InputMethod for CliInput {
    async fn read_input(&self) -> Result<TaskSubmitted, InputError> {
        print!("> ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| InputError::ReadFailed(e.to_string()))?;

        Ok(TaskSubmitted {
            session_id: uuid::Uuid::new_v4().to_string(),
            user_input: input.trim().to_string(),
            submitted_at: chrono::Utc::now().timestamp(),
        })
    }
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

### UIComponent

Interactive UI elements (progress bars, rich output, etc.).

```rust
use agentic_sdk::plugins::interface::{UIComponent, UIError, UIState};

#[async_trait]
pub trait UIComponent: Plugin {
    async fn render(&self, state: &UIState) -> Result<(), UIError>;
}
```

## Configuration Patterns

Plugins are configured via YAML files in the `configs/` directory:

```yaml
plugins:
  storage:
    type: storage-sqlite
    config:
      database_path: /tmp/wireframe.db
  memory:
    type: memory-fts5
    config:
      index_path: /tmp/memory.idx
  enrichment:
    type: enrichment-env
    config:
      allowed_vars:
        - HOME
        - PATH
```

Configuration is passed to the `initialize()` method as a `serde_json::Value`.

## Testing Patterns

### Unit Tests

Test plugin methods in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_backend() {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let storage = SQLiteStorage::new(pool);

        // Initialize
        storage.initialize(&json!({})).await.unwrap();

        // Ensure session
        storage.ensure_session("test-session").await.unwrap();

        // Store message
        storage
            .store_message("test-session", "user", "Hello")
            .await
            .unwrap();

        // Load history
        let history = storage.load_session_history("test-session", 10).await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].content, "Hello");
    }
}
```

### Integration Tests

Test plugin combinations:

```rust
#[tokio::test]
async fn test_context_pipeline() {
    let storage = SQLiteStorage::new(/* ... */);
    let memory = FTS5Memory::new(/* ... */);
    let enrichment = EnvEnrichment::new(/* ... */);

    // Initialize all plugins
    storage.initialize(&json!({})).await.unwrap();
    memory.initialize(&json!({})).await.unwrap();
    enrichment.initialize(&json!({})).await.unwrap();

    // Test pipeline
    let task = TaskSubmitted {
        session_id: "test".to_string(),
        user_input: "Hello".to_string(),
        submitted_at: chrono::Utc::now().timestamp(),
    };

    let mut context = ContextPackage::default();
    context = enrichment.enrich(&task, &context).await.unwrap();

    assert!(!context.memory_chunks.is_empty() || !context.session_history.is_empty());
}
```

## Common Pitfalls

### 1. Blocking in Async Code

Never block the async runtime. Use `.await` for all I/O operations.

```rust
// BAD
std::fs::read_to_string(path).unwrap();

// GOOD
tokio::fs::read_to_string(path).await.unwrap();
```

### 2. Ignoring Errors

Always handle errors properly. Use `?` operator or explicit error handling.

```rust
// BAD
let result = some_operation().unwrap();

// GOOD
let result = some_operation()
    .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
```

### 3. Not Cleaning Up Resources

Always implement `shutdown()` to clean up resources (connections, files, etc.).

```rust
async fn shutdown(&mut self) -> Result<(), PluginError> {
    self.pool.close().await;
    Ok(())
}
```

### 4. Hardcoding Configuration

Use configuration passed to `initialize()` instead of hardcoding values.

```rust
// BAD
const DB_PATH: &str = "/tmp/wireframe.db";

// GOOD
let db_path = config
    .get("database_path")
    .and_then(|v| v.as_str())
    .unwrap_or("/tmp/wireframe.db");
```

### 5. Not Implementing Health Checks

Health checks are critical for monitoring. Always implement meaningful health checks.

```rust
async fn health_check(&self) -> Result<bool, PluginError> {
    // Check database connection
    sqlx::query("SELECT 1")
        .fetch_one(&self.pool)
        .await
        .map(|_| true)
        .map_err(|e| PluginError::HealthCheckFailed(e.to_string()))
}
```

## Best Practices

1. **Use Async/Await**: All plugin methods are async. Use `.await` for I/O operations.
2. **Error Handling**: Use the appropriate error types for each plugin trait.
3. **Configuration**: Make plugins configurable via the `initialize()` method.
4. **Testing**: Write unit tests for each plugin method and integration tests for plugin combinations.
5. **Documentation**: Document plugin behavior, configuration options, and error conditions.
6. **Resource Management**: Implement proper cleanup in `shutdown()`.
7. **Health Checks**: Implement meaningful health checks for monitoring.
8. **Idempotency**: Make operations idempotent where possible (e.g., `ensure_session`).
9. **Logging**: Use logging for debugging and observability.
10. **Versioning**: Use semantic versioning for plugins.

## Next Steps

- See `docs/Configuration-Examples.md` for configuration examples
- See `docs/API-Reference.md` for detailed API documentation
- See `examples/configurations/complete-system.yaml` for a complete system configuration
- See existing plugin implementations in `modules/*/plugins/` for reference
- See `templates/plugins/` for ready-to-use plugin templates
- See `docs/Hello-World-Plugin-Tutorial.md` for a step-by-step tutorial
- See `benchmarks/` for performance benchmarks and profiling
