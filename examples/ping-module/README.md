# Ping Module — Example Wireframe AI Module

Demonstrates the `#[module]` proc-macro — the simplest way to write a Wireframe AI module.

## What it does

- Subscribes to `ping.request`
- Publishes replies on `ping.response`
- Shows the full module lifecycle: announce online → heartbeat → handle → graceful shutdown

## Usage

```bash
# Terminal 1: Start NATS
nats-server

# Terminal 2: Start the ping module
cargo run -p ping-module

# Terminal 3: Send a ping
nats pub ping.request '{"message": "hello from the bus!"}'

# Terminal 4: Watch for pong
nats sub ping.response
```

## What this demonstrates

The entire module logic is ~20 lines. The `#[module]` macro generates:

```rust
#[module(
    subscribes = ["ping.request"],
    publishes  = ["ping.response"],
    queue_group = "ping_handler"
)]
impl Module for PingModule {
    async fn handle(&mut self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let msg = env.payload.get("message")...;
        vec![env.reply("ping.response", json!({"reply": format!("pong: {}", msg)}))]
    }
}
```

Everything else — NATS connection, `sys.module.online`, heartbeat, graceful shutdown — is generated code.
