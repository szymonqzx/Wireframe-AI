//! Metrics Module — Example Wireframe AI module.
//!
//! Collects message flow metrics: counts by topic, latency tracking,
//! and publishes aggregated metrics on a timer.
//!
//! ## Use case
//!
//! Deploy this module to monitor system health and message throughput.
//! Query metrics via `metrics.snapshot` request topic.

use agentic_sdk::{Envelope, Module};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

#[derive(Default, Debug, Clone)]
struct MetricsState {
    message_count_by_topic: HashMap<String, u64>,
    total_messages: u64,
    start_time: i64,
}

struct MetricsModule {
    state: Arc<Mutex<MetricsState>>,
}

impl MetricsModule {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MetricsState {
                start_time: chrono::Utc::now().timestamp(),
                ..Default::default()
            })),
        }
    }

    fn start_snapshot_publisher(&self, client: async_nats::Client) {
        let state = self.state.clone();
        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(60));
            loop {
                ticker.tick().await;
                let snapshot = {
                    let s = state.lock().await;
                    serde_json::json!({
                        "total_messages": s.total_messages,
                        "by_topic": s.message_count_by_topic,
                        "uptime_seconds": chrono::Utc::now().timestamp() - s.start_time,
                        "timestamp": chrono::Utc::now().timestamp(),
                    })
                };
                let env = Envelope::new("metrics.snapshot", snapshot, None);
                if let Ok(data) = serde_json::to_vec(&env) {
                    let _ = client.publish("metrics.snapshot", data.into()).await;
                }
            }
        });
    }
}

#[agentic_sdk::module(
    subscribes = ["task.>", "agent.>", "sys.>", "metrics.query"],
    publishes  = ["metrics.snapshot", "metrics.query.response"],
    queue_group = "metrics"
)]
impl Module for MetricsModule {
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {
        let mut state = self.state.lock().await;

        if env.topic == "metrics.query" {
            let snapshot = serde_json::json!({
                "total_messages": state.total_messages,
                "by_topic": state.message_count_by_topic,
                "uptime_seconds": chrono::Utc::now().timestamp() - state.start_time,
            });
            return vec![env.reply("metrics.query.response", snapshot)];
        }

        state.total_messages += 1;
        *state
            .message_count_by_topic
            .entry(env.topic.clone())
            .or_insert(0) += 1;

        tracing::debug!(topic = %env.topic, total = state.total_messages, "metric recorded");
        vec![]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let module = MetricsModule::new();

    let client = async_nats::connect("nats://localhost:4222").await?;
    agentic_sdk::announce_online(
        &client,
        "metrics-module",
        "0.1.0",
        &["task.>", "agent.>", "sys.>", "metrics.query"],
        &["metrics.snapshot", "metrics.query.response"],
    )
    .await?;

    module.start_snapshot_publisher(client.clone());

    // Run module manually since we need pre-run setup
    let state = module.state.clone();
    let mut sub = client
        .queue_subscribe("task.>", "metrics".to_string())
        .await?;
    let mut sub2 = client
        .queue_subscribe("agent.>", "metrics".to_string())
        .await?;
    let mut sub3 = client
        .queue_subscribe("sys.>", "metrics".to_string())
        .await?;
    let mut sub4 = client
        .queue_subscribe("metrics.query", "metrics".to_string())
        .await?;

    use futures::StreamExt;

    loop {
        tokio::select! {
            msg = sub.next() => {
                match msg {
                    Some(msg) => {
                        if let Ok(env) = serde_json::from_slice::<Envelope<serde_json::Value>>(&msg.payload) {
                            let mut s = state.lock().await;
                            s.total_messages += 1;
                            *s.message_count_by_topic.entry(env.topic.clone()).or_insert(0) += 1;
                        }
                    }
                    None => break,
                }
            }
            msg = sub2.next() => {
                match msg {
                    Some(msg) => {
                        if let Ok(env) = serde_json::from_slice::<Envelope<serde_json::Value>>(&msg.payload) {
                            let mut s = state.lock().await;
                            s.total_messages += 1;
                            *s.message_count_by_topic.entry(env.topic.clone()).or_insert(0) += 1;
                        }
                    }
                    None => break,
                }
            }
            msg = sub3.next() => {
                match msg {
                    Some(msg) => {
                        if let Ok(env) = serde_json::from_slice::<Envelope<serde_json::Value>>(&msg.payload) {
                            let mut s = state.lock().await;
                            s.total_messages += 1;
                            *s.message_count_by_topic.entry(env.topic.clone()).or_insert(0) += 1;
                        }
                    }
                    None => break,
                }
            }
            msg = sub4.next() => {
                match msg {
                    Some(msg) => {
                        if let Ok(env) = serde_json::from_slice::<Envelope<serde_json::Value>>(&msg.payload) {
                            let snapshot = {
                                let s = state.lock().await;
                                serde_json::json!({
                                    "total_messages": s.total_messages,
                                    "by_topic": s.message_count_by_topic.clone(),
                                    "uptime_seconds": chrono::Utc::now().timestamp() - s.start_time,
                                })
                            };
                            let reply = env.reply("metrics.query.response", snapshot);
                            if let Ok(data) = serde_json::to_vec(&reply) {
                                let _ = client.publish(reply.topic, data.into()).await;
                            }
                        }
                    }
                    None => break,
                }
            }
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("shutting down metrics-module");
                break;
            }
        }
    }

    agentic_sdk::announce_offline(&client, "metrics-module", "0.1.0")
        .await
        .ok();
    Ok(())
}
