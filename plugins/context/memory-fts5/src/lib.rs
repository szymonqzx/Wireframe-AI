use agentic_sdk::message_types::MemoryChunk;
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::context::{MemoryBackend, MemoryError};
use async_trait::async_trait;
use rusqlite::Connection;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tracing::warn;

const FTS5_SCHEMA: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS memory_fts USING fts5(
    session_id UNINDEXED,
    content,
    role UNINDEXED,
    tokenize='porter unicode61'
);

CREATE TABLE IF NOT EXISTS memory_chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    content TEXT NOT NULL,
    source TEXT NOT NULL DEFAULT 'conversation',
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_memory_chunks_session
    ON memory_chunks(session_id, created_at);
"#;

const FTS5_RELEVANCE_SCORE_MAX: f64 = 100.0;

pub struct FTS5MemoryPlugin {
    db_path: String,
    conn: Arc<Mutex<Connection>>,
}

impl FTS5MemoryPlugin {
    pub fn new(db_path: String) -> Self {
        let conn = Connection::open(&db_path).expect("Failed to open database");
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .expect("Failed to set WAL mode");
        conn.execute_batch(FTS5_SCHEMA)
            .expect("Failed to create FTS5 tables");
        Self {
            db_path,
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    fn validate_session_id(&self, session_id: &str) -> Result<String, MemoryError> {
        if session_id.len() > 256 {
            return Err(MemoryError::SearchFailed("Session ID too long".to_string()));
        }
        if session_id.is_empty() {
            return Err(MemoryError::SearchFailed(
                "Session ID cannot be empty".to_string(),
            ));
        }
        if !session_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(MemoryError::SearchFailed(
                "Invalid session ID characters".to_string(),
            ));
        }
        Ok(session_id.to_string())
    }

    fn sanitize_query(&self, query: &str) -> String {
        query
            .chars()
            .filter(|c| !c.is_control() || *c == ' ' || *c == '\t' || *c == '\n')
            .collect()
    }

    fn validate_query(&self, query: &str) -> Result<String, MemoryError> {
        if query.trim().is_empty() {
            return Err(MemoryError::SearchFailed(
                "Query cannot be empty".to_string(),
            ));
        }
        if query.len() > 1000 {
            return Err(MemoryError::SearchFailed("Query too long".to_string()));
        }
        let sanitized = self.sanitize_query(query);
        let cleaned: String = sanitized
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-')
            .collect();
        if cleaned.trim().is_empty() {
            return Err(MemoryError::SearchFailed(
                "Query contains no valid characters".to_string(),
            ));
        }
        Ok(cleaned)
    }
}

#[async_trait]
impl Plugin for FTS5MemoryPlugin {
    fn plugin_id(&self) -> &'static str {
        "memory-fts5"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "FTS5 full-text search memory backend"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        if let Some(path) = config.get("db_path").and_then(|v| v.as_str()) {
            self.db_path = path.to_string();
            let conn = Connection::open(&self.db_path).map_err(|e| {
                agentic_sdk::plugin::PluginError::InitializationFailed(e.to_string())
            })?;
            conn.execute_batch("PRAGMA journal_mode=WAL;")
                .map_err(|e| {
                    agentic_sdk::plugin::PluginError::InitializationFailed(e.to_string())
                })?;
            conn.execute_batch(FTS5_SCHEMA).map_err(|e| {
                agentic_sdk::plugin::PluginError::InitializationFailed(e.to_string())
            })?;
            self.conn = Arc::new(Mutex::new(conn));
        }
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, agentic_sdk::plugin::PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }
}

#[async_trait]
impl MemoryBackend for FTS5MemoryPlugin {
    async fn search<'a>(
        &'a self,
        query: &'a str,
        _session_id: &'a str,
        limit: usize,
    ) -> Result<Vec<MemoryChunk>, MemoryError> {
        let validated = match self.validate_query(query) {
            Ok(q) => q,
            Err(e) => {
                warn!(error = %e, query = %query, "Invalid search query");
                return Ok(Vec::new());
            }
        };

        let conn = self
            .conn
            .lock()
            .map_err(|e| MemoryError::SearchFailed(e.to_string()))?;

        let fts_query: String = validated
            .split_whitespace()
            .filter(|word| !word.is_empty())
            .map(|word| {
                let escaped = word.replace('"', "\"\"");
                format!("\"{}\"*", escaped)
            })
            .collect::<Vec<_>>()
            .join(" OR ");

        if fts_query.is_empty() {
            return Ok(Vec::new());
        }

        let mut stmt = conn
            .prepare(
                "SELECT m.session_id, m.content, m.role, m.rowid, rank
             FROM memory_fts m
             WHERE memory_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
            )
            .map_err(|e| MemoryError::SearchFailed(e.to_string()))?;

        let chunks = stmt
            .query_map(rusqlite::params![fts_query, limit as i64], |row| {
                let session_id: String = row.get(0)?;
                let content: String = row.get(1)?;
                let role: String = row.get(2)?;
                let msg_id: i64 = row.get(3)?;
                let rank: f64 = row.get(4)?;

                Ok(MemoryChunk {
                    id: format!("fts_{}", msg_id),
                    content,
                    source: format!("{} (session: {})", role, session_id),
                    relevance_score: (1.0 - (rank / FTS5_RELEVANCE_SCORE_MAX)).max(0.0) as f32,
                })
            })
            .map_err(|e| MemoryError::SearchFailed(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| MemoryError::SearchFailed(e.to_string()))?;

        Ok(chunks)
    }

    async fn persist_chunk<'a>(
        &'a self,
        session_id: &'a str,
        content: &'a str,
        source: &'a str,
    ) -> Result<(), MemoryError> {
        let validated = self.validate_session_id(session_id)?;
        let conn = self
            .conn
            .lock()
            .map_err(|e| MemoryError::PersistenceFailed(e.to_string()))?;
        conn.execute(
            "INSERT INTO memory_chunks (session_id, content, source, created_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![validated, content, source, chrono::Utc::now().timestamp()],
        ).map_err(|e| MemoryError::PersistenceFailed(e.to_string()))?;
        Ok(())
    }

    async fn load_chunks<'a>(
        &'a self,
        session_id: &'a str,
        limit: usize,
    ) -> Result<Vec<MemoryChunk>, MemoryError> {
        let validated = self.validate_session_id(session_id)?;
        let conn = self
            .conn
            .lock()
            .map_err(|e| MemoryError::SearchFailed(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, content, source, created_at FROM memory_chunks
             WHERE session_id = ?1
             ORDER BY created_at DESC
             LIMIT ?2",
            )
            .map_err(|e| MemoryError::SearchFailed(e.to_string()))?;
        let chunks = stmt
            .query_map(rusqlite::params![validated, limit as i64], |row| {
                let id: i64 = row.get(0)?;
                Ok(MemoryChunk {
                    id: format!("persistent_{}", id),
                    content: row.get(1)?,
                    source: row.get(2)?,
                    relevance_score: 1.0,
                })
            })
            .map_err(|e| MemoryError::SearchFailed(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| MemoryError::SearchFailed(e.to_string()))?;
        Ok(chunks)
    }
}
