# agentic-sdk — Wireframe AI Rust SDK

The official SDK for building Wireframe AI modules. Provides envelopes, message types, the `Module` trait, NATS helpers, type-safe builders, and error handling patterns.

## Quick Start

### 1. Add dependency

```toml
[dependencies]
agentic-sdk = { path = "../sdk/agentic-sdk" }
agentic-sdk-macros = { path = "../sdk/agentic-sdk-macros" }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
```

### 2. Scaffold a module

```bash
cargo run --bin wireframe -- new my-module
```

### 3. Implement your logic

```rust
use agentic_sdk::{Envelope, Module};

struct MyModule;

#[agentic_sdk::module(
    subscribes = ["task.submitted"],
    publishes  = ["task.enriched"],
    queue_group = "task_handler"
)]
impl Module for MyModule {
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {
        tracing::info!(topic = %env.topic, "received message");
        vec![env.reply("task.enriched", serde_json::json!({"status": "ok"}))]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    MyModule.run("nats://localhost:4222").await
}
```

### 4. Run it

```bash
cargo run
```

## Core Components

### Envelope

The universal message wrapper every module uses:

```rust
use agentic_sdk::Envelope;

let env = Envelope::new("task.submitted", payload, Some("session_123".to_string()));
let child = env.child("task.enriched", payload, 1);
let reply = env.reply("task.complete", result_payload);
```

### Type-Safe Builders

Construct messages with compile-time required field checking:

```rust
use agentic_sdk::{TaskSubmittedBuilder, TaskCompleteBuilder, AgentJobBuilder};

let submitted = TaskSubmittedBuilder::new()
    .session_id("session_123")
    .user_input("Hello, world!")
    .build_envelope()?;

let complete = TaskCompleteBuilder::new()
    .session_id("session_123")
    .correlation_id("corr_456")
    .result("Done!")
    .side_effect("file_written", "output.txt")
    .build_envelope()?;
```

### Error Handling

Standardized errors with retry logic:

```rust
use agentic_sdk::error::{SdkError, SdkResult, retry_with_backoff, with_timeout};

async fn fetch_data() -> SdkResult<String> {
    retry_with_backoff(3, || async {
        // operation that might fail transiently
        Ok("data".to_string())
    }).await
}

async fn timed_operation() -> SdkResult<u32> {
    with_timeout(30, || async {
        Ok(42)
    }).await
}
```

### Module Registry

Track installed modules dynamically:

```rust
use agentic_sdk::ModuleRegistry;

let mut registry = ModuleRegistry::new();
registry.register_module(metadata)?;
let adapters = registry.list_modules_by_type("adapter");
```

### Compatibility Checking

Verify module interfaces before switching:

```rust
use agentic_sdk::CompatibilityChecker;

let checker = CompatibilityChecker::new();
let result = checker.check_compatibility(&current, &new_module);
assert!(result.is_compatible);
```

## Plugin System

The SDK includes a universal plugin system for runtime module composition. See [PLUGIN_SYSTEM.md](PLUGIN_SYSTEM.md) for details.

### Plugin Traits

- Base `Plugin` trait with lifecycle management
- Module-specific traits for Context, Orchestrator, Sandbox, Interface, and Adapter
- Plugin registry for dynamic plugin management
- Configuration loading from YAML/JSON
- Pipeline orchestration for ordered execution

## Topic Naming Convention

- `namespace.noun.verb` or `namespace.noun`
- Lowercase, dot-separated
- Examples: `task.submitted`, `agent.job`, `sys.module.online`

## Module Identity

Every module must announce itself on startup:

```rust
agentic_sdk::announce_online(
    &client,
    "my-module",
    "0.1.0",
    &["task.submitted"],
    &["task.enriched"],
).await?;
```

And announce shutdown:

```rust
agentic_sdk::announce_offline(&client, "my-module", "0.1.0").await?;
```

## Examples

See the `examples/` directory in the workspace:

- `ping-module` — Basic request/response
- `echo-module` — Echo service
- `logger-module` — Message logging with wildcard subscriptions
- `metrics-module` — Metrics collection and aggregation
- `validator-module` — Schema validation service
- `webhook-receiver` — External webhook ingestion
- `file-watcher-module` — File system monitoring
- `cache-module` — Key-value cache service
- `router-module` — Dynamic message routing
- `rate-limiter-module` — Quota enforcement
- `health-check-module` — Module health monitoring

## License

MIT
