use agentic_sdk::envelope::Envelope;
use agentic_sdk::message_types::{
    ChatMessage, ContextPackage, MemoryChunk, TaskComplete, TaskEnriched, TaskSubmitted,
};
use agentic_sdk::plugins::context::{
    EnrichmentStrategy, MemoryBackend, MemoryError, StorageBackend, StorageError,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::{error, info};
use uuid::Uuid;

/// Simple LRU cache entry
#[derive(Clone)]
struct CacheEntry<T> {
    value: T,
    accessed_at: Instant,
    version: u64, // For cache invalidation
}

impl<T> CacheEntry<T> {
    #[inline]
    fn new(value: T) -> Self {
        Self {
            value,
            accessed_at: Instant::now(),
            version: 0,
        }
    }

    #[inline]
    fn touch(&mut self) {
        self.accessed_at = Instant::now();
    }

    #[inline]
    fn is_expired(&self, ttl: Duration) -> bool {
        self.accessed_at.elapsed() > ttl
    }

    #[inline]
    fn invalidate(&mut self) -> bool {
        self.version += 1;
        true
    }

    #[inline]
    fn version(&self) -> u64 {
        self.version
    }
}

/// Cache invalidation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Default)]
pub enum InvalidationStrategy {
    /// Time-based invalidation only
    TimeOnly,
    /// Version-based invalidation
    VersionBased,
    /// Both time and version based
    #[default]
    Combined,
}


/// Cache type for selective invalidation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheType {
    /// Session history cache
    Session,
    /// Enrichment cache
    Enrichment,
}

/// Simple LRU cache with TTL support and invalidation
struct LruCache<T> {
    entries: HashMap<String, CacheEntry<T>>,
    max_size: usize,
    ttl: Duration,
    global_version: Arc<std::sync::atomic::AtomicU64>, // Global version for invalidation
    invalidation_strategy: InvalidationStrategy,
}

/// Simple string interner for reducing allocations
struct StringInterner {
    strings: HashMap<String, Arc<str>>,
    max_size: usize,
}

impl StringInterner {
    #[inline]
    fn new(max_size: usize) -> Self {
        Self {
            strings: HashMap::with_capacity(max_size),
            max_size,
        }
    }

