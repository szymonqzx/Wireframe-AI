//! wireframe-ai-context-core — Context module orchestration layer
//!
//! This module handles:
//! - NATS communication (task.submitted, task.enriched, task.complete)
//! - Plugin lifecycle management (storage, memory, enrichment)
//! - Task enrichment orchestration
//!
//! Domain logic is delegated to plugins implementing:
//! - StorageBackend (session and message persistence)
//! - MemoryBackend (memory search and retrieval)
//! - EnrichmentStrategy (context enrichment)

pub mod context_core;

pub use context_core::{ContextCore, InMemoryStorage, InMemoryBackend};
