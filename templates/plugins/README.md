# Wireframe-AI Plugin Templates

This directory contains templates for creating custom plugins for Wireframe-AI modules.

## Available Templates

### Context Module Plugins

- **[context-storage-template.rs](context-storage-template.rs)** - Template for implementing a custom storage backend for the Context module
  - Implements `StorageBackend` trait
  - Handles session management and message persistence
  - Example: SQLite, PostgreSQL, Redis storage backends

### Orchestrator Module Plugins

- **[orchestrator-planner-template.rs](orchestrator-planner-template.rs)** - Template for implementing a custom task planner for the Orchestrator module
  - Implements `TaskPlanner` trait
  - Handles task decomposition into subtasks
  - Example: Linear, hierarchical, LLM-based planners

### Sandbox Module Plugins

- **[sandbox-tool-template.rs](sandbox-tool-template.rs)** - Template for implementing a custom tool for the Sandbox module
  - Implements `Tool` trait
  - Handles tool execution with parameter validation
  - Example: HTTP, git, custom API tools

## How to Use Templates

### Step 1: Copy the Template

Copy the template file to your plugin directory:

```bash
cp templates/plugins/context-storage-template.rs my-plugin/src/lib.rs
```

### Step 2: Customize the Template

Replace placeholder values with your own:

1. Replace `MyStorage`, `MyPlanner`, or `MyTool` with your plugin name
2. Update `plugin_id()` to return a unique identifier for your plugin
3. Update `version()` and `description()` with your version and description
4. Implement the trait methods with your custom logic
5. Add your plugin-specific fields and initialization logic

### Step 3: Create Cargo.toml

Create a `Cargo.toml` file for your plugin:

```toml
[package]
name = "my-plugin"
version = "0.1.0"
edition = "2021"

[dependencies]
agentic-sdk = { path = "../../sdk/agentic-sdk" }
async-trait = "0.1"
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
chrono = "0.4"

# Add additional dependencies as needed
# sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "sqlite"] }
# reqwest = { version = "0.11", features = ["json"] }
```

### Step 4: Add to Workspace

Add your plugin to the workspace `Cargo.toml`:

```toml
[workspace]
members = [
    # ... existing members ...
    "my-plugin",
]
```

### Step 5: Register Plugin

Register your plugin in your module:

```rust
use agentic_sdk::PluginRegistry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = PluginRegistry::new();
    
    // Create and register your plugin
    let plugin = Box::new(MyPlugin::new());
    registry.register(plugin).await?;
    
    // Use the plugin
    let plugin_instance: Arc<MyPlugin> = registry.get("my-plugin-id").await?;
    
    Ok(())
}
```

### Step 6: Configure Plugin

Add your plugin configuration to your module's YAML config:

```yaml
plugins:
  storage:
    plugin_id: "my-plugin-id"
    config:
      # Your plugin-specific configuration
      connection_string: "postgresql://localhost/mydb"
      pool_size: 10
```

## Plugin Types

### StorageBackend (Context Module)

Persist sessions and messages to a database.

**Methods:**
- `ensure_session(session_id)` - Ensure a session exists
- `store_message(session_id, role, content)` - Store a message
- `load_session_history(session_id, limit)` - Load session history

**Use Cases:**
- Custom database backends (PostgreSQL, MySQL, MongoDB)
- Cloud storage integration (S3, Azure Blob, GCS)
- Distributed storage (Redis, Cassandra)

### TaskPlanner (Orchestrator Module)

Decompose tasks into subtasks.

**Methods:**
- `decompose(task)` - Decompose a task into subtasks

**Use Cases:**
- Custom decomposition strategies
- LLM-based planning
- Domain-specific task analysis
- Dependency-aware planning

### Tool (Sandbox Module)

Executable tools for agent use.

**Methods:**
- `tool_name()` - Return tool name
- `input_schema()` - Return JSON schema for input validation
- `execute(params, context)` - Execute the tool

**Use Cases:**
- Custom API integrations
- Specialized file operations
- Domain-specific tools
- External service integrations

## Best Practices

1. **Error Handling**: Use appropriate error types for each trait
2. **Configuration**: Make plugins configurable via `initialize()`
3. **Testing**: Write comprehensive unit and integration tests
4. **Documentation**: Document your plugin's behavior and configuration
5. **Resource Management**: Properly cleanup resources in `shutdown()`
6. **Health Checks**: Implement meaningful health checks
7. **Idempotency**: Make operations idempotent where possible
8. **Logging**: Use logging for debugging and observability

## Examples

See existing plugin implementations for reference:

- `modules/context-core/plugins/storage-sqlite/` - SQLite storage backend
- `modules/orchestrator-core/plugins/planner-hierarchical/` - Hierarchical planner
- `modules/sandbox-core/plugins/tools/tool-http/` - HTTP tool

## Testing

Each template includes test examples. Run tests with:

```bash
cargo test -p my-plugin
```

## Next Steps

- See [Plugin Development Guide](../../docs/Plugin-Development-Guide.md) for detailed guidance
- See [Configuration Examples](../../docs/Configuration-Examples.md) for configuration patterns
- See [API Reference](../../docs/API-Reference.md) for detailed API documentation
