# Hello World Plugin Tutorial

Learn how to create your first custom plugin for Wireframe-AI. This tutorial will guide you through creating a simple "Hello World" tool plugin that can be used by agents in the Sandbox module.

## Prerequisites

- Rust 1.80+ installed
- Wireframe-AI repository cloned
- Basic understanding of Rust and async/await

## What We'll Build

We'll create a simple tool plugin that:
1. Takes a name as input
2. Returns a greeting message
3. Demonstrates the complete plugin lifecycle

## Step 1: Create the Plugin Directory

Create a new directory for your plugin:

```bash
mkdir -p plugins/sandbox/tools/tool-helloworld
cd plugins/sandbox/tools/tool-helloworld
```

## Step 2: Create Cargo.toml

Create a `Cargo.toml` file:

```toml
[package]
name = "tool-helloworld"
version = "0.1.0"
edition = "2021"

[dependencies]
agentic-sdk = { path = "../../../../sdk/agentic-sdk" }
async-trait = "0.1"
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
chrono = "0.4"
```

## Step 3: Create the Plugin Implementation

Create `src/lib.rs`:

```rust
use agentic_sdk::plugins::sandbox::{Tool, ToolError, SandboxContext};
use agentic_sdk::plugin::{Plugin, PluginError};
use async_trait::async_trait;
use serde_json::{json, Value};

/// Hello World Tool - A simple greeting tool
pub struct HelloWorldTool;

impl HelloWorldTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Plugin for HelloWorldTool {
    fn plugin_id(&self) -> &'static str {
        "tool-helloworld"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "A simple tool that returns a greeting message"
    }

    async fn initialize(&mut self, _config: &Value) -> Result<(), PluginError> {
        println!("HelloWorldTool initialized");
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        println!("HelloWorldTool shutdown");
        Ok(())
    }
}

#[async_trait]
impl Tool for HelloWorldTool {
    fn tool_name(&self) -> &'static str {
        "helloworld"
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name to greet"
                }
            },
            "required": ["name"]
        })
    }

    async fn execute(
        &self,
        params: Value,
        _sandbox_context: &SandboxContext,
    ) -> Result<Value, ToolError> {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'name' parameter".to_string()))?;

        let greeting = format!("Hello, {}!", name);

        Ok(json!({
            "greeting": greeting,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}
```

## Step 4: Create Tests

Create `src/lib.rs` with tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use agentic_sdk::plugins::sandbox::SandboxContext;

    #[tokio::test]
    async fn test_tool_initialization() {
        let mut tool = HelloWorldTool::new();
        let config = json!({});
        assert!(tool.initialize(&config).await.is_ok());
    }

    #[tokio::test]
    async fn test_tool_name() {
        let tool = HelloWorldTool::new();
        assert_eq!(tool.tool_name(), "helloworld");
    }

    #[tokio::test]
    async fn test_input_schema() {
        let tool = HelloWorldTool::new();
        let schema = tool.input_schema();
        assert!(schema.is_object());
        assert!(schema["properties"]["name"].is_object());
    }

    #[tokio::test]
    async fn test_execute_with_valid_name() {
        let tool = HelloWorldTool::new();
        let context = SandboxContext {
            working_dir: "/tmp".to_string(),
            environment: vec![],
            allowed_paths: vec![],
        };

        let params = json!({ "name": "World" });
        let result = tool.execute(params, &context).await.unwrap();

        assert_eq!(result["greeting"], "Hello, World!");
        assert!(result["timestamp"].is_string());
    }

    #[tokio::test]
    async fn test_execute_with_missing_name() {
        let tool = HelloWorldTool::new();
        let context = SandboxContext {
            working_dir: "/tmp".to_string(),
            environment: vec![],
            allowed_paths: vec![],
        };

        let params = json!({});
        let result = tool.execute(params, &context).await;

        assert!(result.is_err());
    }
}
```

## Step 5: Add to Workspace

Add your plugin to the workspace `Cargo.toml`:

```toml
[workspace]
members = [
    # ... existing members ...
    "plugins/sandbox/tools/tool-helloworld",
]
```

## Step 6: Build the Plugin

Build your plugin:

```bash
cargo build -p tool-helloworld
```

## Step 7: Run Tests

Run the tests:

```bash
cargo test -p tool-helloworld
```

You should see:

```
running 5 tests
test tests::test_tool_initialization ... ok
test tests::test_tool_name ... ok
test tests::test_input_schema ... ok
test tests::test_execute_with_valid_name ... ok
test tests::test_execute_with_missing_name ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Step 8: Integrate with Sandbox Module

Now let's integrate the plugin with the Sandbox module. Update the Sandbox module's plugin loading logic to include your new tool.

### Update Sandbox Configuration

Add your tool to the sandbox configuration:

```yaml
modules:
  sandbox:
    enabled: true
    plugins:
      tools:
        - plugin_id: "tool-helloworld"
          config: {}
        # ... other tools ...
```

### Register Plugin in Sandbox Module

In the Sandbox module's main.rs, register your plugin:

