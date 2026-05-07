//! Wireframe-AI Event Sourcing & Audit Log Module
//!
//! Captures all messages on the NATS bus and stores them as an immutable
//! event log in SQLite. Supports replay, audit queries, and time-travel debugging.
//! All events are persisted to disk for durability and compliance.
//!
//! Subscribes to: audit.query, audit.replay.request, task.>, agent.>, sys.>, webhook.>
//! Publishes to: audit.query.result, audit.replay.stream

use agentic_sdk::{Envelope, Module};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

const MAX_EVENTS: usize = 100_000;

struct EventSourcingModule {
    db: Arc<Mutex<Connection>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct EventRecord {
    id: String,
    topic: String,
    correlation_id: String,
    session_id: Option<String>,
    payload: Value,
    timestamp: i64,
    source_module: Option<String>,
}

const EVENT_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS events (
    id TEXT PRIMARY KEY,
    topic TEXT NOT NULL,
    correlation_id TEXT NOT NULL,
    session_id TEXT,
    payload TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    source_module TEXT
);

CREATE INDEX IF NOT EXISTS idx_events_topic ON events(topic);
CREATE INDEX IF NOT EXISTS idx_events_correlation ON events(correlation_id);
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
"#;

fn init_db(db_path: &str) -> Result<Connection, Box<dyn std::error::Error>> {
    let conn = Connection::open(db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch(EVENT_SCHEMA)?;
    tracing::info!(db = %db_path, "event sourcing database initialized");
    Ok(conn)
}

fn event_from_row(row: &rusqlite::Row) -> rusqlite::Result<EventRecord> {
    let payload_str: String = row.get(4)?;
    let payload = serde_json::from_str(&payload_str).unwrap_or_default();
    Ok(EventRecord {
        id: row.get(0)?,
        topic: row.get(1)?,
        correlation_id: row.get(2)?,
        session_id: row.get(3)?,
        payload,
        timestamp: row.get(5)?,
        source_module: row.get(6)?,
    })
}

#[agentic_sdk::module(
    subscribes = ["audit.query", "audit.replay.request", "task.>", "agent.>", "sys.>", "webhook.>"],
    publishes  = ["audit.query.result", "audit.replay.stream"],
    queue_group = "event_sourcing"
)]
impl Module for EventSourcingModule {
    async fn handle(&mut self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        match env.topic.as_str() {
            "audit.query" => self.handle_query(env).await,
            "audit.replay.request" => self.handle_replay_request(env).await,
            _ => {
                // Store all other messages as events
                self.store_event(&env).await;
                vec![]
            }
        }
    }
}

impl EventSourcingModule {
    async fn store_event(&self, env: &Envelope<Value>) {
        let record = EventRecord {
            id: uuid::Uuid::new_v4().to_string(),
            topic: env.topic.clone(),
            correlation_id: env.correlation_id.clone(),
            session_id: Some(env.session_id.clone()),
            payload: env.payload.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            source_module: env
                .payload
                .get("module_id")
                .and_then(|v| v.as_str())
                .map(String::from),
        };

        let db = self.db.lock().await;
        let payload_json = serde_json::to_string(&record.payload).unwrap_or_default();
        let session_id = record.session_id.clone().unwrap_or_default();
        let source_module = record.source_module.clone().unwrap_or_default();

        if let Err(e) = db.execute(
            "INSERT INTO events (id, topic, correlation_id, session_id, payload, timestamp, source_module)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                &record.id, &record.topic, &record.correlation_id,
                &session_id, &payload_json, record.timestamp, &source_module
            ],
        ) {
            tracing::error!(error = %e, "failed to store event");
            drop(db);
            return;
        }

