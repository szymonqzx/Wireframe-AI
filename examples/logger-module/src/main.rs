//! Logger Module — Example Wireframe AI module.
//!
//! Subscribes to all topics (task.>, agent.>, sys.>) and logs every message
//! with structured fields. Demonstrates wildcard topic subscriptions.
//!
//! ## Use case
//!
//! Deploy this module when you need full observability into the message bus.
//! Messages are logged at INFO level with topic, correlation_id, and session_id.

use agentic_sdk::Module;

struct LoggerModule;

#[agentic_sdk::module(
    subscribes = ["task.>", "agent.>", "sys.>"],
    publishes  = [],
    queue_group = "logger"
)]
impl Module for LoggerModule {
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {
        let payload_preview = match serde_json::to_string(&env.payload) {
            Ok(s) if s.len() > 200 => format!("{}…", &s[..200]),
            Ok(s) => s,
            Err(_) => "<invalid json>".to_string(),
        };

        tracing::info!(
            topic = %env.topic,
            correlation_id = %env.correlation_id,
            session_id = %env.session_id,
            message_id = %env.message_id,
            schema_version = env.schema_version,
            payload = %payload_preview,
            "message logged"
        );

        vec![]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    LoggerModule.run("nats://localhost:4222").await
}
