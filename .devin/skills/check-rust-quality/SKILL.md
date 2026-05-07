---
name: check-rust-quality
description: Check Rust code quality with clippy and fmt
allowed-tools:
  - read
  - grep
  - glob
  - exec
triggers:
  - model
---

# Check Rust Quality

## Purpose

Validate Rust code quality using clippy (linter) and fmt (formatter). Ensures code follows Rust best practices, catches potential bugs, and maintains consistent formatting across the Wireframe-AI codebase.

## When to Use

**Use this skill when:**
- Checking code quality before committing
- Reviewing code changes for style issues
- Running CI/CD quality gates
- Preparing code for review or merge
- Validating refactoring changes

**Do NOT use when:**
- Running tests (use `/run-rust-tests` instead)
- Building for production (use `/build-release` instead)
- Checking security vulnerabilities (use `/security-auditor` instead)

## Protocol

### Phase 1: Clippy Linting

1. **Run clippy with strict warnings**
   ```bash
   cargo clippy -- -D warnings
   ```
   - Checks for common Rust mistakes and anti-patterns
   - Treats warnings as errors with `-D warnings`
   - Provides suggestions for improvements

2. **Analyze clippy output**
   - Count warnings/errors by severity
   - Group by file for systematic fixing
   - Identify patterns (e.g., repeated unused variables)

3. **Run clippy on specific targets** (if needed)
   ```bash
   cargo clippy --bin kernel              # Check binary only
   cargo clippy --lib                     # Check library only
   cargo clippy -p context_module         # Check specific module
   ```

### Phase 2: Formatting Check

1. **Check formatting without modifying**
   ```bash
   cargo fmt --check
   ```
   - Verifies code is formatted correctly
   - Fails if any files need formatting
   - Does not modify files (use `cargo fmt` to fix)

2. **Analyze formatting issues**
   - List files that need formatting
   - Check if formatting is consistent across project

3. **Offer to format** (if issues found)
   ```bash
   cargo fmt
   ```
   - Automatically formats all Rust files
   - Uses standard Rust formatting rules

### Phase 3: Report Findings

1. **Summarize clippy results**
   - Total warnings/errors found
   - File paths and line numbers
   - Severity assessment (warnings vs errors)

2. **Summarize formatting results**
   - Files that need formatting
   - Whether formatting check passed

3. **Provide actionable suggestions**
   - Suggested fixes for common clippy warnings
   - Formatting commands to run
   - Priority order for fixing issues

## Key Checks

| Check Area | Pattern | Correct Usage |
|------------|---------|---------------|
| Async/await | Entry points | Use `#[tokio::main]` for async main functions |
| Error handling | Application code | Use `anyhow::Context` for error context |
| Error handling | Library code | Use `thiserror` for custom error types |
| NATS usage | Client creation | Prefer `async-nats` crate |
| Schema enforcement | Message envelopes | Use serde for envelope serialization |
| Module macros | Module registration | Use `#[module]` from agentic-sdk-macros |
| Unsafe code | Safety justification | Every `unsafe` block must have `// SAFETY:` comment |
| Unwrap usage | Production code | Never use `unwrap()`, use `?` or handle errors |

## Common Clippy Warnings

| Warning | Cause | Fix |
|---------|-------|-----|
| `unused_variable` | Variable declared but not used | Prefix with `_` or use it |
| `dead_code` | Function/struct never used | Mark with `#[allow(dead_code)]` or use it |
| `clippy::unwrap_used` | Using `unwrap()` in code | Replace with `?` or proper error handling |
| `clippy::expect_used` | Using `expect()` in code | Replace with proper error handling |
| `clippy::panic` | Code that can panic | Add error handling instead |
| `clippy::todo!` | Incomplete implementation | Implement the function or mark as unimplemented |
| `clippy::unimplemented!` | Placeholder code | Implement the function |
| `clippy::clone_on_copy` | Cloning a Copy type | Use copy instead of clone |

## Common Formatting Issues

| Issue | Cause | Fix |
|-------|-------|-----|
| Inconsistent indentation | Mixed tabs/spaces | Run `cargo fmt` to standardize |
| Line too long | Lines over 100 chars | Break into multiple lines |
| Trailing whitespace | Spaces at line end | Run `cargo fmt` to remove |
| Inconsistent brace style | Mixed brace placement | Run `cargo fmt` to standardize |
| Missing blank lines | Required spacing | Run `cargo fmt` to add |

## Verification Examples

**Example: Running clippy**

```bash
cargo clippy -- -D warnings
# Output:
#   warning: unused variable: `x`
#    --> modules/context/src/lib.rs:42:9
#     |
#  42 |     let x = 42;
#     |         ^ help: if it is intentional, prefix it with an underscore: `_x`
#     |
#     = note: `#[warn(unused_variables)]` on by default
#
#   warning: this `else { if ... }` can be collapsed
#    --> modules/orchestrator/src/lib.rs:78:12
#     |
#  78 |     } else if condition {
#     |            ^^^^^^^^^^ help: collapse it into: `} if condition {`
#     |
#     = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#else_if_without_else
```

**Example: Checking formatting**

```bash
cargo fmt --check
# Output:
#   Diff in /home/user/Wireframe-AI/modules/context/src/lib.rs at line 42:
#   -         let x = 42;
#   +         let x = 42;
#   warning: the following files were not formatted: modules/context/src/lib.rs
```

**Example: Fixing formatting**

```bash
cargo fmt
# Output:
#   Left modules/context/src/lib.rs
#   Left modules/orchestrator/src/lib.rs
```

## Integration

**Related skills:**
- **superpowers:run-rust-tests** - Run test suite after quality checks
- **superpowers:build-release** - Build in release mode after quality checks
- **superpowers:quality-checklist** - Comprehensive quality checks
- **superpowers:rust-pro** - Rust patterns and best practices

**Workflow context:**
- Use before committing changes
- Use as part of code review process
- Use with `/final-checks` before deployment
- Use after refactoring or new feature implementation
