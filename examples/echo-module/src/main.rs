//! Echo Module — Example Wireframe AI module.
//!
//! Subscribes to `echo.request` and replies with the same payload.
//! Demonstrates the request/response pattern.

use agentic_sdk::Module;

struct EchoModule;

#[agentic_sdk::module(
    subscribes = ["echo.request"],
    publishes  = ["echo.response"],
    queue_group = "echo_handler"
)]
impl Module for EchoModule {
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {
        let message = env
            .payload
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("no message");

        tracing::info!(message, "echoing message");

        let response = serde_json::json!({
            "echo": message,
            "original_topic": env.topic,
            "timestamp": chrono::Utc::now().timestamp(),
        });

        vec![env.reply("echo.response", response)]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    EchoModule.run("nats://localhost:4222").await
}
