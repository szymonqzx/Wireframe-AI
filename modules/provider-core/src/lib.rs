//! Wireframe-AI Provider Core
//!
//! Provider trait and infrastructure for LLM backends.
//! Adapted from jcode's provider pattern for Wireframe-AI.

pub mod config;
pub mod discovery;
pub mod marketplace;
pub mod router;
pub mod testing;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use std::sync::OnceLock;

/// Stream of events from a provider.
pub type EventStream = Pin<Box<dyn Stream<Item = Result<StreamEvent>> + Send>>;

/// Stream event from provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    #[serde(rename = "text_delta")]
    TextDelta { text: String },
    #[serde(rename = "tool_call")]
    ToolCall {
        id: String,
        name: String,
        arguments: String,
    },
    #[serde(rename = "done")]
    Done,
}

/// Message in conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Tool definition for LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Provider metadata from provider.describe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    pub provider_id: String,
    pub provider_label: String,
    pub provider_version: String,
    pub protocol_version: String,
    pub transport: String,
    pub capabilities: ProviderCapabilities,
}

/// Provider capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub core_methods: Vec<String>,
    pub optional_methods: Vec<String>,
    pub features: Vec<String>,
    pub custom_methods: Vec<CustomMethod>,
}

/// Custom provider-specific method.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMethod {
    pub name: String,
    pub stability: String,
    pub description: String,
}

/// Provider status from provider.status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub availability: Availability,
    pub setup_state: SetupState,
    pub requires_manual_setup: bool,
    pub diagnostics: Vec<Diagnostic>,
}

/// Availability status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Availability {
    Ready,
    Degraded,
    Unavailable,
}

/// Setup state.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SetupState {
    Complete,
    Partial,
    Required,
    Broken,
}

/// Diagnostic message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub level: String,
    pub code: String,
    pub message: String,
}

/// Cost tracking for a single request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageCost {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    /// Cost in the provider's native currency (usually USD cents)
    pub cost_cents: Option<u64>,
    /// Provider-specific cost metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Per-provider cost accumulator.
#[derive(Debug, Clone)]
pub struct ProviderCostTracker {
    pub provider_name: String,
    pub total_requests: u64,
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub total_cost_cents: u64,
    pub requests: Vec<UsageCost>,
}

impl ProviderCostTracker {
    pub fn new(provider_name: String) -> Self {
        Self {
            provider_name,
            total_requests: 0,
            total_prompt_tokens: 0,
            total_completion_tokens: 0,
            total_cost_cents: 0,
            requests: vec![],
        }
    }

    pub fn record(&mut self, cost: UsageCost) {
        self.total_requests += 1;
        self.total_prompt_tokens += cost.prompt_tokens as u64;
        self.total_completion_tokens += cost.completion_tokens as u64;
        if let Some(cents) = cost.cost_cents {
            self.total_cost_cents += cents;
        }
        self.requests.push(cost);
    }
}

/// Global cost tracking across all providers.
#[derive(Debug, Clone, Default)]
pub struct CostTracker {
    providers: HashMap<String, ProviderCostTracker>,
}

impl CostTracker {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    pub fn record(&mut self, provider_name: &str, cost: UsageCost) {
        self.providers
            .entry(provider_name.to_string())
            .or_insert_with(|| ProviderCostTracker::new(provider_name.to_string()))
            .record(cost);
    }

    pub fn get_provider_summary(&self, provider_name: &str) -> Option<&ProviderCostTracker> {
        self.providers.get(provider_name)
    }

    pub fn total_cost_cents(&self) -> u64 {
        self.providers.values().map(|p| p.total_cost_cents).sum()
    }

    pub fn all_providers(&self) -> Vec<&ProviderCostTracker> {
        self.providers.values().collect()
    }
}

/// Provider trait for LLM backends.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Send messages and get a streaming response.
    ///
    /// session_id: Optional session ID to resume a previous conversation.
    async fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        system: &str,
        session_id: Option<&str>,
    ) -> Result<EventStream>;

    /// Get provider metadata (provider.describe).
    fn describe(&self) -> ProviderMetadata;

    /// Get current status (provider.status).
    fn status(&self) -> ProviderStatus;

    /// Get the provider name.
    fn name(&self) -> &str;

    /// Get the model identifier being used.
    fn model(&self) -> String;

    /// Set the model to use (returns error if model not supported).
    fn set_model(&self, _model: &str) -> Result<()> {
        Err(anyhow::anyhow!(
            "This provider does not support model switching"
        ))
    }

    /// List available models for this provider.
    fn available_models(&self) -> Vec<String> {
        vec![]
    }

    /// Whether this provider supports streaming.
    fn supports_streaming(&self) -> bool {
        true
    }

    /// Whether this provider supports tools.
    fn supports_tools(&self) -> bool {
        true
    }

    /// Get the transport type.
    fn transport(&self) -> String {
        "http".to_string()
    }

    /// Estimated cost per 1K tokens (prompt, completion) in USD cents.
    fn cost_per_1k_tokens(&self) -> Option<(u64, u64)> {
        None
    }

    /// Create a new provider instance with independent mutable state.
    fn fork(&self) -> Arc<dyn Provider>;
}

/// Session for conversation context.
#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: String,
    pub provider_name: String,
    pub model: String,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
}

