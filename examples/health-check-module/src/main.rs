//! Health Check Module — Example Wireframe AI module.
//!
//! Monitors system health by checking NATS connectivity,
//! module heartbeats, and publishing health status.
//!
//! ## Use case
//!
//! Deploy alongside your agent cluster to monitor uptime and
//! detect module failures.

use agentic_sdk::{Envelope, Module};
use std::collections::HashMap;

#[derive(Default)]
struct HealthCheckModule {
    last_heartbeat: HashMap<String, i64>, // module_id -> last_seen
    start_time: i64,
}

#[agentic_sdk::module(
    subscribes = ["sys.module.heartbeat", "sys.module.online", "sys.module.offline", "health.query"],
    publishes  = ["health.status", "health.alert"],
    queue_group = "health_checker"
)]
impl Module for HealthCheckModule {
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {
        let now = chrono::Utc::now().timestamp();

        match env.topic.as_str() {
            "sys.module.heartbeat" => {
                if let Some(module_id) = env.payload.get("module_id").and_then(|v| v.as_str()) {
                    self.last_heartbeat.insert(module_id.to_string(), now);
                    tracing::debug!(module = module_id, "heartbeat received");
                }
                vec![]
            }
            "sys.module.online" => {
                if let Some(module_id) = env.payload.get("module_id").and_then(|v| v.as_str()) {
                    self.last_heartbeat.insert(module_id.to_string(), now);
                    tracing::info!(module = module_id, "module came online");
                }
                vec![]
            }
            "sys.module.offline" => {
                if let Some(module_id) = env.payload.get("module_id").and_then(|v| v.as_str()) {
                    self.last_heartbeat.remove(module_id);
                    tracing::warn!(module = module_id, "module went offline");

                    let alert = serde_json::json!({
                        "alert_type": "module_offline",
                        "module_id": module_id,
                        "timestamp": now,
                        "message": format!("Module {} has gone offline", module_id),
                    });
                    return vec![Envelope::new("health.alert", alert, None)];
                }
                vec![]
            }
            "health.query" => {
                let uptime_secs = now - self.start_time;
                let mut modules = Vec::new();

                for (module_id, last_seen) in &self.last_heartbeat {
                    let stale = now - last_seen > 90; // 90s timeout
                    modules.push(serde_json::json!({
                        "module_id": module_id,
                        "last_heartbeat": last_seen,
                        "stale": stale,
                        "seconds_since_heartbeat": now - last_seen,
                    }));
                }

                let stale_count = modules
                    .iter()
                    .filter(|m| m["stale"].as_bool().unwrap_or(false))
                    .count();
                let overall = if stale_count > 0 {
                    "degraded"
                } else {
                    "healthy"
                };

                let status = serde_json::json!({
                    "overall": overall,
                    "uptime_seconds": uptime_secs,
                    "modules_tracked": modules.len(),
                    "stale_modules": stale_count,
                    "modules": modules,
                    "timestamp": now,
                });

                vec![env.reply("health.status", status)]
            }
            _ => vec![],
        }
    }
}

impl HealthCheckModule {
    fn new() -> Self {
        Self {
            start_time: chrono::Utc::now().timestamp(),
            ..Default::default()
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    HealthCheckModule::new().run("nats://localhost:4222").await
}
