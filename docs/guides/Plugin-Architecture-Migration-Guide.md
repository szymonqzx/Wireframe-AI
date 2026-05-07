# Plugin Architecture Migration Guide

## Overview

Wireframe-AI has been migrated to a plugin architecture where core modules delegate functionality to plugins. This guide helps existing users understand the changes and adapt their workflows.

## What Changed

### Before Migration
- Core modules contained all implementation logic
- Limited extensibility - required code changes to add features
- Monolithic module structure

### After Migration
- Core modules orchestrate plugins through trait implementations
- High extensibility - add features by creating new plugins
- Modular architecture with clear separation of concerns

## Module Migration Status

| Module | Status | Core Module | Plugins |
|--------|--------|-------------|---------|
| Context | ✅ Complete | context-core | storage-sqlite, memory-fts5, enrichment-env |
| Orchestrator | ✅ Complete | orchestrator-core | planner-linear, planner-hierarchical, execution-parallel, execution-sequential, synthesizer-merge |
| Sandbox | ✅ Complete | sandbox-core | tool-shell, tool-file, tool-http, policy-whitelist, policy-custom, limits-unix |
| Interface | ✅ Complete | interface-core | input-cli, output-markdown |

## Configuration Changes

### New Configuration Files
Configuration has moved from inline code to YAML files in the `configs/` directory:

- `configs/context-default.yaml` - Context module configuration
- `configs/orchestrator-default.yaml` - Orchestrator module configuration
- `configs/sandbox-default.yaml` - Sandbox module configuration
- `configs/interface-default.yaml` - Interface module configuration

### Environment Variables
Configuration files now support environment variable expansion using `${VAR}` syntax:

```yaml
modules:
  context:
    plugins:
      storage:
        plugin_id: "storage-sqlite"
        config:
          db_path: "${DATABASE_PATH:-./wireframe_ai.db}"
```

### Hot-Reload Support
Configuration files support hot-reload through the ConfigWatcher, allowing runtime configuration changes without restarting modules.

## API Changes

### Core Module APIs
Core module APIs remain backward compatible. The migration was designed to preserve all existing functionality while adding plugin support.

### Plugin APIs
New plugin traits have been added to the SDK:

- `StorageBackend` - For context storage plugins
- `MemoryBackend` - For context memory plugins
- `EnrichmentStrategy` - For context enrichment plugins
- `TaskPlanner` - For orchestrator planning plugins
- `ExecutionStrategy` - For orchestrator execution plugins
- `ResultSynthesizer` - For orchestrator synthesis plugins
- `Tool` - For sandbox tool plugins
- `SecurityPolicy` - For sandbox security plugins
- `ResourceLimiter` - For sandbox resource limit plugins
- `InputMethod` - For interface input plugins
- `OutputFormatter` - For interface output plugins
- `UIComponent` - For interface UI component plugins

## Migration Steps for Existing Users

### Step 1: Update Dependencies
Ensure your `Cargo.toml` includes the agentic-sdk with the macros feature:

```toml
[dependencies]
agentic-sdk = { workspace = true, features = ["macros"] }
```

### Step 2: Update Configuration Files
If you have custom module configurations, migrate them to the new YAML format:

**Old format (inline code):**
```rust
let config = MyConfig {
    storage_path: "./data.db".to_string(),
    memory_limit: 1000,
};
```

**New format (YAML):**
```yaml
modules:
  my_module:
    enabled: true
    plugins:
      storage:
        plugin_id: "storage-sqlite"
        config:
          db_path: "${STORAGE_PATH:-./data.db}"
      memory:
        plugin_id: "memory-fts5"
        config:
          db_path: "${MEMORY_PATH:-./memory.db}"
```

### Step 3: Update Module Code
If you have custom module implementations, update them to use the plugin architecture:

**Old format (monolithic):**
```rust
struct MyModule {
    storage: Arc<SQLiteStorage>,
    memory: Arc<FTS5Memory>,
}

impl MyModule {
    async fn process(&self, input: &str) -> Result<String, Error> {
        // Direct implementation
        let history = self.storage.load_history().await?;
        let chunks = self.memory.search(input).await?;
        // Process...
    }
}
```

**New format (plugin-based):**
```rust
struct MyModule {
    storage: Arc<dyn StorageBackend>,
    memory: Arc<dyn MemoryBackend>,
}

impl MyModule {
    async fn process(&self, input: &str) -> Result<String, Error> {
        // Delegate to plugins
        let history = self.storage.load_session_history("session", 100).await?;
        let chunks = self.memory.search(input, "session", 10).await?;
        // Process...
    }
}
```

