---
name: run-rust-tests
description: Run Rust test suite for Wireframe-AI
allowed-tools:
  - read
  - grep
  - glob
  - exec
triggers:
  - model
---

# Run Rust Tests

## Purpose

Execute the complete Rust test suite for Wireframe-AI, including unit tests, integration tests, and release builds. Ensures code quality, validates module interactions via NATS, and verifies schema contracts before deployment.

## When to Use

**Use this skill when:**
- Running the full test suite before committing code
- Verifying fixes for bugs or new features
- Validating changes to NATS messaging or schemas
- Running CI/CD test pipelines
- Checking release build compatibility

**Do NOT use when:**
- Only checking code style (use `/check-rust-quality` instead)
- Building for production without testing (use `/build-release` instead)
- Running specific test subsets (use `cargo test <test_name>` directly)

## Protocol

### Phase 1: Pre-Test Verification

1. **Check NATS server status**
   - Verify NATS server is running (required for integration tests)
   - If not running: Start NATS server or skip integration tests

2. **Check for test-specific dependencies**
   - Verify database files are available (if tests use SQLite)
   - Check for required environment variables (NATS_URL, DATABASE_URL)

### Phase 2: Execute Test Suite

1. **Run unit tests (debug mode)**
   ```bash
   cargo test
   ```
   - Runs all unit tests in module crates
   - Includes doc tests and integration tests
   - Faster than release mode for development

2. **Run release tests**
   ```bash
   cargo test --release
   ```
   - Tests with release optimizations
   - Catches release-mode specific issues
   - Slower but more representative of production

3. **Run integration tests separately** (if needed)
   ```bash
   cargo test --test integration
   ```
   - Tests module-to-module communication via NATS
   - Validates message envelope contracts
   - Requires NATS server running

4. **Run specific test categories** (if specified)
   ```bash
   cargo test --lib                    # Library tests only
   cargo test --bins                   # Binary tests only
   cargo test <test_name>              # Specific test
   cargo test <module_name>            # Module-specific tests
   ```

### Phase 3: Analyze Results

1. **Check test summary**
   - Count tests passed/failed
   - Identify any panics or crashes
   - Note skipped tests (if any)

2. **Analyze failures**
   - Extract file paths and line numbers from failure output
   - Identify failure patterns (assertion errors, panics, timeouts)
   - Check for flaky tests (intermittent failures)

3. **Report findings**
   - Total test count and pass rate
   - Specific failures with file:line references
   - Suggestions for fixing common failure patterns
   - Coverage gaps if detected

## Test Categories

| Category | Location | Purpose | Dependencies |
|----------|----------|---------|--------------|
| Unit tests | `modules/*/src/lib.rs` (#[cfg(test)]) | Test individual functions and modules | None |
| Integration tests | `tests/` directory | Test module-to-module communication via NATS | NATS server |
| Benchmark tests | `benches/` directory | Performance validation and regression testing | None |
| Schema validation tests | `tests/` or module tests | Validate against `schemas/v1/` contracts | Schema files |
| Doc tests | In documentation comments | Ensure code examples work | None |

## Common Failure Patterns

| Failure Pattern | Likely Cause | Resolution |
|----------------|--------------|------------|
| `connection refused` | NATS server not running | Start NATS server with `nats-server` |
| `database locked` | SQLite file in use by another process | Close other connections or use :memory: database |
| `timeout` | Test waiting for message that never arrives | Check NATS topic naming and subscription setup |
| `assertion failed` | Logic error in code or test | Review test expectations and implementation |
| `thread 'main' panicked` | Unhandled error or unwrap() | Add proper error handling with `?` operator |
| `borrow checker error` | Ownership issue in test code | Use clones or adjust test structure |

## Verification Examples

**Example: Running full test suite**

```bash
# Check NATS is running
nats-server -p 4222 &

# Run unit tests
cargo test
# Output: test result: ok. 42 passed; 0 failed

# Run release tests
cargo test --release
# Output: test result: ok. 42 passed; 0 failed
```

**Example: Running specific test**

```bash
# Test specific module
cargo test context
# Output: running 8 tests in context module

# Test specific function
cargo test test_get_state
# Output: running 1 test in context::tests
```

**Example: Analyzing failure output**

```bash
cargo test
# Output:
#   failures:
#       test_module_lifecycle
#   test tests/integration_test.rs:15 - module_lifecycle::test_module_lifecycle
#
#   failures:
#       module_lifecycle::test_module_lifecycle
#
# Test result: FAILED. 41 passed; 1 failed
```

## Integration

**Related skills:**
- **superpowers:check-rust-quality** - Run clippy and fmt for code style
- **superpowers:build-release** - Build in release mode
- **superpowers:quality-checklist** - Comprehensive quality checks
- **superpowers:systematic-debugging** - Debug test failures

**Workflow context:**
- Use after implementing features or fixing bugs
- Use before committing changes
- Use as part of CI/CD pipeline
- Use with `/final-checks` before deployment
