# Rust Testing Advanced Patterns

Advanced testing patterns for Wireframe-AI including integration tests, async testing, property-based testing, mocking, and Wireframe-AI specific testing patterns.

## Integration Tests

### File Structure

```text
wireframe-ai/
├── kernel/
│   └── modules/
│       └── context/
│           └── src/
│               └── lib.rs
├── tests/              # Integration tests
│   ├── context_test.rs     # Each file is a separate test binary
│   ├── nats_test.rs
│   └── common/         # Shared test utilities
│       └── mod.rs
```

### Writing Integration Tests

```rust
// tests/context_test.rs
use wireframe_ai::context::ContextManager;
use wireframe_ai::Config;

#[test]
fn full_context_lifecycle() {
    let config = Config::test_default();
    let context = ContextManager::new(config);

    let result = context.set_state("user:123", r#"{"name": "Alice"}"#);
    assert!(result.is_ok());

    let retrieved = context.get_state("user:123");
    assert!(retrieved.is_ok());
    assert_eq!(retrieved.unwrap(), r#"{"name": "Alice"}"#);
}
```

## Async Tests

### With Tokio

```rust
#[tokio::test]
async fn fetches_data_successfully() {
    let client = TestClient::new().await;
    let result = client.get("/data").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().items.len(), 3);
}

#[tokio::test]
async fn handles_timeout() {
    use std::time::Duration;
    let result = tokio::time::timeout(
        Duration::from_millis(100),
        slow_operation(),
    ).await;

    assert!(result.is_err(), "should have timed out");
}
```

## Test Organization Patterns

### Parameterized Tests with `rstest`

```rust
use rstest::{rstest, fixture};

#[rstest]
#[case("hello", 5)]
#[case("", 0)]
#[case("rust", 4)]
fn test_string_length(#[case] input: &str, #[case] expected: usize) {
    assert_eq!(input.len(), expected);
}

// Fixtures
#[fixture]
fn test_db() -> TestDb {
    TestDb::new_in_memory()
}

#[rstest]
fn test_insert(test_db: TestDb) {
    test_db.insert("key", "value");
    assert_eq!(test_db.get("key"), Some("value".into()));
}
```

### Test Helpers

```rust
#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a test user with sensible defaults.
    fn make_user(name: &str) -> User {
        User::new(name, &format!("{name}@test.com")).unwrap()
    }

    #[test]
    fn user_display() {
        let user = make_user("alice");
        assert_eq!(user.display_name(), "alice");
    }
}
```

## Property-Based Testing with `proptest`

### Basic Property Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn encode_decode_roundtrip(input in ".*") {
        let encoded = encode(&input);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(input, decoded);
    }

    #[test]
    fn sort_preserves_length(mut vec in prop::collection::vec(any::<i32>(), 0..100)) {
        let original_len = vec.len();
        vec.sort();
        assert_eq!(vec.len(), original_len);
    }

    #[test]
    fn sort_produces_ordered_output(mut vec in prop::collection::vec(any::<i32>(), 0..100)) {
        vec.sort();
        for window in vec.windows(2) {
            assert!(window[0] <= window[1]);
        }
    }
}
```

### Custom Strategies

```rust
use proptest::prelude::*;

fn valid_email() -> impl Strategy<Value = String> {
    ("[a-z]{1,10}", "[a-z]{1,5}")
        .prop_map(|(user, domain)| format!("{user}@{domain}.com"))
}

proptest! {
    #[test]
    fn accepts_valid_emails(email in valid_email()) {
        assert!(User::new("Test", &email).is_ok());
    }
}
```

## Mocking with `mockall`

### Trait-Based Mocking

```rust
use mockall::{automock, predicate::eq};

#[automock]
trait UserRepository {
    fn find_by_id(&self, id: u64) -> Option<User>;
    fn save(&self, user: &User) -> Result<(), StorageError>;
}

#[test]
fn service_returns_user_when_found() {
    let mut mock = MockUserRepository::new();
    mock.expect_find_by_id()
        .with(eq(42))
        .times(1)
        .returning(|_| Some(User { id: 42, name: "Alice".into() }));

    let service = UserService::new(Box::new(mock));
    let user = service.get_user(42).unwrap();
    assert_eq!(user.name, "Alice");
}

