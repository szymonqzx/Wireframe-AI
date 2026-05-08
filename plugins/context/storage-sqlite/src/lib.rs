use agentic_sdk::message_types::ChatMessage;
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::context::{StorageBackend, StorageError};
use async_trait::async_trait;
use rusqlite::Connection;
use serde_json::Value;
use std::sync::{Arc, Mutex};

const DATABASE_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS sessions (
    session_id TEXT PRIMARY KEY,
    created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS chat_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    FOREIGN KEY (session_id) REFERENCES sessions(session_id)
);

CREATE INDEX IF NOT EXISTS idx_chat_messages_session
    ON chat_messages(session_id, timestamp);
"#;

pub struct SQLiteStoragePlugin {
    db_path: String,
    conn: Arc<Mutex<Connection>>,
}

impl SQLiteStoragePlugin {
    pub fn new(db_path: String) -> Self {
        let conn = Connection::open(&db_path).expect("Failed to open database");
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .expect("Failed to set WAL mode");
        conn.execute_batch(DATABASE_SCHEMA)
            .expect("Failed to create tables");
        Self {
            db_path,
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    fn validate_session_id(&self, session_id: &str) -> Result<String, StorageError> {
        if session_id.len() > 256 {
            return Err(StorageError::DatabaseError(
                "Session ID too long".to_string(),
            ));
        }
        if session_id.is_empty() {
            return Err(StorageError::DatabaseError(
                "Session ID cannot be empty".to_string(),
            ));
        }
        if !session_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(StorageError::DatabaseError(
                "Invalid session ID characters".to_string(),
            ));
        }
        Ok(session_id.to_string())
    }
}

#[async_trait]
impl Plugin for SQLiteStoragePlugin {
    fn plugin_id(&self) -> &'static str {
        "storage-sqlite"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "SQLite storage backend for sessions and messages"
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
            conn.execute_batch(DATABASE_SCHEMA).map_err(|e| {
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
impl StorageBackend for SQLiteStoragePlugin {
    async fn ensure_session<'a>(&'a self, session_id: &'a str) -> Result<(), StorageError> {
        let validated = self.validate_session_id(session_id)?;
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        conn.execute(
            "INSERT OR IGNORE INTO sessions (session_id, created_at) VALUES (?1, ?2)",
            rusqlite::params![validated, chrono::Utc::now().timestamp()],
        )
        .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn store_message<'a>(
        &'a self,
        session_id: &'a str,
        role: &'a str,
        content: &'a str,
    ) -> Result<(), StorageError> {
        let validated = self.validate_session_id(session_id)?;
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        conn.execute(
            "INSERT INTO chat_messages (session_id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![validated, role, content, chrono::Utc::now().timestamp()],
        ).map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    async fn load_session_history<'a>(
        &'a self,
        session_id: &'a str,
        limit: usize,
    ) -> Result<Vec<ChatMessage>, StorageError> {
        let validated = self.validate_session_id(session_id)?;
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT role, content, timestamp FROM chat_messages
             WHERE session_id = ?1
             ORDER BY timestamp DESC
             LIMIT ?2",
            )
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        let messages = stmt
            .query_map(rusqlite::params![validated, limit as i64], |row| {
                Ok(ChatMessage {
                    role: row.get(0)?,
                    content: row.get(1)?,
                    timestamp: row.get(2)?,
                })
            })
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?
            .into_iter()
            .rev()
            .collect();
        Ok(messages)
    }
}
