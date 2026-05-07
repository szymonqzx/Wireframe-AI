//! NATS integration for minimal TUI
//! 
//! Handles NATS connection, message publishing, and subscription

use anyhow::Result;
use async_nats::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// NATS client wrapper
#[derive(Debug)]
pub struct NatsClient {
    client: Arc<Client>,
    url: String,
}

impl NatsClient {
    /// Create new NATS client
    pub async fn connect(url: &str) -> Result<Self> {
        let client = async_nats::connect(url).await?;
        Ok(Self {
            client: Arc::new(client),
            url: url.to_string(),
        })
    }
    
    /// Get the underlying client
    pub fn client(&self) -> Arc<Client> {
        self.client.clone()
    }
    
    /// Get the connection URL
    pub fn url(&self) -> &str {
        &self.url
    }
    
    /// Publish a message
    pub async fn publish(&self, subject: &str, payload: Vec<u8>) -> Result<()> {
        self.client.publish(subject.to_string(), payload.into()).await?;
        Ok(())
    }
    
    /// Subscribe to a subject
    pub async fn subscribe(&self, subject: &str) -> Result<async_nats::Subscriber> {
        let subscriber = self.client.subscribe(subject.to_string()).await?;
        Ok(subscriber)
    }
}

/// Task submitted message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSubmitted {
    pub session_id: String,
    pub user_input: String,
    pub submitted_at: i64,
}

/// Task complete message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskComplete {
    pub session_id: String,
    pub correlation_id: String,
    pub result: String,
    pub side_effects: Vec<String>,
    pub warnings: Vec<String>,
    pub completed_at: i64,
}

/// Message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub message_id: String,
    pub session_id: String,
    pub correlation_id: String,
    pub topic: String,
    pub payload: T,
    pub schema_version: String,
    pub timestamp: i64,
}

impl<T> Envelope<T> {
    pub fn new(topic: &str, payload: T, session_id: Option<String>) -> Self {
        Self {
            message_id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            correlation_id: uuid::Uuid::new_v4().to_string(),
            topic: topic.to_string(),
            payload,
            schema_version: "v1".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

/// TUI NATS manager
#[derive(Debug)]
pub struct TuiNatsManager {
    client: Arc<RwLock<Option<Arc<NatsClient>>>>,
    pending_tasks: Arc<RwLock<Vec<String>>>,
}

impl TuiNatsManager {
    pub fn new() -> Self {
        Self {
            client: Arc::new(RwLock::new(None)),
            pending_tasks: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    /// Connect to NATS
    pub async fn connect(&self, url: &str) -> Result<()> {
        let client = NatsClient::connect(url).await?;
        *self.client.write().await = Some(Arc::new(client));
        Ok(())
    }
    
    /// Check if connected
    pub fn is_connected(&self) -> bool {
        // This is a blocking check for simplicity
        // In production, you'd want to handle this asynchronously
        let guard = self.client.try_read();
        if let Ok(client) = guard {
            client.is_some()
        } else {
            false
        }
    }
    
    /// Submit a task
    pub async fn submit_task(&self, user_input: &str) -> Result<String> {
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref().ok_or_else(|| anyhow::anyhow!("Not connected to NATS"))?;
        
        let session_id = uuid::Uuid::new_v4().to_string();
        let task = TaskSubmitted {
            session_id: session_id.clone(),
            user_input: user_input.to_string(),
            submitted_at: chrono::Utc::now().timestamp(),
        };
        
        let envelope = Envelope::new("task.submitted", task, Some(session_id.clone()));
        let payload = serde_json::to_vec(&envelope)?;
        
        client.publish("task.submitted", payload).await?;
        
        // Track pending task
        self.pending_tasks.write().await.push(session_id.clone());
        
        Ok(session_id)
    }
    
    /// Get pending tasks
    pub async fn get_pending_tasks(&self) -> Vec<String> {
        self.pending_tasks.read().await.clone()
    }
    
    /// Remove completed task from pending
    pub async fn complete_task(&self, session_id: &str) {
        let mut pending = self.pending_tasks.write().await;
        pending.retain(|id| id != session_id);
    }
}

impl Default for TuiNatsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_envelope_creation() {
        let envelope: Envelope<String> = Envelope::new("test.topic", "test payload".to_string(), Some("session123".to_string()));
        assert_eq!(envelope.topic, "test.topic");
        assert_eq!(envelope.session_id, "session123");
    }
    
    #[test]
    fn test_manager_creation() {
        let manager = TuiNatsManager::new();
        assert!(!manager.is_connected());
    }
}
