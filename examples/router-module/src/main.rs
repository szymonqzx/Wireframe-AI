//! Router Module — Example Wireframe AI module.
//!
//! Routes messages based on rules. Subscribes to `route.request`
//! and forwards to the appropriate destination topic.
//!
//! ## Use case
//!
//! Dynamic message routing for load balancing, A/B testing, or
//! multi-tenant message isolation.

use agentic_sdk::{Envelope, Module};
use std::collections::HashMap;

struct RouterModule {
    rules: HashMap<String, String>, // source_topic -> destination_topic
}

impl Default for RouterModule {
    fn default() -> Self {
        let mut rules = HashMap::new();
        rules.insert("task.submitted".to_string(), "task.enriched".to_string());
        rules.insert("agent.result".to_string(), "task.complete".to_string());
        Self { rules }
    }
}

#[agentic_sdk::module(
    subscribes = ["route.request", "route.register", "route.unregister"],
    publishes  = ["route.response", "routed.message"],
    queue_group = "router"
)]
impl Module for RouterModule {
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {
        match env.topic.as_str() {
            "route.request" => {
                let source = env
                    .payload
                    .get("source_topic")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let payload = env.payload.get("payload").cloned().unwrap_or_default();

                let destination = self.rules.get(source).cloned();

                tracing::info!(source, ?destination, "routing message");

                if let Some(dest) = destination {
                    let routed = Envelope::new(&dest, payload, Some(env.session_id.clone()));
                    let response = serde_json::json!({
                        "status": "routed",
                        "source": source,
                        "destination": dest,
                        "timestamp": chrono::Utc::now().timestamp(),
                    });
                    vec![routed, env.reply("route.response", response)]
                } else {
                    let response = serde_json::json!({
                        "status": "no_route",
                        "source": source,
                        "timestamp": chrono::Utc::now().timestamp(),
                    });
                    vec![env.reply("route.response", response)]
                }
            }
            "route.register" => {
                let source = env
                    .payload
                    .get("source_topic")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let dest = env
                    .payload
                    .get("destination_topic")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                if !source.is_empty() && !dest.is_empty() {
                    self.rules.insert(source.clone(), dest.clone());
                    tracing::info!(source, dest, "route registered");
                }

                let response = serde_json::json!({
                    "status": "registered",
                    "routes": self.rules,
                    "timestamp": chrono::Utc::now().timestamp(),
                });
                vec![env.reply("route.response", response)]
            }
            "route.unregister" => {
                let source = env
                    .payload
                    .get("source_topic")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                self.rules.remove(&source);

                let response = serde_json::json!({
                    "status": "unregistered",
                    "routes": self.rules,
                    "timestamp": chrono::Utc::now().timestamp(),
                });
                vec![env.reply("route.response", response)]
            }
            _ => vec![],
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    RouterModule::default().run("nats://localhost:4222").await
}
