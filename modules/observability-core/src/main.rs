//! Wireframe-AI Observability Module
//!
//! Collects metrics, traces, and logs from all modules.
//! Provides dashboards, alerts, and health monitoring.
//! All data is persisted to SQLite for durability and historical analysis.
//!
//! Langfuse Integration:
//! The langfuse-sdk is available for direct instrumentation in modules.
//! To use Langfuse tracing in your module:
//! 1. Add langfuse-sdk = "0.1" to your Cargo.toml
//! 2. Set LANGFUSE_PUBLIC_KEY and LANGFUSE_SECRET_KEY environment variables
//! 3. Use the #[observe] macro or with_span() closure API
//!
//! Subscribes to: metrics.>, trace.>, log.>, health.check
//! Publishes to: metrics.aggregated, trace.span, health.status

use agentic_sdk::{Envelope, Module};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

struct ObservabilityModule {
    db: Arc<Mutex<Connection>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TraceSpan {
    trace_id: String,
    span_id: String,
    parent_id: Option<String>,
    name: String,
    start_time: i64,
    end_time: Option<i64>,
    attributes: std::collections::HashMap<String, Value>,
    status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct HealthSnapshot {
    module_id: String,
    status: String,
    last_seen: i64,
    latency_ms: u64,
    error_rate: f64,
}

const OBSERVABILITY_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS metrics (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    value REAL NOT NULL,
    labels TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_metrics_name_time ON metrics(name, timestamp);

CREATE TABLE IF NOT EXISTS traces (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    trace_id TEXT NOT NULL,
    span_id TEXT NOT NULL,
    parent_id TEXT,
    name TEXT NOT NULL,
    start_time INTEGER NOT NULL,
    end_time INTEGER,
    attributes TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_traces_trace_id ON traces(trace_id);

CREATE TABLE IF NOT EXISTS logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    level TEXT NOT NULL,
    module TEXT NOT NULL,
    message TEXT NOT NULL,
    correlation_id TEXT,
    attributes TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_logs_time ON logs(timestamp);
CREATE INDEX IF NOT EXISTS idx_logs_correlation ON logs(correlation_id);

CREATE TABLE IF NOT EXISTS health_snapshots (
    module_id TEXT PRIMARY KEY,
    status TEXT NOT NULL,
    last_seen INTEGER NOT NULL,
    latency_ms INTEGER NOT NULL,
    error_rate REAL NOT NULL
);
"#;

fn init_db(db_path: &str) -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch(OBSERVABILITY_SCHEMA)?;
    tracing::info!(db = %db_path, "observability database initialized");
    Ok(conn)
}

#[agentic_sdk::module(
    subscribes = ["metrics.>", "trace.>", "log.>", "health.check", "health.report"],
    publishes  = ["metrics.aggregated", "trace.span", "health.status"],
    queue_group = "observability"
)]
impl Module for ObservabilityModule {
    async fn handle(&mut self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        match env.topic.as_str() {
            t if t.starts_with("metrics.") && t != "metrics.aggregated" => {
                self.handle_metric(env).await
            }
            t if t.starts_with("trace.") && t != "trace.span" => self.handle_trace(env).await,
            t if t.starts_with("log.") => self.handle_log(env).await,
            "health.check" => self.handle_health_check(env).await,
            "health.report" => self.handle_health_report(env).await,
            _ => vec![],
        }
    }
}

impl ObservabilityModule {
    async fn handle_metric(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let payload = &env.payload;
        let name = payload
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let value = payload.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let labels = payload
            .get("labels")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect::<std::collections::HashMap<_, _>>()
            })
            .unwrap_or_default();

        let db = self.db.lock().await;
        let labels_json = serde_json::to_string(&labels).unwrap_or_default();
        let _ = db.execute(
            "INSERT INTO metrics (name, timestamp, value, labels) VALUES (?1, ?2, ?3, ?4)",
            params![name, chrono::Utc::now().timestamp(), value, labels_json],
        );
        drop(db);

