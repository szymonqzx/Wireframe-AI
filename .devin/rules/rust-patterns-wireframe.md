---
paths:
  - "**/*.rs"
---
# Wireframe-AI Specific Rust Patterns

Wireframe-AI specific patterns for NATS messaging, Provider system, and Context state ownership.

## Wireframe-AI NATS Messaging Pattern

### Message Envelope

Consistent message structure with immutable root fields:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope<T> {
    pub version: String,           // Immutable - version of envelope schema
    pub timestamp: i64,            // Immutable - message creation time
    pub source: String,            // Immutable - module that sent the message
    pub correlation_id: String,    // Immutable - for request/response correlation
    pub payload: T,                // Mutable - actual message payload
}
```

### Topic Naming

Use `namespace.noun.verb` or `namespace.noun` format:

```rust
// GOOD - follows convention
const TOPIC_USER_CREATED: &str = "user.user.created";
const TOPIC_ORDER_STATUS: &str = "order.order";
const TOPIC_MODULE_ONLINE: &str = "sys.module.online";

// BAD - violates convention
const TOPIC_BAD: &str = "UserCreatedEvent";
const TOPIC_BAD2: &str = "user/created";
```

### Module Registration

Modules must publish lifecycle messages:

```rust
pub async fn register_module(
    nats: &async_nats::Client,
    module_name: &str,
) -> anyhow::Result<()> {
    let info = ModuleInfo {
        name: module_name.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };

    nats.publish("sys.module.online", serde_json::to_vec(&info)?)
        .await?;

    Ok(())
}

pub async fn unregister_module(
    nats: &async_nats::Client,
    module_name: &str,
) -> anyhow::Result<()> {
    let info = ModuleInfo {
        name: module_name.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    };

    nats.publish("sys.module.offline", serde_json::to_vec(&info)?)
        .await?;

    Ok(())
}
```

### Message Publishing

```rust
pub async fn publish_message<T>(
    nats: &async_nats::Client,
    topic: &str,
    payload: T,
    source: &str,
) -> anyhow::Result<()>
where
    T: Serialize,
{
    let envelope = MessageEnvelope {
        version: "1.0".to_string(),
        timestamp: chrono::Utc::now().timestamp(),
        source: source.to_string(),
        correlation_id: Uuid::new_v4().to_string(),
        payload,
    };

    let data = serde_json::to_vec(&envelope)?;
    nats.publish(topic, data).await?;

    Ok(())
}
```

### Message Subscription

```rust
pub async fn subscribe_to_messages<T, F>(
    nats: &async_nats::Client,
    topic: &str,
    handler: F,
) -> anyhow::Result<()>
where
    T: DeserializeOwned + Send + 'static,
    F: Fn(MessageEnvelope<T>) -> anyhow::Result<()> + Send + 'static,
{
    let mut subscriber = nats.subscribe(topic).await?;

    tokio::spawn(async move {
        while let Some(message) = subscriber.next().await {
            if let Ok(envelope) = serde_json::from_slice::<MessageEnvelope<T>>(&message.payload) {
                if let Err(e) = handler(envelope) {
                    tracing::error!("Message handler failed: {}", e);
                }
            }
        }
    });

    Ok(())
}
```

## Provider System Pattern

Wireframe-AI uses a unified Provider trait for LLM providers:

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    async fn chat_completion(&self, request: ChatRequest) -> Result<ChatResponse, ProviderError>;
    async fn stream_completion(&self, request: ChatRequest) -> Result<Pin<Box<dyn Stream<Item = Result<String, ProviderError>> + Send>>, ProviderError>;
    fn capabilities(&self) -> ProviderCapabilities;
    fn provider_type(&self) -> ProviderType;
}

pub struct ProviderCapabilities {
    pub supports_streaming: bool,
    pub supports_function_calling: bool,
    pub max_tokens: Option<usize>,
    pub supported_models: Vec<String>,
}
```

## Schema Validation Pattern

Validate schemas against contracts before use:

```rust
pub fn validate_schema<T>(schema: &Value) -> anyhow::Result<()>
where
    T: DeserializeOwned,
{
    let schema_str = serde_json::to_string(schema)?;
    let _: T = serde_json::from_str(&schema_str)?;
    Ok(())
}
```

## Context State Ownership Pattern

The Context module owns all persistent state:

```rust
pub struct Context {
    db: SqlitePool,
    nats: async_nats::Client,
}

impl Context {
    pub async fn new(db_path: &str, nats_url: &str) -> anyhow::Result<Self> {
        let db = SqlitePoolOptions::new().connect(db_path).await?;
        let nats = async_nats::connect(nats_url).await?;
        Ok(Self { db, nats })
    }

    pub async fn get_state<T>(&self, key: &str) -> anyhow::Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        // Implementation
        todo!()
    }

    pub async fn set_state<T>(&self, key: &str, value: T) -> anyhow::Result<()>
    where
        T: Serialize,
    {
        // Implementation
        todo!()
    }
}
```

## References

See `rust-patterns.md` for general Rust patterns including Repository, Service Layer, Newtype, and Builder patterns.
See skill: `rust-pro` for comprehensive Rust patterns including ownership, traits, generics, concurrency, and async.
See AGENTS.md for Wireframe-AI specific patterns and conventions.