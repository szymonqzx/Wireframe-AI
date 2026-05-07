# Testing Strategy - Advanced Topics

Advanced testing topics including running tests, test data management, quality gates, CI/CD integration, and test maintenance.

## Running Tests

### Unit Tests
```bash
# Run all unit tests
cargo test

# Run specific module tests
cargo test --package context-module

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_message_processing
```

### Integration Tests
```bash
# Start test NATS server
nats-server -p 4223 -js

# Run integration tests
cargo test --test integration

# Run specific integration test
cargo test --test integration test_context_module_publishes_online
```

### Python Tests
```bash
# Run all Python tests
pytest adapter/python/tests/

# Run with coverage
pytest adapter/python/tests/ --cov=adapter

# Run specific test
pytest adapter/python/tests/test_adapter.py::test_message_handling
```

### E2E Tests
```bash
# Run E2E tests
pytest tests/e2e/

# Run with headless browser
pytest tests/e2e/ --headless

# Run specific E2E test
pytest tests/e2e/test_session.py::test_session_creation
```

## Test Data Management

### Fixtures

**Rust:**
```rust
// tests/fixtures/mod.rs
pub fn create_test_envelope() -> Envelope {
    Envelope {
        id: Uuid::new_v4(),
        source: "test".to_string(),
        timestamp: Utc::now(),
        payload: Payload::Test(TestPayload::default()),
    }
}
```

**Python:**
```python
# tests/fixtures.py
@pytest.fixture
def test_envelope():
    return {
        "id": str(uuid.uuid4()),
        "source": "test",
        "timestamp": datetime.utcnow().isoformat(),
        "payload": {"type": "test"}
    }
```

### Golden Files (Regression Tests)

**Purpose:** Store expected outputs for critical behavior to prevent regressions.

**Pattern:**
```rust
#[test]
fn test_critical_behavior() {
    let input = get_test_input();
    let output = process(input);

    let expected = fs::read_to_string("tests/golden/critical_behavior.txt")
        .expect("Golden file missing");

    assert_eq!(output, expected.trim());
}
```

**Updating Golden Files:**
```bash
# Regenerate golden files (only when behavior change is intentional)
cargo test -- --accept
```

## Quality Gates

### Pre-Commit
- All unit tests pass
- Code compiles without warnings
- Clippy passes with no warnings
- Format check passes

### Pre-Merge
- All integration tests pass
- Code coverage > 80% for new code
- No regressions in golden file tests
- E2E tests for critical features pass

### Pre-Release
- Full test suite passes
- Performance benchmarks meet thresholds
- Security scans pass
- Documentation is up to date

## Continuous Testing

### CI/CD Integration

**GitHub Actions Example:**
```yaml
name: Test

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - run: cargo test --workspace

  integration-tests:
    runs-on: ubuntu-latest
    services:
      nats:
        image: nats:latest
        ports:
          - 4222:4222
    steps:
      - uses: actions/checkout@v3
      - run: cargo test --test integration

  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: pytest tests/e2e/
```

## Test Maintenance

### When Tests Fail

1. **Isolate the failure:** Run the specific test alone
2. **Check the environment:** Ensure dependencies are running
3. **Review recent changes:** Identify what broke the test
4. **Fix the code OR the test:**
   - If the behavior changed intentionally → update the test
   - If the behavior changed unintentionally → fix the code
5. **Add regression test:** If this was a bug, add a test to prevent recurrence

### Flaky Tests

**Causes:**
- Race conditions in async code
- Timing dependencies (sleep vs wait)
- External service unavailability
- Non-deterministic test data

**Solutions:**
- Use explicit waits instead of sleep
- Mock external dependencies
- Make tests deterministic (seed random data)
- Retry flaky tests with exponential backoff

### Test Debt

**Track in TODO.md:**
```markdown
## Test Debt
- [ ] Add integration test for NATS message ordering
- [ ] Add E2E test for error recovery workflow
- [ ] Increase coverage for error handling paths
```

## Related Resources

See `@[skills/testing/SKILL.md]` for testing philosophy, the testing pyramid, and Wireframe-AI testing patterns.

See `@[skills/testing/SKILL-examples.md]` for testing anti-patterns, metrics, and related resources.