### Step 4: Enable Plugin Loading
Update your module's main function to load plugins from configuration:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = PluginConfig::from_file(&PathBuf::from("config.yaml"))?;
    let registry = PluginRegistry::new();
    
    // Load plugins from configuration
    registry.load_from_config(&PathBuf::from("config.yaml")).await?;
    
    // Set plugins on core module
    let storage = registry.get::<SQLiteStoragePlugin>("storage-sqlite").await?;
    let memory = registry.get::<FTS5MemoryPlugin>("memory-fts5").await?;
    
    let core = MyModule::new(storage, memory);
    core.run("nats://localhost:4222").await
}
```

## Creating Custom Plugins

### Step 1: Choose Plugin Type
Decide which module you want to extend and which plugin trait to implement:

- **Context Module**: StorageBackend, MemoryBackend, EnrichmentStrategy
- **Orchestrator Module**: TaskPlanner, ExecutionStrategy, ResultSynthesizer
- **Sandbox Module**: Tool, SecurityPolicy, ResourceLimiter
- **Interface Module**: InputMethod, OutputFormatter, UIComponent

### Step 2: Scaffold Plugin
Use the CLI scaffolding tool to create a new plugin:

```bash
cargo run --bin wireframe-cli -- new my-custom-plugin --template basic
```

### Step 3: Implement Plugin Trait
Implement the appropriate plugin trait:

```rust
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::context::StorageBackend;
use async_trait::async_trait;

#[async_trait]
impl Plugin for MyStoragePlugin {
    fn plugin_id(&self) -> &'static str {
        "my-storage"
    }
    
    fn version(&self) -> &'static str {
        "1.0.0"
    }
    
    fn description(&self) -> &'static str {
        "My custom storage backend"
    }
    
    async fn initialize(&mut self, config: &Value) -> Result<(), PluginError> {
        // Initialize plugin
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
impl StorageBackend for MyStoragePlugin {
    async fn ensure_session(&self, session_id: &str) -> Result<(), StorageError> {
        // Implement session creation
        Ok(())
    }
    
    async fn store_message(&self, session_id: &str, role: &str, content: &str) -> Result<(), StorageError> {
        // Implement message storage
        Ok(())
    }
    
    async fn load_session_history(&self, session_id: &str, limit: usize) -> Result<Vec<ChatMessage>, StorageError> {
        // Implement history loading
        Ok(vec![])
    }
}
```

### Step 4: Register Plugin
Add your plugin to the configuration file:

```yaml
modules:
  context:
    enabled: true
    plugins:
      storage:
        plugin_id: "my-storage"
        config:
          custom_setting: "value"
```

### Step 5: Test Plugin
Create tests for your plugin:

```rust
#[tokio::test]
async fn test_my_storage_plugin() {
    let mut plugin = MyStoragePlugin::new();
    let config = serde_json::json!({});
    
    plugin.initialize(&config).await.unwrap();
    assert!(plugin.health_check().await.unwrap());
    
    plugin.ensure_session("test_session").await.unwrap();
    plugin.shutdown().await.unwrap();
}
```

## Troubleshooting

### Issue: Module macro not found
**Error**: `cannot find module in agentic_sdk`
**Solution**: Add `features = ["macros"]` to the agentic-sdk dependency in Cargo.toml

### Issue: Borrow checker errors with envelope
**Error**: `borrow of partially moved value: env`
**Solution**: Clone envelope fields instead of moving them:
```rust
// Instead of:
topic: env.topic

// Use:
topic: env.topic.clone()
```

### Issue: Plugin not loading
**Error**: Plugin not found in registry
**Solution**: Ensure the plugin_id in configuration matches the plugin_id returned by the plugin trait

### Issue: Configuration not loading
**Error**: Configuration parse error
**Solution**: Validate YAML syntax and ensure all required fields are present

## Resources

- **SDK Quick Start**: `docs/SDK-Quick-Start.md`
- **Plugin Development Guide**: `docs/Plugin-Development-Guide.md`
- **Hello World Plugin Tutorial**: `docs/Hello-World-Plugin-Tutorial.md`
- **Example Modules**: `examples/` directory
- **CLI Tool**: `tools/wireframe-cli/`

## Support

For questions or issues:
1. Check the documentation in `docs/`
2. Review example modules in `examples/`
3. Consult `AGENTS.md` for development patterns
4. Check `docs/Phase-1-Implementation-Notes.md` for known issues and fixes
