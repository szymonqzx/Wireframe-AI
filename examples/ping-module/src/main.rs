//! Ping Module — example Wireframe AI module using the `#[module]` macro.
//!
//! Listens on "ping.request" and replies on "ping.response".
//! Demonstrates the full module lifecycle:
//!   - sys.module.online on startup
//!   - Periodic heartbeat
//!   - Graceful shutdown with sys.module.offline
//!   - Handle function with Envelope::reply()

use agentic_sdk::Module;

struct PingModule;

#[agentic_sdk::module(
    subscribes = ["ping.request"],
    publishes  = ["ping.response"],
    queue_group = "ping_handler"
)]
impl Module for PingModule {
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {
        let payload = &env.payload;
        let msg = payload
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("ping");

        tracing::info!(message = %msg, "received ping request");

        // Reply with a pong
        let response = serde_json::json!({
            "reply": format!("pong: {}", msg),
            "echo": payload,
        });

        vec![env.reply("ping.response", response)]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    PingModule.run("nats://localhost:4222").await
}