    #[inline]
    fn intern(&mut self, s: &str) -> Arc<str> {
        if let Some(interned) = self.strings.get(s) {
            interned.clone()
        } else {
            let arc: Arc<str> = s.into();
            // Evict if at capacity
            if self.strings.len() >= self.max_size {
                if let Some(key) = self.strings.keys().next().cloned() {
                    self.strings.remove(&key);
                }
            }
            self.strings.insert(s.to_string(), arc.clone());
            arc
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.strings.clear();
    }

    #[inline]
    fn len(&self) -> usize {
        self.strings.len()
    }
}

impl<T: Clone> LruCache<T> {
    #[inline]
    fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            entries: HashMap::with_capacity(max_size),
            max_size,
            ttl,
            global_version: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            invalidation_strategy: InvalidationStrategy::default(),
        }
    }

    #[inline]
    fn new_with_invalidation(
        max_size: usize,
        ttl: Duration,
        strategy: InvalidationStrategy,
    ) -> Self {
        Self {
            entries: HashMap::with_capacity(max_size),
            max_size,
            ttl,
            global_version: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            invalidation_strategy: strategy,
        }
    }

    #[inline]
    fn get(&mut self, key: &str) -> Option<T> {
        if let Some(entry) = self.entries.get_mut(key) {
            // Check version-based invalidation
            if self.invalidation_strategy == InvalidationStrategy::VersionBased
                || self.invalidation_strategy == InvalidationStrategy::Combined
            {
                let global_ver = self
                    .global_version
                    .load(std::sync::atomic::Ordering::SeqCst);
                if entry.version() < global_ver {
                    self.entries.remove(key);
                    return None;
                }
            }

            // Check time-based invalidation
            if entry.is_expired(self.ttl) {
                self.entries.remove(key);
                return None;
            }

            entry.touch();
            Some(entry.value.clone())
        } else {
            None
        }
    }

    #[inline]
    fn put(&mut self, key: String, value: T) {
        // Evict if at capacity
        if self.entries.len() >= self.max_size {
            if let Some(oldest_key) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.accessed_at)
                .map(|(k, _)| k.clone())
            {
                self.entries.remove(&oldest_key);
            }
        }

        let current_version = self
            .global_version
            .load(std::sync::atomic::Ordering::SeqCst);
        let mut entry = CacheEntry::new(value);
        entry.version = current_version;
        self.entries.insert(key, entry);
    }

    #[inline]
    fn clear(&mut self) {
        self.entries.clear();
    }

    #[inline]
    fn len(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    fn invalidate_all(&mut self) {
        self.global_version
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    #[inline]
    fn invalidate_key(&mut self, key: &str) {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.invalidate();
        }
    }

    #[inline]
    fn global_version(&self) -> u64 {
        self.global_version
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    #[inline]
    fn invalidation_strategy(&self) -> InvalidationStrategy {
        self.invalidation_strategy
    }
}

/// Context core orchestration layer with efficient caching
pub struct ContextCore {
    storage: Arc<RwLock<Option<Arc<dyn StorageBackend>>>>,
    memory: Arc<RwLock<Option<Arc<dyn MemoryBackend>>>>,
    enrichment_pipeline: Arc<RwLock<Vec<Arc<dyn EnrichmentStrategy>>>>,
    max_session_history: usize,
    max_memory_chunks: usize,
    max_context_tokens: usize,
    cached_working_dir: PathBuf,
    // Caching layers
    session_cache: Arc<RwLock<LruCache<Vec<ChatMessage>>>>,
    enrichment_cache: Arc<RwLock<LruCache<ContextPackage>>>,
    // String interning for reducing allocations
    string_interner: Arc<RwLock<StringInterner>>,
    // Connection pooling for concurrent operations
    storage_semaphore: Arc<Semaphore>,
    memory_semaphore: Arc<Semaphore>,
}

impl ContextCore {
    pub fn new(
        max_session_history: usize,
        max_memory_chunks: usize,
        max_context_tokens: usize,
    ) -> Self {
        let working_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        // Initialize caches with reasonable defaults
        let cache_ttl = Duration::from_secs(300); // 5 minutes TTL
        let cache_size = 100; // Max 100 entries per cache
        let interner_size = 1000; // Max 1000 interned strings

        Self {
            storage: Arc::new(RwLock::new(None)),
            memory: Arc::new(RwLock::new(None)),
            enrichment_pipeline: Arc::new(RwLock::new(Vec::new())),
            max_session_history,
            max_memory_chunks,
            max_context_tokens,
            cached_working_dir: working_dir,
            session_cache: Arc::new(RwLock::new(LruCache::new(cache_size, cache_ttl))),
            enrichment_cache: Arc::new(RwLock::new(LruCache::new(cache_size, cache_ttl))),
            string_interner: Arc::new(RwLock::new(StringInterner::new(interner_size))),
            storage_semaphore: Arc::new(Semaphore::new(10)), // Max 10 concurrent storage ops
            memory_semaphore: Arc::new(Semaphore::new(10)),  // Max 10 concurrent memory ops
        }
    }

    /// Clear all caches
    #[inline]
    pub async fn clear_caches(&self) {
        self.session_cache.write().await.clear();
        self.enrichment_cache.write().await.clear();
        self.string_interner.write().await.clear();
    }

    /// Get cache statistics
    #[inline]
    pub async fn cache_stats(&self) -> (usize, usize, usize) {
        let session_len = self.session_cache.read().await.len();
        let enrichment_len = self.enrichment_cache.read().await.len();
        let interner_len = self.string_interner.read().await.len();
        (session_len, enrichment_len, interner_len)
    }

    /// Invalidate all caches (global version increment).
    #[inline]
    pub async fn invalidate_all_caches(&self) {
        self.session_cache.write().await.invalidate_all();
        self.enrichment_cache.write().await.invalidate_all();
    }

    /// Invalidate a specific cache entry by key.
    #[inline]
    pub async fn invalidate_cache_key(&self, cache_type: CacheType, key: &str) {
        match cache_type {
            CacheType::Session => self.session_cache.write().await.invalidate_key(key),
            CacheType::Enrichment => self.enrichment_cache.write().await.invalidate_key(key),
        }
    }

    /// Get cache version information.
    #[inline]
    pub async fn cache_versions(&self) -> (u64, u64) {
        let session_ver = self.session_cache.read().await.global_version();
        let enrichment_ver = self.enrichment_cache.read().await.global_version();
        (session_ver, enrichment_ver)
    }

    /// Set the storage plugin directly
    pub async fn set_storage(&self, storage: Arc<dyn StorageBackend>) {
        *self.storage.write().await = Some(storage);
    }

    /// Set the memory plugin directly
    pub async fn set_memory(&self, memory: Arc<dyn MemoryBackend>) {
        *self.memory.write().await = Some(memory);
    }

    /// Add an enrichment strategy to the pipeline
    pub async fn add_enrichment(&self, enrichment: Arc<dyn EnrichmentStrategy>) {
        self.enrichment_pipeline.write().await.push(enrichment);
    }

    /// Process a task.submitted message with caching
    pub async fn handle_task(
        &self,
        task: TaskSubmitted,
        envelope: Envelope<TaskSubmitted>,
    ) -> Result<TaskEnriched, Box<dyn std::error::Error>> {
        info!(session = %task.session_id, "processing task.submitted");

        // Optimize: Get all references in one read to reduce lock contention
        let (storage_opt, memory_opt, pipeline) = {
            let storage_guard = self.storage.read().await;
            let memory_guard = self.memory.read().await;
            let pipeline_guard = self.enrichment_pipeline.read().await;
            (
                storage_guard.clone(),
                memory_guard.clone(),
                pipeline_guard.clone(),
            )
        };

        // 1. Store user message via storage plugin with connection pooling
        if let Some(storage) = storage_opt.as_ref() {
            let _permit = self.storage_semaphore.acquire().await.unwrap();
            if let Err(e) = storage.ensure_session(&task.session_id).await {
                error!(error = ?e, "failed to ensure session");
            }
            if let Err(e) = storage
                .store_message(&task.session_id, "user", &task.user_input)
                .await
            {
                error!(error = ?e, "failed to store user message");
            }
        }

        // 2. Load session history with caching
        let cache_key = format!("session:{}", task.session_id);
        let session_history = {
            let mut cache = self.session_cache.write().await;
            if let Some(cached) = cache.get(&cache_key) {
                cached
            } else {
                let history = if let Some(storage) = storage_opt.as_ref() {
                    let _permit = self.storage_semaphore.acquire().await.unwrap();
                    storage
                        .load_session_history(&task.session_id, self.max_session_history)
                        .await
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };
                cache.put(cache_key, history.clone());
                history
            }
        };

        // 3. Search memory with connection pooling
        let mut memory_chunks = if let Some(memory) = memory_opt.as_ref() {
            let _permit = self.memory_semaphore.acquire().await.unwrap();
            memory
                .search(&task.user_input, &task.session_id, self.max_memory_chunks)
                .await
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        // 4. Load persistent chunks via memory plugin with connection pooling
        if let Some(memory) = memory_opt.as_ref() {
            let _permit = self.memory_semaphore.acquire().await.unwrap();
            if let Ok(persistent) = memory
                .load_chunks(&task.session_id, self.max_memory_chunks / 2)
                .await
            {
                memory_chunks.extend(persistent);
            }
        }

        // 5. Run enrichment pipeline with caching
        let enrichment_key = format!(
            "enrich:{}:{}",
            task.session_id,
            task.user_input.chars().take(50).collect::<String>()
        );
        let context = {
            let mut cache = self.enrichment_cache.write().await;
            if let Some(cached) = cache.get(&enrichment_key) {
                cached
            } else {
                let mut ctx = ContextPackage {
                    memory_chunks,
                    session_history,
                    readonly_files: vec![],
                    safe_env: std::collections::HashMap::new(),
                    working_dir: self.cached_working_dir.clone(),
                    max_context_tokens: self.max_context_tokens,
                };

                // Record baseline sizes so that when each plugin returns a full
                // ContextPackage (clone of input + its own additions), we only
                // merge the *delta* and don't duplicate the original data once
                // per plugin.
                let base_memory_len = ctx.memory_chunks.len();
                let base_session_len = ctx.session_history.len();
                let base_readonly_len = ctx.readonly_files.len();
                let base_env_keys: std::collections::HashSet<String> =
                    ctx.safe_env.keys().cloned().collect();

                // Use cloned pipeline to avoid holding read lock during enrichment
                let mut futures = Vec::new();
                for plugin in pipeline.iter() {
                    let task_ref = &task;
                    let ctx_ref = &ctx;
                    futures.push(async move { plugin.enrich(task_ref, ctx_ref).await });
                }

                let results = futures::future::join_all(futures).await;

                for mut enriched_ctx in results.into_iter().flatten() {
                    if enriched_ctx.memory_chunks.len() > base_memory_len {
                        ctx.memory_chunks
                            .extend(enriched_ctx.memory_chunks.drain(base_memory_len..));
                    }
                    if enriched_ctx.session_history.len() > base_session_len {
                        ctx.session_history
                            .extend(enriched_ctx.session_history.drain(base_session_len..));
                    }
                    if enriched_ctx.readonly_files.len() > base_readonly_len {
                        ctx.readonly_files
                            .extend(enriched_ctx.readonly_files.drain(base_readonly_len..));
                    }
                    for (k, v) in enriched_ctx.safe_env {
                        if !base_env_keys.contains(&k) {
                            ctx.safe_env.insert(k, v);
                        }
                    }
                }

                cache.put(enrichment_key, ctx.clone());
                ctx
            }
        };

        // Optimize string allocations for common fields
        let session_id = task.session_id.clone();
        let user_input = task.user_input.clone();

        let enriched = TaskEnriched {
            session_id,
            correlation_id: envelope.correlation_id,
            user_input,
            context,
            inferred_constraints: vec![],
            enriched_at: chrono::Utc::now().timestamp(),
        };

        Ok(enriched)
    }

    /// Handle task.complete to persist assistant response
    pub async fn handle_complete(
        &self,
        complete: TaskComplete,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!(session = %complete.session_id, "persisting task.complete as memory");

        let memory_opt = self.memory.read().await.clone();
        if let Some(memory) = memory_opt.as_ref() {
            let _permit = self.memory_semaphore.acquire().await.unwrap();
            if let Err(e) = memory
                .persist_chunk(&complete.session_id, &complete.result, "assistant_response")
                .await
            {
                error!(error = ?e, "failed to persist memory chunk");
            }
        }

        Ok(())
    }
}

/// Built-in in-memory storage backend for basic functionality.
pub struct InMemoryStorage {
    sessions: Arc<RwLock<HashMap<String, Vec<ChatMessage>>>>,
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl StorageBackend for InMemoryStorage {
    async fn ensure_session<'a>(&'a self, session_id: &'a str) -> Result<(), StorageError> {
        let mut sessions = self.sessions.write().await;
        if !sessions.contains_key(session_id) {
            sessions.insert(session_id.to_string(), Vec::new());
        }
        Ok(())
    }

    async fn store_message<'a>(
        &'a self,
        session_id: &'a str,
        role: &'a str,
        content: &'a str,
    ) -> Result<(), StorageError> {
        let mut sessions = self.sessions.write().await;
        let messages = sessions
            .entry(session_id.to_string())
            .or_insert_with(Vec::new);
        messages.push(ChatMessage {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
        });
        Ok(())
    }

    async fn load_session_history<'a>(
        &'a self,
        session_id: &'a str,
        limit: usize,
    ) -> Result<Vec<ChatMessage>, StorageError> {
        let sessions = self.sessions.read().await;
        Ok(sessions
            .get(session_id)
            .map(|msgs| {
                let len = msgs.len();
                if len > limit {
                    msgs[len - limit..].to_vec()
                } else {
                    msgs.clone()
                }
            })
            .unwrap_or_default())
    }
}

