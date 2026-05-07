---
paths:
  - "**/*.rs"
---
# Wireframe-AI Specific Testing

Wireframe-AI specific testing patterns for NATS messaging, Provider system, and Schema validation.

## NATS Testing

### Test NATS Server

Use a test NATS server for integration tests:

```rust
#[tokio::test]
async fn test_nats_message_publish() {
    // Start test NATS server
    let server = nats_server::run_test_server().await;

    let client = async_nats::connect(server.url()).await.unwrap();

    // Subscribe to test topic
    let mut sub = client.subscribe("test.topic").await.unwrap();

    // Publish message
    client.publish("test.topic", b"test payload").await.unwrap();

    // Verify message received
    let msg = sub.next().await.unwrap();
    assert_eq!(msg.payload, b"test payload");

    server.shutdown().await;
}
```

### Mock NATS for Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Mock NATS client trait
    #[async_trait]
    pub trait NatsClient: Send + Sync {
        async fn publish(&self, topic: &str, payload: &[u8]) -> anyhow::Result<()>;
        async fn subscribe(&self, topic: &str) -> anyhow::Result<Box<dyn Subscription>>;
    }

    struct MockNatsClient {
        published_messages: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
    }

    #[async_trait]
    impl NatsClient for MockNatsClient {
        async fn publish(&self, topic: &str, payload: &[u8]) -> anyhow::Result<()> {
            self.published_messages.lock().await.push((topic.to_string(), payload.to_vec()));
            Ok(())
        }

        async fn subscribe(&self, _topic: &str) -> anyhow::Result<Box<dyn Subscription>> {
            todo!()
        }
    }

    #[tokio::test]
    async fn module_publishes_online_message() {
        let messages = Arc::new(Mutex::new(Vec::new()));
        let mock = MockNatsClient {
            published_messages: messages.clone(),
        };

        let module = TestModule::new(Box::new(mock));
        module.start().await.unwrap();

        let published = messages.lock().await;
        assert_eq!(published.len(), 1);
        assert_eq!(published[0].0, "sys.module.online");
    }
}
```

## Provider Testing

### Mock Provider for Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    struct MockProvider {
        responses: Arc<Mutex<Vec<ChatResponse>>>,
    }

    #[async_trait]
    impl Provider for MockProvider {
        async fn chat_completion(&self, _request: ChatRequest) -> Result<ChatResponse, ProviderError> {
            let mut responses = self.responses.lock().await;
            Ok(responses.remove(0))
        }

        async fn stream_completion(&self, _request: ChatRequest) -> Result<Pin<Box<dyn Stream<Item = Result<String, ProviderError>> + Send>>, ProviderError> {
            todo!()
        }

        fn capabilities(&self) -> ProviderCapabilities {
            ProviderCapabilities {
                supports_streaming: true,
                supports_function_calling: true,
                max_tokens: Some(4096),
                supported_models: vec!["test-model".to_string()],
            }
        }

        fn provider_type(&self) -> ProviderType {
            ProviderType::OpenAI
        }
    }

    #[tokio::test]
    async fn test_provider_chat_completion() {
        let expected_response = ChatResponse {
            content: "Test response".to_string(),
            model: "test-model".to_string(),
            usage: Usage { prompt_tokens: 10, completion_tokens: 5 },
        };

        let responses = Arc::new(Mutex::new(vec![expected_response.clone()]));
        let provider = MockProvider { responses };

        let request = ChatRequest {
            messages: vec![Message::user("Test")],
            model: "test-model".to_string(),
        };

        let response = provider.chat_completion(request).await.unwrap();
        assert_eq!(response.content, expected_response.content);
    }
}
```

## Schema Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_validation() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "age": { "type": "number" }
            },
            "required": ["name"]
        });

        let valid_data = json!({
            "name": "Alice",
            "age": 30
        });

        let result = validate_against_schema(&schema, &valid_data);
        assert!(result.is_ok());

        let invalid_data = json!({
            "age": 30  // Missing required "name" field
        });

        let result = validate_against_schema(&schema, &invalid_data);
        assert!(result.is_err());
    }
}
```

## Module Integration Testing

```rust
// tests/module_test.rs
use wireframe_ai::context::Context;
use wireframe_ai::orchestrator::Orchestrator;

#[tokio::test]
async fn test_module_communication() {
    // Set up test environment
    let nats = async_nats::connect("nats://localhost:4222").await.unwrap();
    let context = Context::new(":memory:", "nats://localhost:4222").await.unwrap();

    // Start modules
    let orchestrator = Orchestrator::new(nats.clone(), context.clone()).await;
    orchestrator.start().await.unwrap();

    // Send test message
    let test_message = json!({"test": "data"});
    nats.publish("test.input", serde_json::to_vec(&test_message).unwrap())
        .await
        .unwrap();

    // Verify response
    let mut sub = nats.subscribe("test.output").await.unwrap();
    let msg = tokio::time::timeout(Duration::from_secs(5), sub.next())
        .await
        .unwrap()
        .unwrap();

    let response: serde_json::Value = serde_json::from_slice(&msg.payload).unwrap();
    assert_eq!(response["test"], "data");

    // Cleanup
    orchestrator.stop().await.unwrap();
}
```

## Wireframe-AI Specific Test Utilities

### Test Fixtures

```rust
// tests/common/mod.rs
pub fn test_nats_url() -> String {
    std::env::var("TEST_NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string())
}

pub fn test_db_path() -> String {
    std::env::var("TEST_DB_PATH").unwrap_or_else(|_| ":memory:".to_string())
}

pub async fn setup_test_environment() -> (async_nats::Client, Context) {
    let nats = async_nats::connect(&test_nats_url()).await.unwrap();
    let context = Context::new(&test_db_path(), &test_nats_url()).await.unwrap();
    (nats, context)
}

pub async fn teardown_test_environment(nats: async_nats::Client, context: Context) {
    context.cleanup().await.unwrap();
    // NATS cleanup handled by server shutdown
}
```

## References

See `rust-testing.md` for general Rust testing patterns including unit tests, parameterized tests, and mocking.
See skill: `run-rust-tests` for comprehensive testing patterns including property-based testing, fixtures, and benchmarking with Criterion.
See skill: `test-driven-development` for TDD workflow in Wireframe-AI.