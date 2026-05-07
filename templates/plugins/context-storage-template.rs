//! Template for a Context Storage Backend Plugin
//!
//! This template provides a starting point for implementing a custom
//! storage backend for the Wireframe-AI Context module.
//!
//! To use this template:
//! 1. Copy this file to your plugin directory
//! 2. Replace "MyStorage" with your plugin name
//! 3. Implement the StorageBackend trait methods
//! 4. Add your specific storage logic
//! 5. Register the plugin in your module

use agentic_sdk::plugins::context::{StorageBackend, StorageError};
use agentic_sdk::message_types::ChatMessage;
use agentic_sdk::plugin::{Plugin, PluginError};
use async_trait::async_trait;
use serde_json::Value;

/// Your custom storage backend implementation
pub struct MyStorage {
    // Add your storage-specific fields here
    // Example:
    // connection_string: String,
    // pool: Option<ConnectionPool>,
}

impl MyStorage {
    /// Create a new instance of your storage backend
    pub fn new(/* Add your constructor parameters */) -> Self {
        Self {
            // Initialize your fields
        }
    }
}

#[async_trait]
impl Plugin for MyStorage {
    fn plugin_id(&self) -> &'static str {
        "storage-my-custom" // Replace with your plugin ID
    }

    fn version(&self) -> &'static str {
        "1.0.0" // Update with your version
    }

    fn description(&self) -> &'static str {
        "My custom storage backend for Wireframe-AI" // Update with your description
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), PluginError> {
        // Parse configuration and initialize your storage
        // Example:
        // let connection_string = config
        //     .get("connection_string")
        //     .and_then(|v| v.as_str())
        //     .ok_or_else(|| PluginError::ConfigurationError("Missing connection_string".to_string()))?;
        //
        // self.connection_string = connection_string.to_string();
        // self.pool = Some(connect_to_database(&self.connection_string).await?);

        println!("MyStorage initialized with config: {}", config);
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        // Check if your storage is healthy
        // Example:
        // if let Some(pool) = &self.pool {
        //     return pool.ping().await
        //         .map(|_| true)
        //         .map_err(|e| PluginError::HealthCheckFailed(e.to_string()));
        // }

        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        // Cleanup resources
        // Example:
        // if let Some(pool) = self.pool.take() {
        //     pool.close().await;
        // }

        println!("MyStorage shutdown complete");
        Ok(())
    }
}

#[async_trait]
impl StorageBackend for MyStorage {
    async fn ensure_session(&self, session_id: &str) -> Result<(), StorageError> {
        // Ensure a session exists in your storage
        // Example:
        // self.pool
        //     .execute("INSERT OR IGNORE INTO sessions (id, created_at) VALUES (?, ?)")
        //     .bind(session_id)
        //     .bind(chrono::Utc::now().timestamp())
        //     .await
        //     .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        println!("Ensured session exists: {}", session_id);
        Ok(())
    }

    async fn store_message(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> Result<(), StorageError> {
        // Store a message in your storage
        // Example:
        // self.pool
        //     .execute("INSERT INTO messages (id, session_id, role, content, timestamp) VALUES (?, ?, ?, ?, ?)")
        //     .bind(uuid::Uuid::new_v4().to_string())
        //     .bind(session_id)
        //     .bind(role)
        //     .bind(content)
        //     .bind(chrono::Utc::now().timestamp())
        //     .await
        //     .map_err(|e| StorageError::DatabaseError(e.to_string()))?;

        println!("Stored message in session {}: role={}, content={}", session_id, role, content);
        Ok(())
    }

    async fn load_session_history(
        &self,
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<ChatMessage>, StorageError> {
        // Load session history from your storage
        // Example:
        // let rows = self.pool
        //     .query_as::<_, (String, String, i64)>(
        //         "SELECT role, content, timestamp FROM messages WHERE session_id = ? ORDER BY timestamp DESC LIMIT ?"
        //     )
        //     .bind(session_id)
        //     .bind(limit as i64)
        //     .fetch_all()
        //     .await
        //     .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        //
        // let messages = rows
        //     .into_iter()
        //     .rev()
        //     .map(|(role, content, timestamp)| ChatMessage {
        //         role,
        //         content,
        //         timestamp,
        //     })
        //     .collect();

        println!("Loaded {} messages from session {}", limit, session_id);

        // Return empty vector for template
        Ok(vec![])
    }
}

// ============================================================================
// Example Configuration
// ============================================================================

/*
Add this to your module's configuration file:

plugins:
  storage:
    plugin_id: "storage-my-custom"
    config:
      connection_string: "postgresql://user:password@localhost/mydb"
      pool_size: 10
*/

// ============================================================================
// Example Usage in Module
// ============================================================================

/*
use agentic_sdk::PluginRegistry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = PluginRegistry::new();

    // Create and register your storage plugin
    let storage = Box::new(MyStorage::new(/* params */));
    registry.register(storage).await?;

    // Retrieve and use the plugin
    let storage_plugin: Arc<MyStorage> = registry.get("storage-my-custom").await?;
    storage_plugin.ensure_session("test-session").await?;

    Ok(())
}
*/

// ============================================================================
// Testing
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_initialization() {
        let mut storage = MyStorage::new();
        let config = json!({ "connection_string": "test" });
        assert!(storage.initialize(&config).await.is_ok());
    }

    #[tokio::test]
    async fn test_ensure_session() {
        let storage = MyStorage::new();
        assert!(storage.ensure_session("test-session").await.is_ok());
    }

    #[tokio::test]
    async fn test_store_message() {
        let storage = MyStorage::new();
        assert!(storage
            .store_message("test-session", "user", "Hello")
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_load_session_history() {
        let storage = MyStorage::new();
        let history = storage.load_session_history("test-session", 10).await.unwrap();
        assert!(history.is_empty()); // Template returns empty
    }
}
