# Rust Advanced Patterns

Advanced Rust patterns including traits, generics, concurrency, Wireframe-AI specific patterns, unsafe code, and module system.

## Traits and Generics

### Accept Generics, Return Concrete Types

```rust
// Good: Generic input, concrete output
fn read_all(reader: &mut impl Read) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(buf)
}

// Good: Trait bounds for multiple constraints
fn process<T: Display + Send + 'static>(item: T) -> String {
    format!("processed: {item}")
}
```

### Trait Objects for Dynamic Dispatch

```rust
// Use when you need heterogeneous collections or plugin systems
trait Handler: Send + Sync {
    fn handle(&self, request: &Request) -> Response;
}

struct Router {
    handlers: Vec<Box<dyn Handler>>,
}

// Use generics when you need performance (monomorphization)
fn fast_process<H: Handler>(handler: &H, request: &Request) -> Response {
    handler.handle(request)
}
```

### Newtype Pattern for Type Safety

```rust
// Good: Distinct types prevent mixing up arguments
struct UserId(u64);
struct OrderId(u64);

fn get_order(user: UserId, order: OrderId) -> Result<Order> {
    // Can't accidentally swap user and order IDs
    todo!()
}

// Bad: Easy to swap arguments
fn get_order_bad(user_id: u64, order_id: u64) -> Result<Order> {
    todo!()
}
```

## Concurrency

### `Arc<Mutex<T>>` for Shared Mutable State

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
let handles: Vec<_> = (0..10).map(|_| {
    let counter = Arc::clone(&counter);
    std::thread::spawn(move || {
        let mut num = counter.lock().expect("mutex poisoned");
        *num += 1;
    })
}).collect();

for handle in handles {
    handle.join().expect("worker thread panicked");
}
```

### Channels for Message Passing

```rust
use std::sync::mpsc;

let (tx, rx) = mpsc::sync_channel(16); // Bounded channel with backpressure

for i in 0..5 {
    let tx = tx.clone();
    std::thread::spawn(move || {
        tx.send(format!("message {i}")).expect("receiver disconnected");
    });
}
drop(tx); // Close sender so rx iterator terminates

for msg in rx {
    println!("{msg}");
}
```

### Async with Tokio

```rust
use tokio::time::Duration;

async fn fetch_with_timeout(url: &str) -> Result<String> {
    let response = tokio::time::timeout(
        Duration::from_secs(5),
        reqwest::get(url),
    )
    .await
    .context("request timed out")?
    .context("request failed")?;

    response.text().await.context("failed to read body")
}

// Spawn concurrent tasks
async fn fetch_all(urls: Vec<String>) -> Vec<Result<String>> {
    let handles: Vec<_> = urls.into_iter()
        .map(|url| tokio::spawn(async move {
            fetch_with_timeout(&url).await
        }))
        .collect();

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        results.push(handle.await.unwrap_or_else(|e| panic!("spawned task panicked: {e}")));
    }
    results
}
```

## Wireframe-AI Specific Patterns

### NATS Message Handling

```rust
// Good: Use async/await with NATS
async fn subscribe_to_topic(client: &async_nats::Client, topic: &str) -> Result<()> {
    let mut subscriber = client.subscribe(topic.to_string()).await?;

    while let Some(message) = subscriber.next().await {
        handle_message(message).await?;
    }

    Ok(())
}

// Good: Proper error handling for NATS operations
async fn publish_message(client: &async_nats::Client, topic: &str, payload: &[u8]) -> Result<()> {
    client.publish(topic.to_string(), payload.to_vec())
        .await
        .context("failed to publish NATS message")?;
    Ok(())
}
```

### SQLite Database Operations

```rust
// Good: Use transactions for multi-step operations
async fn update_user_with_transaction(db: &Pool<Sqlite>, user_id: i64, updates: UserUpdate) -> Result<()> {
    let mut tx = db.begin().await?;

    sqlx::query("UPDATE users SET name = ? WHERE id = ?")
        .bind(&updates.name)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("INSERT INTO user_audit (user_id, action) VALUES (?, ?)")
        .bind(user_id)
        .bind("update")
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

// Good: Use prepared statements for repeated queries
async fn get_user_by_id(db: &Pool<Sqlite>, id: i64) -> Result<Option<User>> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_optional(db)
        .await
        .context("failed to fetch user")
}
```

### Provider System Integration

```rust
// Good: Use the Provider trait for LLM integration
async fn call_provider(provider: &dyn Provider, prompt: &str) -> Result<String> {
    let response = provider
        .complete(prompt)
        .await
        .context("provider call failed")?;
    Ok(response)
}

// Good: Handle provider-specific errors
async fn handle_provider_error(error: ProviderError) -> Result<()> {
    match error {
        ProviderError::RateLimit => {
            tokio::time::sleep(Duration::from_secs(5)).await;
            Ok(())
        }
        ProviderError::Authentication => {
            bail!("provider authentication failed - check API key")
        }
        ProviderError::Timeout => {
            bail!("provider request timed out")
        }
    }
}
```

## Unsafe Code

### When Unsafe Is Acceptable

```rust
// Acceptable: FFI boundary with documented invariants (Rust 2024+)
/// # Safety
/// `ptr` must be a valid, aligned pointer to an initialized `Widget`.
unsafe fn widget_from_raw<'a>(ptr: *const Widget) -> &'a Widget {
    // SAFETY: caller guarantees ptr is valid and aligned
    unsafe { &*ptr }
}

// Acceptable: Performance-critical path with proof of correctness
// SAFETY: index is always < len due to the loop bound
unsafe { slice.get_unchecked(index) }
```

### When Unsafe Is NOT Acceptable

```rust
// Bad: Using unsafe to bypass borrow checker
// Bad: Using unsafe for convenience
// Bad: Using unsafe without a Safety comment
// Bad: Transmuting between unrelated types
```

## Module System and Crate Structure

### Organize by Domain, Not by Type

```text
wireframe-ai/
├── kernel/
│   ├── interface/
│   │   └── src/
│   │       ├── main.rs
│   │       └── lib.rs
│   └── modules/
│       ├── context/
│       │   └── src/
│       │       ├── lib.rs
│       │       ├── state.rs
│       │       └── storage.rs
│       ├── orchestrator/
│       │   └── src/
│       │       ├── lib.rs
│       │       └── scheduler.rs
│       └── provider/
│           └── src/
│               ├── lib.rs
│               └── trait.rs
├── adapter/
│   └── python/
│       └── src/
│           └── lib.rs
├── schemas/
│   └── v1/
│       └── envelope.json
└── Cargo.toml
```

### Visibility — Expose Minimally

```rust
// Good: pub(crate) for internal sharing
pub(crate) fn validate_input(input: &str) -> bool {
    !input.is_empty()
}

// Good: Re-export public API from lib.rs
pub mod context;
pub use context::ContextManager;

// Bad: Making everything pub
pub fn internal_helper() {} // Should be pub(crate) or private
```

## Reference

See `SKILL.md` for core patterns including ownership, error handling, enums, and iterators.
See `rust-patterns-examples.md` for tooling, quick reference, anti-patterns, and integration.