        // Enforce retention limit: evict oldest events when limit exceeded.
        let count: i64 = db
            .query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))
            .unwrap_or(0);
        if count > MAX_EVENTS as i64 {
            let overflow = count - MAX_EVENTS as i64;
            if let Err(e) = db.execute(
                "DELETE FROM events WHERE id IN (
                    SELECT id FROM events ORDER BY timestamp ASC LIMIT ?1
                )",
                params![overflow],
            ) {
                tracing::warn!(error = %e, "failed to evict old events");
            } else {
                tracing::warn!(
                    evicted = overflow,
                    max = MAX_EVENTS,
                    "event store overflow; oldest events evicted"
                );
            }
        }
        drop(db);

        tracing::debug!(topic = %env.topic, id = %record.id, "event stored");
    }

    async fn handle_query(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let query = &env.payload;
        let topic_filter = query.get("topic").and_then(|v| v.as_str());
        let correlation_filter = query.get("correlation_id").and_then(|v| v.as_str());
        let since = query.get("since").and_then(|v| v.as_i64());
        let limit = query.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;

        let db = self.db.lock().await;
        let mut sql = String::from(
            "SELECT id, topic, correlation_id, session_id, payload, timestamp, source_module FROM events WHERE 1=1"
        );
        let mut sql_params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

        if let Some(topic) = topic_filter {
            sql.push_str(" AND (topic = ? OR topic LIKE ?)");
            let prefix = topic.replace(">", "%");
            sql_params.push(Box::new(topic.to_string()));
            sql_params.push(Box::new(prefix));
        }
        if let Some(corr) = correlation_filter {
            sql.push_str(" AND correlation_id = ?");
            sql_params.push(Box::new(corr.to_string()));
        }
        if let Some(s) = since {
            sql.push_str(" AND timestamp >= ?");
            sql_params.push(Box::new(s));
        }
        sql.push_str(" ORDER BY timestamp DESC LIMIT ?");
        sql_params.push(Box::new(limit as i64));

        let param_refs: Vec<&dyn rusqlite::ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = match db.prepare(&sql) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(error = %e, "failed to prepare query");
                return vec![env.reply(
                    "audit.query.result",
                    serde_json::json!({
                        "error": "query_failed",
                        "message": e.to_string(),
                    }),
                )];
            }
        };

        let rows = stmt.query_map(param_refs.as_slice(), event_from_row);
        let results: Vec<EventRecord> = match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(e) => {
                tracing::error!(error = %e, "failed to execute query");
                return vec![env.reply(
                    "audit.query.result",
                    serde_json::json!({
                        "error": "query_failed",
                        "message": e.to_string(),
                    }),
                )];
            }
        };

        let response = serde_json::json!({
            "count": results.len(),
            "events": results,
            "queried_at": chrono::Utc::now().timestamp(),
        });

        vec![env.reply("audit.query.result", response)]
    }

    async fn handle_replay_request(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let request = &env.payload;
        let from_timestamp = request.get("from").and_then(|v| v.as_i64()).unwrap_or(0);
        let to_timestamp = request
            .get("to")
            .and_then(|v| v.as_i64())
            .unwrap_or(i64::MAX);
        let topic_filter = request.get("topic").and_then(|v| v.as_str());

        let db = self.db.lock().await;
        let mut sql = String::from(
            "SELECT id, topic, correlation_id, session_id, payload, timestamp, source_module FROM events WHERE timestamp >= ? AND timestamp <= ?"
        );
        let mut sql_params: Vec<Box<dyn rusqlite::ToSql>> =
            vec![Box::new(from_timestamp), Box::new(to_timestamp)];

        if let Some(topic) = topic_filter {
            sql.push_str(" AND (topic = ? OR topic LIKE ?)");
            let prefix = topic.replace(">", "%");
            sql_params.push(Box::new(topic.to_string()));
            sql_params.push(Box::new(prefix));
        }
        sql.push_str(" ORDER BY timestamp ASC");

        let param_refs: Vec<&dyn rusqlite::ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = match db.prepare(&sql) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!(error = %e, "failed to prepare replay query");
                return vec![];
            }
        };

        let rows = stmt.query_map(param_refs.as_slice(), event_from_row);
        let replay_events: Vec<EventRecord> = match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
            Err(e) => {
                tracing::error!(error = %e, "failed to execute replay query");
                return vec![];
            }
        };

        let mut outputs = Vec::new();
        for record in replay_events {
            let replay_env = Envelope::new(
                &record.topic,
                record.payload.clone(),
                record.session_id.clone(),
            );
            outputs.push(Envelope::new(
                "audit.replay.stream",
                serde_json::json!({
                    "original_topic": record.topic,
                    "original_correlation_id": record.correlation_id,
                    "original_timestamp": record.timestamp,
                    "payload": replay_env.payload,
                }),
                record.session_id.clone(),
            ));
        }

        tracing::info!(count = outputs.len(), "replay events streamed");

        outputs
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let db_path = std::env::var("WIREFRAME_AI_EVENT_DB")
        .unwrap_or_else(|_| "wireframe_ai_events.db".to_string());
    let conn = init_db(&db_path)?;

    let module = EventSourcingModule {
        db: Arc::new(Mutex::new(conn)),
    };

    module.run("nats://localhost:4222").await
}
