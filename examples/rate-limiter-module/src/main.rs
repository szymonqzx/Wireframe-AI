//! Rate Limiter Module — Example Wireframe AI module.
//!
//! Tracks request rates per session and enforces quotas.
//! Subscribes to `rate.check` and publishes `rate.result`.
//!
//! ## Use case
//!
//! Protect downstream services from overload and implement
//! fair-use policies across sessions.

use agentic_sdk::Module;
use std::collections::HashMap;

#[derive(Default)]
struct RateLimiterState {
    requests_per_window: HashMap<String, Vec<i64>>, // session_id -> timestamps
}

struct RateLimiterModule {
    max_requests: u64,
    window_seconds: i64,
    state: std::sync::Arc<tokio::sync::Mutex<RateLimiterState>>,
}

impl Default for RateLimiterModule {
    fn default() -> Self {
        Self {
            max_requests: 60,
            window_seconds: 60,
            state: std::sync::Arc::new(tokio::sync::Mutex::new(RateLimiterState::default())),
        }
    }
}

#[agentic_sdk::module(
    subscribes = ["rate.check", "rate.configure"],
    publishes  = ["rate.result", "rate.configured"],
    queue_group = "rate_limiter"
)]
impl Module for RateLimiterModule {
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {
        let now = chrono::Utc::now().timestamp();

        match env.topic.as_str() {
            "rate.check" => {
                let session_id = env
                    .payload
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&env.session_id)
                    .to_string();
                let cost = env
                    .payload
                    .get("cost")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(1);

                let mut state = self.state.lock().await;
                let window_start = now - self.window_seconds;

                let timestamps = state
                    .requests_per_window
                    .entry(session_id.clone())
                    .or_default();
                timestamps.retain(|&ts| ts > window_start);

                let current_count = timestamps.len() as u64;
                let allowed = current_count.saturating_add(cost) <= self.max_requests;

                if allowed {
                    for _ in 0..cost {
                        timestamps.push(now);
                    }
                }

                tracing::info!(
                    session = %session_id,
                    current = current_count,
                    allowed,
                    "rate check"
                );

                let result = serde_json::json!({
                    "session_id": session_id,
                    "allowed": allowed,
                    "current_requests": current_count,
                    "max_requests": self.max_requests,
                    "window_seconds": self.window_seconds,
                    "remaining": if allowed { self.max_requests.saturating_sub(current_count).saturating_sub(cost) } else { 0 },
                    "timestamp": now,
                });

                vec![env.reply("rate.result", result)]
            }
            "rate.configure" => {
                if let Some(max) = env.payload.get("max_requests").and_then(|v| v.as_u64()) {
                    self.max_requests = max;
                }
                if let Some(window) = env.payload.get("window_seconds").and_then(|v| v.as_i64()) {
                    self.window_seconds = window;
                }

                tracing::info!(
                    max = self.max_requests,
                    window = self.window_seconds,
                    "rate limiter reconfigured"
                );

                let result = serde_json::json!({
                    "status": "configured",
                    "max_requests": self.max_requests,
                    "window_seconds": self.window_seconds,
                    "timestamp": now,
                });

                vec![env.reply("rate.configured", result)]
            }
            _ => vec![],
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    RateLimiterModule::default()
        .run("nats://localhost:4222")
        .await
}
