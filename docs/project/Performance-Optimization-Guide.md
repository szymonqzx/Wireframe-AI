# Wireframe-AI Performance Optimization Guide

This guide provides comprehensive strategies for optimizing Wireframe-AI performance across all modules and plugins.

## Table of Contents

- [General Principles](#general-principles)
- [Async/Await Best Practices](#asyncawait-best-practices)
- [Plugin Performance](#plugin-performance)
- [NATS Message Optimization](#nats-message-optimization)
- [Database Optimization](#database-optimization)
- [Memory Management](#memory-management)
- [Caching Strategies](#caching-strategies)
- [Connection Pooling](#connection-pooling)
- [Profiling and Benchmarking](#profiling-and-benchmarking)
- [Monitoring and Metrics](#monitoring-and-metrics)

## General Principles

### 1. Measure Before Optimizing

Always profile before optimizing. Use the built-in benchmarks:

```bash
cargo bench --bench plugin_benchmarks
```

### 2. Focus on Hot Paths

Identify and optimize the most frequently executed code paths:
- NATS message handling
- Plugin execution
- Database queries
- Serialization/deserialization

### 3. Avoid Premature Optimization

Optimize only when you have measurable performance issues. Clear code is often fast enough.

### 4. Consider Trade-offs

Optimization often involves trade-offs:
- Memory vs. CPU
- Latency vs. throughput
- Complexity vs. performance

## Async/Await Best Practices

### 1. Use Async Runtime Efficiently

```rust
// GOOD: Use tokio::spawn for concurrent operations
let handles: Vec<_> = tasks
    .into_iter()
    .map(|task| tokio::spawn(async move { process(task).await }))
    .collect();

for handle in handles {
    handle.await?;
}

// BAD: Sequential execution in async context
for task in tasks {
    process(task).await?;
}
```

### 2. Avoid Blocking in Async Code

```rust
// GOOD: Use async equivalents
let content = tokio::fs::read_to_string(path).await?;

// BAD: Blocking I/O in async context
let content = std::fs::read_to_string(path)?;
```

### 3. Use `join_all` for Independent Operations

```rust
use futures::future::join_all;

let results = join_all(
    tasks.iter().map(|task| process(task))
).await;
```

### 4. Timeout Long-Running Operations

```rust
use tokio::time::{timeout, Duration};

let result = timeout(Duration::from_secs(30), async_operation()).await?;
```

## Plugin Performance

### 1. Minimize Plugin Overhead

Keep plugin initialization and health checks lightweight:

```rust
// GOOD: Fast initialization
async fn initialize(&mut self, _config: &Value) -> Result<(), PluginError> {
    // Validate config only
    Ok(())
}

// BAD: Expensive initialization
async fn initialize(&mut self, config: &Value) -> Result<(), PluginError> {
    // Don't connect to database here
    self.db.connect(config["db_url"].as_str()).await?;
    Ok(())
}
```

### 2. Use Connection Pooling

```rust
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .max_connections(10)
    .connect(&db_url)
    .await?;
```

### 3. Cache Frequently Used Data

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

struct CachedPlugin {
    cache: Arc<RwLock<HashMap<String, Value>>>,
}

impl CachedPlugin {
    async fn get_cached(&self, key: &str) -> Option<Value> {
        let cache = self.cache.read().await;
        cache.get(key).cloned()
    }

    async fn set_cached(&self, key: String, value: Value) {
        let mut cache = self.cache.write().await;
        cache.insert(key, value);
    }
}
```

### 4. Batch Operations

```rust
// GOOD: Batch database writes
async fn store_messages_batch(&self, messages: Vec<Message>) -> Result<()> {
    let mut tx = self.pool.begin().await?;
    for msg in messages {
        sqlx::query("INSERT INTO messages ...")
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

// BAD: Individual writes
async fn store_messages(&self, messages: Vec<Message>) -> Result<()> {
    for msg in messages {
        sqlx::query("INSERT INTO messages ...")
            .execute(&self.pool)
            .await?;
    }
    Ok(())
}
```

## NATS Message Optimization

### 1. Use Efficient Serialization

```rust
// GOOD: Use bincode for performance
use bincode;

let serialized = bincode::serialize(&data)?;

// AVOID: JSON for hot paths
let serialized = serde_json::to_vec(&data)?;
```

### 2. Compress Large Payloads

```rust
use flate2::write::GzEncoder;
use flate2::Compression;

let compressed = if payload.len() > 1024 {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&payload)?;
    encoder.finish()?
} else {
    payload
};
```

### 3. Use Queue Groups Wisely

```rust
// GOOD: Appropriate queue group size
let queue_group = "workers"; // Let NATS load balance

// AVOID: Too many queue groups
let queue_group = format!("worker-{}", worker_id); // Fragmentation
```

### 4. Implement Backpressure

```rust
use tokio::sync::Semaphore;

let semaphore = Arc::new(Semaphore::new(100)); // Max 100 concurrent messages

let permit = semaphore.acquire().await?;
process_message(message).await?;
drop(permit);
```

## Database Optimization

### 1. Use Prepared Statements

```rust
// GOOD: Prepared statement
let stmt = sqlx::query_as::<_, (String, String)>(
    "SELECT role, content FROM messages WHERE session_id = $1 ORDER BY timestamp DESC LIMIT $2"
);

stmt.bind(session_id).bind(limit).fetch_all(&pool).await?;

// AVOID: Query string concatenation
let query = format!(
    "SELECT * FROM messages WHERE session_id = '{}' LIMIT {}",
    session_id, limit
);
sqlx::query(&query).fetch_all(&pool).await?;
```

### 2. Add Indexes Strategically

```sql
-- Index for session lookups
CREATE INDEX idx_messages_session_id ON messages(session_id);

-- Index for timestamp ordering
CREATE INDEX idx_messages_timestamp ON messages(timestamp DESC);

-- Composite index for common queries
CREATE INDEX idx_messages_session_timestamp ON messages(session_id, timestamp DESC);
```

### 3. Use Connection Pooling

```rust
let pool = PgPoolOptions::new()
    .max_connections(20) // Adjust based on workload
    .min_connections(5)  // Keep some connections warm
    .connect(&db_url)
    .await?;
```

### 4. Implement Read Replicas

```rust
// Use read replicas for read-heavy workloads
let read_pool = PgPoolOptions::new()
    .max_connections(30)
    .connect(&read_replica_url)
    .await?;

let write_pool = PgPoolOptions::new()
    .max_connections(10)
    .connect(&primary_url)
    .await?;
```

## Memory Management

### 1. Avoid Cloning Large Data

```rust
// GOOD: Use references
async fn process_message(&self, message: &Message) -> Result<()> {
    // Process without cloning
}

// AVOID: Cloning large data
async fn process_message(&self, message: Message) -> Result<()> {
    // Unnecessary clone
}
```

### 2. Use `Arc` for Shared Data

```rust
use std::sync::Arc;

struct Plugin {
    config: Arc<Config>,
}

impl Plugin {
    async fn process(&self) -> Result<()> {
        // Share config without cloning
        let config = Arc::clone(&self.config);
        // Use config
    }
}
```

### 3. Release Memory Promptly

```rust
// GOOD: Drop large data when done
{
    let large_data = load_large_data().await?;
    process(&large_data).await?;
} // large_data dropped here

// Continue with other work
```

### 4. Use Streaming for Large Data

```rust
// GOOD: Stream processing
async fn process_large_file(&self, path: &Path) -> Result<()> {
    let file = tokio::fs::File::open(path).await?;
    let reader = BufReader::new(file);
    
    for line in reader.lines() {
        let line = line?;
        process_line(&line).await?;
    }
    Ok(())
}
```

## Caching Strategies

### 1. In-Memory Caching

```rust
use std::collections::LRU;
use std::sync::Arc;
use tokio::sync::RwLock;

struct Cache {
    inner: Arc<RwLock<lru::LruCache<String, Value>>>,
}

impl Cache {
    fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(lru::LruCache::new(capacity))),
        }
    }

    async fn get(&self, key: &str) -> Option<Value> {
        let cache = self.inner.read().await;
        cache.get(key).cloned()
    }

    async fn put(&self, key: String, value: Value) {
        let mut cache = self.inner.write().await;
        cache.put(key, value);
    }
}
```

### 2. TTL-Based Caching

```rust
use std::time::{Duration, Instant};

struct CacheEntry {
    value: Value,
    expires_at: Instant,
}

struct TtlCache {
    inner: Arc<RwLock<HashMap<String, CacheEntry>>>,
    ttl: Duration,
}

impl TtlCache {
    async fn get(&self, key: &str) -> Option<Value> {
        let mut cache = self.inner.write().await;
        
        if let Some(entry) = cache.get(key) {
            if entry.expires_at > Instant::now() {
                return Some(entry.value.clone());
            } else {
                cache.remove(key);
            }
        }
        None
    }

    async fn put(&self, key: String, value: Value) {
        let mut cache = self.inner.write().await;
        cache.insert(key, CacheEntry {
            value,
            expires_at: Instant::now() + self.ttl,
        });
    }
}
```

### 3. Cache Invalidation

```rust
impl Cache {
    async fn invalidate(&self, pattern: &str) {
        let mut cache = self.inner.write().await;
        cache.retain(|key, _| !key.contains(pattern));
    }

    async fn invalidate_session(&self, session_id: &str) {
        let pattern = format!("session:{}", session_id);
        self.invalidate(&pattern).await;
    }
}
```

## Connection Pooling

### 1. Database Connection Pool

```rust
let pool = PgPoolOptions::new()
    .max_connections(20)
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(30))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(3600))
    .connect(&db_url)
    .await?;
```

### 2. HTTP Client Pool

```rust
use reqwest::Client;

let client = Client::builder()
    .pool_max_idle_per_host(10)
    .pool_idle_timeout(Duration::from_secs(90))
    .timeout(Duration::from_secs(30))
    .build()?;
```

### 3. NATS Connection Pool

```rust
use async_nats::ConnectOptions;

let nc = ConnectOptions::new()
    .max_reconnects(Some(10))
    .reconnect_delay(Duration::from_secs(5))
    .connect("nats://localhost:4222")
    .await?;
```

## Profiling and Benchmarking

### 1. Use Criterion for Benchmarks

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_operation(c: &mut Criterion) {
    c.bench_function("operation", |b| {
        b.iter(|| {
            // Your operation
        });
    });
}

criterion_group!(benches, bench_operation);
criterion_main!(benches);
```

### 2. Use Flamegraphs

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bench plugin_benchmarks
```

### 3. Use perf for CPU Profiling

```bash
# Record CPU usage
perf record -g cargo test

# Analyze
perf report
```

### 4. Use heaptrack for Memory Profiling

```bash
# Track memory allocations
heaptrack cargo test

# Analyze
heaptrack cargo test
```

## Monitoring and Metrics

### 1. Monitor Key Metrics

Use the monitoring module to track:
- NATS message rate and latency
- Plugin execution time
- Task completion rate
- Resource usage (CPU, memory)

### 2. Set Up Alerts

Configure alerts for:
- High error rates
- Increased latency
- Resource exhaustion
- Queue buildup

### 3. Use Distributed Tracing

Enable tracing to debug performance issues:
```rust
use monitoring::tracing::{init_tracing, TracingConfig};

let config = TracingConfig {
    enabled: true,
    sampling_rate: 0.1,
    exporter_type: ExporterType::Jaeger,
};
init_tracing(&config)?;
```

### 4. Profile in Production

Use continuous profiling to identify issues:
- Sample-based profiling
- Performance regression detection
- Capacity planning

## Common Performance Issues

### 1. N+1 Query Problem

```rust
// BAD: N+1 queries
for session_id in session_ids {
    let messages = load_messages(session_id).await?;
}

// GOOD: Batch query
let messages = load_messages_batch(&session_ids).await?;
```

### 2. Excessive Locking

```rust
// BAD: Holding lock too long
let mut cache = self.cache.write().await;
for item in items {
    process(item).await; // Don't do this while holding lock
    cache.insert(item.key, item.value);
}

// GOOD: Minimize lock duration
let mut cache = self.cache.write().await;
for item in items {
    cache.insert(item.key, item.value);
}
drop(cache);

for item in items {
    process(item).await;
}
```

### 3. Memory Leaks

```rust
// GOOD: Use weak references
use std::sync::{Arc, Weak};

struct Plugin {
    cache: Arc<RwLock<HashMap<String, Weak<Value>>>>,
}
```

### 4. CPU Spinning

```rust
// GOOD: Use async等待
tokio::time::sleep(Duration::from_millis(100)).await;

// AVOID: Busy waiting
std::thread::sleep(Duration::from_millis(100));
```

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| NATS message latency | < 10ms (p95) | `nats_message_processing_duration_seconds` |
| Plugin execution | < 100ms (p95) | `plugin_execution_duration_seconds` |
| Task completion | < 60s (p95) | `task_duration_seconds` |
| Database query | < 50ms (p95) | Custom metric |
| Memory usage | < 2GB | `memory_usage_bytes` |
| CPU usage | < 80% | `cpu_usage_percent` |

## Next Steps

- See [Monitoring README](../monitoring/README.md) for monitoring setup
- See [Plugin Benchmarks](../benchmarks/README.md) for benchmarking
- See [Production Deployment Guide](./Production-Deployment-Guide.md) for deployment strategies