```rust
use tool_helloworld::HelloWorldTool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = PluginRegistry::new();

    // Register your tool
    let helloworld_tool = Box::new(HelloWorldTool::new());
    registry.register(helloworld_tool).await?;

    // ... rest of the module initialization ...
}
```

## Step 9: Test the Plugin

Start the Wireframe-AI system and test your plugin:

```bash
# Start NATS
docker run -p 4222:4222 nats:latest

# Start all modules
cargo run --release -p wireframe-ai-context &
cargo run --release -p wireframe-ai-orchestrator &
cargo run --release -p wireframe-ai-sandbox &
cargo run --release -p wireframe-ai-interface
```

In the interface, ask the agent to use your tool:

```
> Use the helloworld tool to greet "Alice"
```

The agent should respond with:

```
Hello, Alice!
```

## Step 10: Advanced Features

Let's add some advanced features to make the tool more useful.

### Add Custom Greeting Templates

Update the tool to support custom greeting templates:

```rust
pub struct HelloWorldTool {
    greeting_template: String,
}

impl HelloWorldTool {
    pub fn new() -> Self {
        Self {
            greeting_template: "Hello, {name}!".to_string(),
        }
    }
}

#[async_trait]
impl Plugin for HelloWorldTool {
    async fn initialize(&mut self, config: &Value) -> Result<(), PluginError> {
        if let Some(template) = config.get("greeting_template").and_then(|v| v.as_str()) {
            self.greeting_template = template.to_string();
        }
        Ok(())
    }
}

#[async_trait]
impl Tool for HelloWorldTool {
    async fn execute(
        &self,
        params: Value,
        _sandbox_context: &SandboxContext,
    ) -> Result<Value, ToolError> {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'name' parameter".to_string()))?;

        let greeting = self.greeting_template.replace("{name}", name);

        Ok(json!({
            "greeting": greeting,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}
```

Update the configuration:

```yaml
tools:
  - plugin_id: "tool-helloworld"
    config:
      greeting_template: "Greetings, {name}! Welcome to Wireframe-AI."
```

### Add Multiple Languages

Add support for multiple languages:

```rust
pub struct HelloWorldTool {
    greeting_template: String,
    language: String,
}

impl HelloWorldTool {
    pub fn new() -> Self {
        Self {
            greeting_template: "Hello, {name}!".to_string(),
            language: "en".to_string(),
        }
    }
}

#[async_trait]
impl Plugin for HelloWorldTool {
    async fn initialize(&mut self, config: &Value) -> Result<(), PluginError> {
        if let Some(template) = config.get("greeting_template").and_then(|v| v.as_str()) {
            self.greeting_template = template.to_string();
        }
        if let Some(lang) = config.get("language").and_then(|v| v.as_str()) {
            self.language = lang.to_string();
        }
        Ok(())
    }
}

#[async_trait]
impl Tool for HelloWorldTool {
    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name to greet"
                },
                "language": {
                    "type": "string",
                    "description": "Language code (en, es, fr, de)",
                    "default": "en"
                }
            },
            "required": ["name"]
        })
    }

    async fn execute(
        &self,
        params: Value,
        _sandbox_context: &SandboxContext,
    ) -> Result<Value, ToolError> {
        let name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidParameters("Missing 'name' parameter".to_string()))?;

        let language = params
            .get("language")
            .and_then(|v| v.as_str())
            .unwrap_or(&self.language);

        let greeting = match language {
            "es" => format!("¡Hola, {}!", name),
            "fr" => format!("Bonjour, {}!", name),
            "de" => format!("Hallo, {}!", name),
            _ => self.greeting_template.replace("{name}", name),
        };

        Ok(json!({
            "greeting": greeting,
            "language": language,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    }
}
```

## Summary

You've successfully created your first Wireframe-AI plugin! Here's what you learned:

1. **Plugin Structure**: How to structure a plugin with the Plugin trait
2. **Tool Implementation**: How to implement the Tool trait for sandbox tools
3. **Configuration**: How to make plugins configurable
4. **Testing**: How to write comprehensive tests for your plugin
5. **Integration**: How to integrate your plugin with the Wireframe-AI system
6. **Advanced Features**: How to add custom templates and multi-language support

## Next Steps

- Try creating plugins for other module types (Context, Orchestrator, Interface)
- Explore the [plugin templates](../../templates/plugins/) for more examples
- Read the [Plugin Development Guide](./Plugin-Development-Guide.md) for advanced topics
- Check out existing plugin implementations in the `plugins/` directory

## Common Issues

### Build Errors

If you encounter build errors, ensure:
- The SDK path in `Cargo.toml` is correct
- All dependencies are listed in `Cargo.toml`
- The workspace includes your plugin

### Plugin Not Found

If the plugin is not found:
- Check that the plugin is registered in the module
- Verify the `plugin_id` matches the configuration
- Ensure the plugin is built before running the module

### Test Failures

If tests fail:
- Check that all required parameters are provided
- Verify the input schema matches the test parameters
- Ensure the sandbox context is properly initialized

## Resources

- [Plugin Development Guide](./Plugin-Development-Guide.md)
- [API Reference](./API-Reference.md)
- [Configuration Examples](./Configuration-Examples.md)
- [Plugin Templates](../../templates/plugins/)