#[test]
fn service_returns_none_when_not_found() {
    let mut mock = MockUserRepository::new();
    mock.expect_find_by_id()
        .returning(|_| None);

    let service = UserService::new(Box::new(mock));
    assert!(service.get_user(99).is_none());
}
```

## Wireframe-AI Specific Testing

### Testing NATS Message Handling

```rust
#[cfg(test)]
mod nats_tests {
    use super::*;
    use async_nats::Client;

    #[tokio::test]
    async fn publishes_module_online_message() {
        let client = setup_test_nats_client().await;
        let module = TestModule::new("test-module");

        module.start(&client).await;

        // Verify message was published
        let subscriber = client.subscribe("sys.module.online").await.unwrap();
        let message = subscriber.next().await.unwrap();
        assert_eq!(message.payload, b"test-module");
    }

    #[tokio::test]
    async fn handles_nats_connection_failure() {
        let client = failing_nats_client();
        let module = TestModule::new("test-module");

        let result = module.start(&client).await;
        assert!(result.is_err());
    }
}
```

### Testing SQLite Database Operations

```rust
#[cfg(test)]
mod db_tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn creates_and_retrieves_state() {
        let pool = create_test_pool().await;
        let context = ContextManager::new(pool);

        context.set_state("test:key", r#"{"value": 42}"#).await.unwrap();

        let retrieved = context.get_state("test:key").await.unwrap();
        assert_eq!(retrieved, r#"{"value": 42}"#);
    }

    #[tokio::test]
    async fn transaction_rolls_back_on_error() {
        let pool = create_test_pool().await;
        let mut tx = pool.begin().await.unwrap();

        sqlx::query("INSERT INTO states (key, value) VALUES (?, ?)")
            .bind("test:key")
            .bind(r#"{"value": 42}"#)
            .execute(&mut *tx)
            .await
            .unwrap();

        // Simulate error
        tx.rollback().await.unwrap();

        // Verify data was not committed
        let result = sqlx::query("SELECT * FROM states WHERE key = ?")
            .bind("test:key")
            .fetch_optional(&pool)
            .await
            .unwrap();

        assert!(result.is_none());
    }
}
```

### Testing Provider System

```rust
#[cfg(test)]
mod provider_tests {
    use super::*;
    use mockall::mock;

    mock! {
        pub TestProvider {}

        #[async_trait::async_trait]
        impl Provider for TestProvider {
            async fn complete(&self, prompt: &str) -> Result<String, ProviderError>;
            async fn stream(&self, prompt: &str) -> Result<Pin<Box<dyn Stream<Item = String> + Send>>, ProviderError>;
        }
    }

    #[tokio::test]
    async fn provider_returns_completion() {
        let mut mock = MockTestProvider::new();
        mock.expect_complete()
            .returning(|_| Ok("test response".to_string()));

        let result = mock.complete("test prompt").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test response");
    }

    #[tokio::test]
    async fn provider_handles_rate_limit() {
        let mut mock = MockTestProvider::new();
        mock.expect_complete()
            .returning(|_| Err(ProviderError::RateLimit));

        let result = mock.complete("test prompt").await;
        assert!(matches!(result.unwrap_err(), ProviderError::RateLimit));
    }
}
```

### Testing Schema Validation

```rust
#[cfg(test)]
mod schema_tests {
    use super::*;
    use wireframe_ai::schemas::v1::Envelope;

    #[test]
    fn validates_envelope_structure() {
        let envelope = Envelope {
            version: "1.0".to_string(),
            timestamp: chrono::Utc::now(),
            payload: serde_json::json!({"test": "data"}),
        };

        let result = envelope.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn rejects_invalid_envelope_version() {
        let envelope = Envelope {
            version: "2.0".to_string(), // Invalid version
            timestamp: chrono::Utc::now(),
            payload: serde_json::json!({"test": "data"}),
        };

        let result = envelope.validate();
        assert!(result.is_err());
    }
}
```

## Reference

See `SKILL.md` for core testing patterns and TDD workflow.
See `rust-testing-examples.md` for doc tests, benchmarking, coverage, and CI integration.