/// Built-in in-memory memory backend for testing.
pub struct InMemoryBackend {
    chunks: Arc<RwLock<HashMap<String, Vec<MemoryChunk>>>>,
}

impl Default for InMemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryBackend {
    pub fn new() -> Self {
        Self {
            chunks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl MemoryBackend for InMemoryBackend {
    async fn search<'a>(
        &'a self,
        query: &'a str,
        session_id: &'a str,
        limit: usize,
    ) -> Result<Vec<MemoryChunk>, MemoryError> {
        let chunks = self.chunks.read().await;
        let all_chunks = chunks.get(session_id).cloned().unwrap_or_default();

        // Simple substring matching for basic functionality
        let query_lower = query.to_lowercase();
        let matched: Vec<MemoryChunk> = all_chunks
            .into_iter()
            .filter(|chunk| chunk.content.to_lowercase().contains(&query_lower))
            .take(limit)
            .collect();

        Ok(matched)
    }

    async fn persist_chunk<'a>(
        &'a self,
        session_id: &'a str,
        content: &'a str,
        source: &'a str,
    ) -> Result<(), MemoryError> {
        let mut chunks = self.chunks.write().await;
        let session_chunks = chunks
            .entry(session_id.to_string())
            .or_insert_with(Vec::new);
        session_chunks.push(MemoryChunk {
            id: Uuid::new_v4().to_string(),
            content: content.to_string(),
            source: source.to_string(),
            relevance_score: 0.0,
        });
        Ok(())
    }

    async fn load_chunks<'a>(
        &'a self,
        session_id: &'a str,
        limit: usize,
    ) -> Result<Vec<MemoryChunk>, MemoryError> {
        let chunks = self.chunks.read().await;
        Ok(chunks
            .get(session_id)
            .map(|session_chunks| {
                let len = session_chunks.len();
                if len > limit {
                    session_chunks[len - limit..].to_vec()
                } else {
                    session_chunks.clone()
                }
            })
            .unwrap_or_default())
    }
}
