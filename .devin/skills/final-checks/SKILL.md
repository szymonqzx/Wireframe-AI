---
name: final-checks
description: Final checklist protocol for Wireframe-AI - build, schema validation, integration tests, benchmarks
allowed-tools:
  - exec
  - read
triggers:
  - model
---

# Final Checks

## Purpose

Execute comprehensive final verification before deployment or completion. Ensures build succeeds, schemas are valid, integration tests pass, benchmarks meet performance targets, and cross-platform compatibility is verified.

## When to Use

**Use this skill when:**
- Completing a feature or bug fix before merging
- Preparing for deployment to production or staging
- Running CI/CD final verification gates
- Validating release candidates
- Completing development work before PR

**Do NOT use when:**
- Running quick development tests (use `/run-rust-tests` instead)
- Checking code style only (use `/check-rust-quality` instead)
- Building without testing (use `/build-release` instead)

## Protocol

### Phase 1: Build Verification

1. **Run release build**
   ```bash
   cargo build --release
   ```
   - Verify all modules compile successfully
   - Check for any compilation warnings
   - Ensure release optimizations work correctly

2. **Analyze build output**
   - Check for warnings (treat as errors if strict)
   - Verify binary sizes are reasonable
   - Note any deprecation warnings

3. **Fix build failures first**
   - Address compilation errors before proceeding
   - Resolve warnings if strict mode enabled
   - Re-run build to verify fixes

### Phase 2: Schema Validation

1. **Validate schema contracts**
   ```bash
   python scripts/validate_schemas.py
   ```
   - Ensure envelope contracts are valid
   - Verify schema changes are backward compatible
   - Check all modules use correct schema versions

2. **Analyze schema validation results**
   - Identify any contract violations
   - Check for breaking changes
   - Verify migration paths exist if needed

### Phase 3: Integration Tests

1. **Run integration tests**
   ```bash
   cargo test --test integration_test
   ```
   - Test end-to-end module communication via NATS
   - Verify message envelope contracts
   - Ensure NATS server is running

2. **Analyze integration test results**
   - Check for module-to-module communication failures
   - Verify message flows work correctly
   - Identify any timeout or connection issues

### Phase 4: Benchmark Tests

1. **Run benchmark tests**
   ```bash
   cargo test --test benchmark_test
   ```
   - Validate performance targets are met
   - Check for performance regressions
   - Compare against baseline metrics

2. **Analyze benchmark results**
   - Identify performance regressions
   - Verify targets are within acceptable ranges
   - Note any significant performance changes

### Phase 5: Python SDK Test

1. **Run Python SDK tests**
   ```bash
   python tests/test_python_sdk.py
   ```
   - Verify Python adapter compatibility
   - Test SDK integration with Rust modules
   - Ensure Python-Rust interop works

2. **Analyze Python SDK results**
   - Check for Python-Rust integration failures
   - Verify SDK API compatibility
   - Identify any serialization issues

### Phase 6: Cross-Platform Build (Optional)

1. **Run cross-platform build**
   ```bash
   cross build --release
   ```
   - Verify multi-platform support (Linux, macOS, Windows)
   - Check platform-specific issues
   - Ensure dependencies work across platforms

2. **Analyze cross-platform results**
   - Identify platform-specific failures
   - Verify all target platforms build successfully
   - Note any platform-specific warnings

### Phase 7: Report Results

1. **Summarize all check results**
   - Build status (pass/fail)
   - Schema validation status
   - Integration test status
   - Benchmark status
   - Python SDK status
   - Cross-platform status (if run)

2. **Report failures with details**
   - File paths and line numbers for failures
   - Command outputs for debugging
   - Suggested fixes for common issues

3. **Declare completion only when all pass**
   - Task is not finished until all checks pass
   - Fix failures in priority order
   - Re-run failed checks after fixes

## Checklist Order

| Stage | Command | Purpose | Priority |
|-------|---------|---------|----------|
| Build Verification | `cargo build --release` | Verify all modules compile | 1 (highest) |
| Schema Validation | `python scripts/validate_schemas.py` | Ensure envelope/contract compliance | 2 |
| Integration Tests | `cargo test --test integration_test` | End-to-end module communication | 3 |
| Benchmark Tests | `cargo test --test benchmark_test` | Performance validation | 4 |
| Python SDK Test | `python tests/test_python_sdk.py` | Python adapter compatibility | 5 |
| Cross-Platform Build | `cross build --release` (optional) | Verify multi-platform support | 6 (optional) |

## Common Failures and Fixes

| Failure Type | Common Cause | Fix |
|--------------|--------------|-----|
| Build compilation error | Syntax error or missing dependency | Fix syntax or add dependency with `cargo add` |
| Schema validation error | Breaking change in envelope contract | Revert change or provide migration path |
| Integration test timeout | NATS server not running or topic mismatch | Start NATS server or verify topic naming |
| Benchmark regression | Performance degradation in recent change | Profile and optimize or revert change |
| Python SDK failure | Serialization mismatch or API change | Update Python SDK or Rust interface |
| Cross-platform build error | Platform-specific dependency issue | Add platform-specific conditionals or alternative |

## Anti-Patterns

| Anti-Pattern | Why Bad | Correct Approach |
|-------------|---------|------------------|
| Skipping build verification | Compilation errors may exist | Always run build first |
| Running tests out of order | Early failures waste time | Follow priority order |
| Ignoring warnings | Warnings may indicate bugs | Treat warnings as errors in strict mode |
| Not re-running after fixes | Fixes may introduce new issues | Re-run all checks after each fix |
| Declaring success with failures | Incomplete verification | Only declare success when all pass |
| Running unnecessary checks | Wastes time on irrelevant checks | Use narrowest relevant check for validation |

## Verification Examples

**Example: Running full final checks**

```bash
# Phase 1: Build Verification
cargo build --release
# Output: Finished release [optimized] target(s) in 2m 45s

# Phase 2: Schema Validation
python scripts/validate_schemas.py
# Output: All schemas validated successfully

# Phase 3: Integration Tests
cargo test --test integration_test
# Output: test result: ok. 15 passed; 0 failed

# Phase 4: Benchmark Tests
cargo test --test benchmark_test
# Output: test result: ok. 8 passed; 0 failed

# Phase 5: Python SDK Test
python tests/test_python_sdk.py
# Output: All Python SDK tests passed
```

**Example: Handling build failure**

```bash
cargo build --release
# Output: error[E0425]: cannot find value `x` in this scope
#    --> modules/context/src/lib.rs:42:9
#
# Fix: Add variable declaration or rename
# Re-run build
cargo build --release
# Output: Finished release [optimized] target(s) in 2m 45s
```

## Integration

**Related skills:**
- **superpowers:build-release** - Build in release mode
- **superpowers:run-rust-tests** - Run Rust test suite
- **superpowers:check-rust-quality** - Check code quality
- **superpowers:quality-checklist** - Comprehensive quality checks

**Required workflow skills:**
- **superpowers:finishing-a-development-branch** - Use this skill before completing development work

**Workflow context:**
- Use as final verification before deployment
- Use before merging PRs
- Use as part of CI/CD pipeline
- Use with `/karpathy-guidelines` for evidence-based completion
