//! Webhook Receiver Module — Example Wireframe AI module.
//!
//! Subscribes to `webhook.incoming` and normalizes external webhook payloads
//! into the internal event format. Publishes normalized events to `event.normalized`.
//!
//! ## Use case
//!
//! Bridge external systems (GitHub, Slack, etc.) into the Wireframe AI message bus.

use agentic_sdk::{Envelope, Module};

struct WebhookReceiver;

#[agentic_sdk::module(
    subscribes = ["webhook.incoming"],
    publishes  = ["event.normalized", "webhook.received"],
    queue_group = "webhook_handler"
)]
impl Module for WebhookReceiver {
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {
        let request = &env.payload;
        let source = request
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let event_type = request
            .get("event_type")
            .and_then(|v| v.as_str())
            .unwrap_or("generic");

        tracing::info!(source, event_type, "webhook received");

        let normalized = serde_json::json!({
            "source": source,
            "event_type": event_type,
            "original_payload": request.get("payload").cloned(),
            "received_at": chrono::Utc::now().timestamp(),
            "webhook_id": uuid::Uuid::new_v4().to_string(),
        });

        let ack = serde_json::json!({
            "webhook_id": normalized["webhook_id"],
            "status": "received",
            "source": source,
        });

        vec![
            Envelope::new("event.normalized", normalized, None),
            env.reply("webhook.received", ack),
        ]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    WebhookReceiver.run("nats://localhost:4222").await
}
