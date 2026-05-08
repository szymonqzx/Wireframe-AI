//! Plugin traits for the Context module.

use crate::message_types::{ChatMessage, ContextPackage, MemoryChunk, TaskComplete, TaskSubmitted};
use crate::plugin::Plugin;
use async_trait::async_trait;
use thiserror::Error;

/// Storage backend for sessions and messages.
///
/// Implementations handle persistence of chat sessions and messages,
/// supporting different databases (SQLite, PostgreSQL, etc.).
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Ensure a session exists in storage.
    async fn ensure_session<'a>(&'a self, session_id: &'a str) -> Result<(), StorageError>;

    /// Store a message in a session.
    async fn store_message<'a>(
        &'a self,
        session_id: &'a str,
        role: &'a str,
        content: &'a str,
    ) -> Result<(), StorageError>;

    /// Load session history with optional limit.
    async fn load_session_history<'a>(
        &'a self,
        session_id: &'a str,
        limit: usize,
    ) -> Result<Vec<ChatMessage>, StorageError>;
}

/// Memory retrieval backend.
///
/// Implementations handle memory search and persistence, supporting
/// different strategies (FTS5, RAG, graph-based, etc.).
#[async_trait]
pub trait MemoryBackend: Send + Sync {
    /// Search memory for relevant chunks.
    async fn search<'a>(
        &'a self,
        query: &'a str,
        session_id: &'a str,
        limit: usize,
    ) -> Result<Vec<MemoryChunk>, MemoryError>;

    /// Persist a memory chunk.
    async fn persist_chunk<'a>(
        &'a self,
        session_id: &'a str,
        content: &'a str,
        source: &'a str,
    ) -> Result<(), MemoryError>;

    /// Load memory chunks for a session.
    async fn load_chunks<'a>(
        &'a self,
        session_id: &'a str,
        limit: usize,
    ) -> Result<Vec<MemoryChunk>, MemoryError>;
}

/// Context enrichment strategy.
///
/// Implementations add context to tasks (memory retrieval, file context,
/// environment variables, etc.). Multiple strategies can be chained in a pipeline.
#[async_trait]
pub trait EnrichmentStrategy: Plugin + Send + Sync {
    /// Enrich a task with additional context.
    async fn enrich<'a>(
        &'a self,
        task: &'a TaskSubmitted,
        base_context: &'a ContextPackage,
    ) -> Result<ContextPackage, EnrichmentError>;

    /// Called when a task completes, for post-processing.
    async fn on_complete<'a>(
        &'a self,
        session_id: &'a str,
        result: &'a TaskComplete,
    ) -> Result<(), EnrichmentError>;
}

// ============================================================================
// Error Types
// ============================================================================

/// Storage backend error.
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Memory backend error.
#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Search failed: {0}")]
    SearchFailed(String),

    #[error("Persistence failed: {0}")]
    PersistenceFailed(String),

    #[error("Embedding generation failed: {0}")]
    EmbeddingFailed(String),

    #[error("Vector database error: {0}")]
    VectorDbError(String),
}

/// Enrichment strategy error.
#[derive(Error, Debug)]
pub enum EnrichmentError {
    #[error("Memory retrieval failed: {0}")]
    MemoryRetrievalFailed(String),

    #[error("File context failed: {0}")]
    FileContextFailed(String),

    #[error("Environment context failed: {0}")]
    EnvironmentContextFailed(String),
}