        vec![]
    }

    async fn handle_trace(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let payload = &env.payload;
        let span: TraceSpan = match serde_json::from_value(payload.clone()) {
            Ok(s) => s,
            Err(_) => return vec![],
        };

        // Store in local SQLite
        let db = self.db.lock().await;
        let attributes_json = serde_json::to_string(&span.attributes).unwrap_or_default();
        let _ = db.execute(
            "INSERT INTO traces (trace_id, span_id, parent_id, name, start_time, end_time, attributes, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &span.trace_id,
                &span.span_id,
                span.parent_id.as_deref(),
                &span.name,
                span.start_time,
                span.end_time,
                attributes_json,
                &span.status
            ],
        );
        drop(db);

        // Note: Langfuse SDK is available for direct instrumentation in modules.
        // Traces received via NATS are stored locally. For Langfuse integration,
        // individual modules should use the langfuse-sdk directly with the
        // #[observe] macro or with_span() closure API.

        vec![Envelope::new(
            "trace.span",
            serde_json::to_value(&span).unwrap_or_default(),
            None,
        )]
    }

    async fn handle_log(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let payload = &env.payload;
        let timestamp = payload
            .get("timestamp")
            .and_then(|v| v.as_i64())
            .unwrap_or_else(|| chrono::Utc::now().timestamp());
        let level = payload
            .get("level")
            .and_then(|v| v.as_str())
            .unwrap_or("info")
            .to_string();
        let module_name = payload
            .get("module")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let message = payload
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let correlation_id = env.correlation_id.clone();
        let attributes: std::collections::HashMap<String, Value> = payload
            .as_object()
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        let db = self.db.lock().await;
        let attributes_json = serde_json::to_string(&attributes).unwrap_or_default();
        let _ = db.execute(
            "INSERT INTO logs (timestamp, level, module, message, correlation_id, attributes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                timestamp,
                &level,
                &module_name,
                &message,
                &correlation_id,
                attributes_json
            ],
        );
        drop(db);

        vec![]
    }

    async fn handle_health_check(&self, _env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let db = self.db.lock().await;
        let now = chrono::Utc::now().timestamp();

        let mut stmt = match db.prepare(
            "SELECT module_id, status, last_seen, latency_ms, error_rate FROM health_snapshots",
        ) {
            Ok(s) => s,
            Err(_) => return vec![],
        };

        let rows = stmt.query_map([], |row| {
            Ok(HealthSnapshot {
                module_id: row.get(0)?,
                status: row.get(1)?,
                last_seen: row.get(2)?,
                latency_ms: row.get::<_, i64>(3)? as u64,
                error_rate: row.get(4)?,
            })
        });

        let statuses: Vec<Value> = match rows {
            Ok(iter) => iter
                .filter_map(|r| {
                    let h = r.ok()?;
                    Some(serde_json::json!({
                        "module_id": h.module_id,
                        "status": h.status,
                        "last_seen_seconds_ago": now - h.last_seen,
                        "latency_ms": h.latency_ms,
                        "error_rate": h.error_rate,
                    }))
                })
                .collect(),
            Err(_) => vec![],
        };

        let overall = if statuses.is_empty() {
            "unknown"
        } else if statuses.iter().all(|s| s["status"] == "healthy") {
            "healthy"
        } else {
            "degraded"
        };

        vec![Envelope::new(
            "health.status",
            serde_json::json!({
                "overall": overall,
                "modules": statuses,
                "checked_at": now,
            }),
            None,
        )]
    }

    async fn handle_health_report(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let payload = &env.payload;
        let module_id = payload
            .get("module_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let status = payload
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let latency_ms = payload
            .get("latency_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let error_rate = payload
            .get("error_rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let db = self.db.lock().await;
        let _ = db.execute(
            "INSERT INTO health_snapshots (module_id, status, last_seen, latency_ms, error_rate)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(module_id) DO UPDATE SET
                 status = excluded.status,
                 last_seen = excluded.last_seen,
                 latency_ms = excluded.latency_ms,
                 error_rate = excluded.error_rate",
            params![
                module_id,
                status,
                chrono::Utc::now().timestamp(),
                latency_ms as i64,
                error_rate
            ],
        );
        drop(db);

        vec![]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Log Langfuse availability
    if std::env::var("LANGFUSE_PUBLIC_KEY").is_ok()
        && std::env::var("LANGFUSE_SECRET_KEY").is_ok()
    {
        tracing::info!("Langfuse credentials detected - modules can use langfuse-sdk for tracing");
    }

    let db_path = std::env::var("WIREFRAME_AI_OBSERVABILITY_DB")
        .unwrap_or_else(|_| "wireframe_ai_observability.db".to_string());
    let conn = init_db(&db_path)?;

    let module = ObservabilityModule {
        db: Arc::new(Mutex::new(conn)),
    };

    module.run("nats://localhost:4222").await
}
