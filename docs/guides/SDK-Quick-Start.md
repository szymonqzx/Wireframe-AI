# Wireframe-AI SDK Quick Start

Get a working module running in under 30 minutes.

## Prerequisites

- Rust 1.80+ (`rustup update`)
- NATS server (`docker run -p 4222:4222 nats:latest`)
- `cargo install cargo-watch` (optional, for hot reload)

## Step 1: Scaffold a Module

```bash
# Build the CLI
cargo build --release -p wireframe-cli

# Create a new module
cargo run --release -p wireframe-cli -- new my-module --template basic
cd my-module
```

## Step 2: Understand the Structure

```
my-module/
  Cargo.toml          # SDK dependency
  src/main.rs         # Module implementation
  README.md
```

The `src/main.rs` contains a struct implementing the `Module` trait:

```rust
use agentic_sdk::{Envelope, Module};

struct MyModule;

#[agentic_sdk::module(
    subscribes = ["example.topic"],
    publishes  = ["example.response"],
    queue_group = "example_handler"
)]
impl Module for MyModule {
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {
        let response = serde_json::json!({
            "echo": env.payload,
            "timestamp": chrono::Utc::now().timestamp(),
        });
        vec![env.reply("example.response", response)]
    }
}
```

## Step 3: Run It

```bash
# Terminal 1: Start NATS
docker run -p 4222:4222 nats:latest

# Terminal 2: Run the module
cargo run
```

## Step 4: Test with NATS CLI

```bash
# Publish a message
nats pub example.topic '{"hello":"world"}'

# Subscribe to responses
nats sub example.response
```

## Templates Available

| Template | Description |
|---|---|
| `basic` | Minimal module |
| `adapter` | Reasoning adapter (agent.job -> agent.result) |
| `context` | Context enrichment |
| `orchestrator` | Task orchestration with fan-out/fan-in |
| `listener` | Log all messages |
| `service` | Request/response handler |
| `webhook` | Webhook receiver |
| `integration` | External API bridge |
| `cache` | TTL-based caching |
| `rate-limiter` | Token-bucket rate limiting |

## Advanced: Using Orchestrator Patterns

```rust
use agentic_sdk::orchestrator_patterns::{fan_out, fan_in};

// Split a task into parallel jobs
let jobs = fan_out(&enriched, vec!["subtask A".into(), "subtask B".into()]);

// Later, aggregate results
let complete = fan_in("session_1", "corr_1", &results);
```

## Advanced: Tool Composition

```rust
use agentic_sdk::reasoning::{ComposedTool, ToolChainExecutor};

let tool = ComposedTool::new("search_and_summarize", "Search then summarize")
    .step("search", "search_result")
    .map_input("query", "user_input")
    .step("summarize", "summary")
    .map_input("text", "search_result");

let mut executor = ToolChainExecutor::new();
executor.set_initial("user_input", serde_json::json!("AI safety"));
let result = executor.execute_with(&tool, |_tool_name, inputs| {
    serde_json::json!({ "tool": _tool_name, "inputs": inputs, "mock": true })
});
```

## Debugging

```bash
# Inspect all messages
wireframe debug --topic >

# Filter by correlation
wireframe debug --correlation abc-123 --capture debug.jsonl

# Replay captured messages
wireframe replay debug.jsonl --speed 2.0
```

## Next Steps

- Read `docs/Project-Architecture.md` for system overview
- Read `docs/Provider-System.md` to add LLM providers
- Check `examples/` for working implementations
