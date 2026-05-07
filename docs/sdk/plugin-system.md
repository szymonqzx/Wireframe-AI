# Wireframe-AI Plugin System

The Wireframe-AI SDK includes a universal plugin system that enables runtime composition of module behavior. Every module in the system can be extended with plugins without recompilation.

## Core Concepts

### Plugin Trait

All plugins implement the base `Plugin` trait, which provides:

- **Identification**: `plugin_id()`, `version()`, `description()`
- **Lifecycle**: `initialize()`, `health_check()`, `shutdown()`
- **Configuration**: Plugins receive JSON config on initialization

```rust
use agentic_sdk::Plugin;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
impl Plugin for MyPlugin {
    fn plugin_id(&self) -> &'static str {
        "my-plugin"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "My custom plugin"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), PluginError> {
        // Initialize from config
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        // Cleanup
        Ok(())
    }
}
```

### Module-Specific Traits

Each module has its own set of plugin traits:

- **Context**: `StorageBackend`, `MemoryBackend`, `EnrichmentStrategy`
- **Orchestrator**: `TaskPlanner`, `ExecutionStrategy`, `ResultSynthesizer`
- **Sandbox**: `Tool`, `SecurityPolicy`, `ResourceLimiter`
- **Interface**: `InputMethod`, `OutputFormatter`, `UIComponent`
- **Adapter**: `AIModel`, `ToolSelector`, `ReasoningStrategy`

### Plugin Registry

The `PluginRegistry` manages plugin lifecycle:

```rust
use agentic_sdk::PluginRegistry;

let registry = PluginRegistry::new();
registry.register(Box::new(my_plugin)).await.unwrap();

let plugin = registry.get::<MyPlugin>("my-plugin").await?;
```

### Configuration

Plugins are configured via YAML or JSON:

```yaml
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
```

### Pipeline Execution

Plugins can be executed in ordered pipelines:

```rust
use agentic_sdk::Pipeline;

let mut pipeline = Pipeline::new();
pipeline.add_step(PipelineStep {
    plugin: Box::new(plugin1),
    order: 1,
});
pipeline.add_step(PipelineStep {
    plugin: Box::new(plugin2),
    order: 2,
});

let result = pipeline.execute(input).await?;
```

## Creating a Plugin

### Example: Custom Memory Backend

```rust
use agentic_sdk::{Plugin, PluginError};
use agentic_sdk::plugins::context::MemoryBackend;
use async_trait::async_trait;

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

## Configuration Loading

```rust
use agentic_sdk::PluginConfig;
use std::path::PathBuf;

let config = PluginConfig::from_file(&PathBuf::from("./config.yaml"))?;

// Load plugins from config
let registry = PluginRegistry::new();
registry.load_from_config(&PathBuf::from("./config.yaml")).await?;
```

## Error Handling

All plugin operations return typed errors:

```rust
use agentic_sdk::PluginError;

match plugin.initialize(&config).await {
    Ok(()) => tracing::info!("Plugin initialized"),
    Err(PluginError::InitializationFailed(msg)) => {
        tracing::error!("Initialization failed: {}", msg);
    }
    Err(e) => tracing::error!("Plugin error: {}", e),
}
```

## Best Practices

1. **Keep plugins focused**: Each plugin should do one thing well
2. **Use async properly**: All plugin methods are async
3. **Handle errors gracefully**: Return typed errors, don't panic
4. **Resource cleanup**: Always release resources in `shutdown()`
5. **Version compatibility**: Follow semantic versioning
6. **Documentation**: Provide clear descriptions of plugin behavior

## Next Steps

- See the modularization plan for plugin implementation phases
- Check module-specific trait documentation for implementation details
- Review example plugins in the `plugins/` directory (coming in later phases)