impl Session {
    pub fn new(provider_name: String, model: &str) -> Self {
        let now = Utc::now();
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            provider_name,
            model: model.to_string(),
            messages: vec![],
            created_at: now,
            updated_at: now,
            last_accessed: now,
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
        self.updated_at = Utc::now();
        self.last_accessed = Utc::now();
    }

    /// Update last accessed timestamp.
    pub fn touch(&mut self) {
        self.last_accessed = Utc::now();
    }

    /// Check if session is expired based on TTL.
    pub fn is_expired(&self, ttl: Duration) -> bool {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.last_accessed);
        elapsed.num_seconds() > ttl.as_secs() as i64
    }
}

/// Session manager for conversation lifecycle.
/// Uses dashmap for better concurrency performance.
#[derive(Debug, Clone)]
pub struct SessionManager {
    sessions: dashmap::DashMap<String, Session>,
    ttl: Duration,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: dashmap::DashMap::new(),
            ttl: Duration::from_secs(3600), // Default 1 hour TTL
        }
    }

    pub fn with_ttl(ttl: Duration) -> Self {
        Self {
            sessions: dashmap::DashMap::new(),
            ttl,
        }
    }

    /// Ensure a session exists (create or reuse).
    pub fn ensure_session(
        &self,
        session_id: Option<&str>,
        provider: &str,
        model: &str,
    ) -> String {
        if let Some(id) = session_id {
            if self.sessions.contains_key(id) {
                // Update last accessed on reuse
                if let Some(mut session) = self.sessions.get_mut(id) {
                    session.touch();
                }
                return id.to_string();
            }
        }

        let session = Session::new(provider.to_string(), model);
        let id = session.session_id.clone();
        self.sessions.insert(id.clone(), session);
        id
    }

    /// Get a session by ID.
    pub fn get_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.get(session_id).map(|s| {
            // Update last accessed on read
            if let Some(mut session) = self.sessions.get_mut(session_id) {
                session.touch();
            }
            s.clone()
        })
    }

    /// Get a mutable session by ID (for internal use).
    pub fn get_session_mut(&self, session_id: &str) -> Option<dashmap::mapref::one::RefMut<'_, String, Session>> {
        self.sessions.get_mut(session_id)
    }

    /// Close a session.
    pub fn close_session(&self, session_id: &str) -> Option<Session> {
        self.sessions.remove(session_id).map(|(_, s)| s)
    }

    /// List all active sessions.
    pub fn list_sessions(&self) -> Vec<Session> {
        self.sessions.iter().map(|s| s.clone()).collect()
    }

    /// Clean up expired sessions with optimized batch processing.
    /// Only scans sessions if the count exceeds a threshold to avoid unnecessary scans.
    pub fn cleanup_expired(&self) -> usize {
        const CLEANUP_THRESHOLD: usize = 100;
        
        // Skip cleanup if we have few sessions
        if self.sessions.len() < CLEANUP_THRESHOLD {
            return 0;
        }

        let mut expired_keys = Vec::new();
        let now = chrono::Utc::now();
        
        for entry in self.sessions.iter() {
            // Fast path: check if session is definitely expired
            let last_accessed = entry.value().last_accessed;
            let elapsed = now.signed_duration_since(last_accessed);
            let ttl_delta = chrono::Duration::from_std(self.ttl).unwrap_or(chrono::Duration::seconds(self.ttl.as_secs() as i64));
            if elapsed > ttl_delta {
                expired_keys.push(entry.key().clone());
            }
        }
        
        let count = expired_keys.len();
        for key in expired_keys {
            self.sessions.remove(&key);
        }
        count
    }

    /// Force cleanup of all expired sessions regardless of threshold.
    pub fn force_cleanup_expired(&self) -> usize {
        let mut expired_keys = Vec::new();
        for entry in self.sessions.iter() {
            if entry.value().is_expired(self.ttl) {
                expired_keys.push(entry.key().clone());
            }
        }
        let count = expired_keys.len();
        for key in expired_keys {
            self.sessions.remove(&key);
        }
        count
    }

    /// Get the number of active sessions.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Get or create a shared HTTP client for provider use.
/// This enables connection pooling and reuse across providers with optimized settings.
pub fn get_http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .pool_max_idle_per_host(25) // Increased for better concurrency
            .pool_idle_timeout(Duration::from_secs(120)) // Longer idle timeout
            .timeout(Duration::from_secs(120)) // Longer overall timeout for LLM calls
            .connect_timeout(Duration::from_secs(10))
            .tcp_keepalive(Duration::from_secs(60)) // Keep connections alive
            .http2_keep_alive_interval(Duration::from_secs(30)) // HTTP/2 keep-alive
            .http2_keep_alive_timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new("openai".to_string(), "gpt-4o");
        assert!(!session.session_id.is_empty());
        assert_eq!(session.provider_name, "openai");
        assert_eq!(session.model, "gpt-4o");
        assert!(session.messages.is_empty());
    }

    #[test]
    fn test_session_manager() {
        let manager = SessionManager::new();
        let session_id = manager.ensure_session(None, "openai", &"gpt-4o".to_string());
        assert!(!session_id.is_empty());

        let session = manager.get_session(&session_id);
        assert!(session.is_some());
        assert_eq!(session.unwrap().provider_name, "openai");
    }

    #[test]
    fn test_session_add_message() {
        let mut session = Session::new("openai".to_string(), "gpt-4o");
        let message = Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
            tool_call_id: None,
        };
        session.add_message(message);
        assert_eq!(session.messages.len(), 1);
    }
}
