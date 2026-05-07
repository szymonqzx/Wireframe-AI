# Wireframe-AI Best Practices

## Module Development

### Topic Naming

Follow the convention: `namespace.noun.verb` or `namespace.noun`

Examples:
- `task.submitted`
- `agent.job`
- `webhook.receive`
- `integration.github.issues.list`

### State Management

- **Stateless modules**: Preferred. Store state in Context module or external DB.
- **Stateful modules**: Use `tokio::sync::RwLock` or `Mutex` for in-memory state.
- **Persistent state**: Only the Context module should own persistent state.

### Error Handling

Use SDK error patterns:

```rust
use agentic_sdk::error::{SdkError, SdkResult, retry_with_backoff};

async fn fetch_data() -> SdkResult<String> {
    retry_with_backoff(3, || async {
        // operation
        Ok("data".to_string())
    }).await
}
```

### Logging & Tracing

- Use `tracing` macros: `tracing::info!`, `tracing::error!`
- Include `correlation_id` and `session_id` in every log line
- Set `RUST_LOG=info` in production, `RUST_LOG=debug` in development

### Schema Validation

Enable in production:

```bash
cargo build --release --features schema-validation
```

## Provider Development

### Adding a New Provider

1. Create crate: `providers/<name>/`
2. Implement `Provider` trait from `provider-core`
3. Add cost tracking via `cost_per_1k_tokens()`
4. Add to marketplace index in `provider-core/src/marketplace.rs`
5. Register in adapter's `ProviderRegistry`

### Capability Negotiation

Providers should declare:
- `core_methods`: `complete`, `describe`, `status`
- `features`: `streaming`, `tools`, `vision`
- `transport`: `http`, `stdio`, `grpc`

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_my_module() {
    let mut module = MyModule::new();
    let env = Envelope::new("test.topic", serde_json::json!({}), None);
    let results = module.handle(env).await;
    assert_eq!(results.len(), 1);
}
```

### Integration Tests

Use `wireframe-cli test`:

```bash
wireframe test my-module --integration
```

### Provider Conformance Tests

```rust
use wireframe_provider_core::testing::{MockProvider, ProviderTestHarness};

#[tokio::test]
async fn test_provider_conformance() {
    let provider = Arc::new(MockProvider::new("test", "model"));
    let mut harness = ProviderTestHarness::new(provider);
    harness.run_all().await;
    assert!(harness.results.iter().all(|r| r.passed));
}
```

## Performance

### Release Profile

Use the optimized release profile:

```bash
cargo build --release
```

For distribution builds:

```bash
cargo build --profile release-lto
```

### Message Size

Keep payloads under 1MB. For large files:
1. Store file in shared volume/S3
2. Pass reference (path/URL) in message

### Connection Pooling

Reuse HTTP clients (e.g., `reqwest::Client`) across requests.

## Security

- Validate all inputs with JSON schemas
- Use sandbox mode for untrusted code execution
- Rotate API keys regularly
- Enable audit logging in production
- Never log secrets or API keys
