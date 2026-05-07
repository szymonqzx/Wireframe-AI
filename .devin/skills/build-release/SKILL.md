---
name: build-release
description: Build Wireframe-AI in release mode
allowed-tools:
  - read
  - grep
  - glob
  - exec
triggers:
  - model
---

# Build Release

## Purpose

Build Wireframe-AI in release mode with optimizations enabled. Ensure all workspace crates compile successfully, identify compilation errors and warnings, and verify that the build produces all required binaries for deployment.

## When to Use

Use this skill when:
- Building Wireframe-AI for production deployment
- Creating release binaries for distribution
- Verifying that all modules compile successfully
- Testing performance optimizations
- After making changes that affect multiple crates
- Before packaging or deploying the application

## Protocol

### Step 1: Pre-Build Verification

1. **Check Git State**
   ```bash
   git status
   ```
   Ensure working tree is clean before building

2. **Update Dependencies**
   ```bash
   cargo update
   ```
   Ensure dependencies are up to date

### Step 2: Execute Release Build

1. **Run Release Build**
   ```bash
   cargo build --release
   ```
   This builds all workspace crates with optimizations enabled

2. **Monitor Build Progress**
   - Watch for compilation errors
   - Note any warnings that appear
   - Track which crates are being built

### Step 3: Verify Build Success

1. **Check Exit Code**
   - Exit code 0 indicates success
   - Non-zero exit code indicates failure

2. **Verify Binary Output**
   ```bash
   ls -la target/release/
   ```
   Check that expected binaries are present:
   - Kernel binaries
   - Module binaries (orchestrator, context, sandbox)
   - SDK crates

### Step 4: Analyze Build Output

1. **Check for Errors**
   - Review any compilation errors
   - Note file:line references for errors
   - Identify which crate failed to build

2. **Assess Warnings**
   - Review compiler warnings
   - Assess severity (critical vs informational)
   - Note patterns that might indicate issues

### Step 5: Report Results

1. **Build Status**
   - Report success or failure
   - Include total build time if available
   - Note any performance observations

2. **Error Reporting**
   - List compilation errors with file:line references
   - Provide context for each error
   - Suggest potential fixes if applicable

3. **Warning Summary**
   - Count total warnings
   - Highlight critical warnings
   - Note any warning patterns

## Build Targets

| Component | Binary Location | Purpose |
|-----------|----------------|---------|
| Kernel | `target/release/kernel` | Main orchestrator binary |
| Context Module | `target/release/context` | State management module |
| Orchestrator Module | `target/release/orchestrator` | Task orchestration module |
| Sandbox Module | `target/release/sandbox` | Code execution module |
| agentic-sdk | `target/release/libagentic_sdk.*` | Rust SDK library |
| agentic-sdk-macros | `target/release/libagentic_sdk_macros.*` | SDK procedural macros |
| agentic-sdk-py | Python package | Python adapter |

## Common Build Issues

| Issue | Cause | Resolution |
|-------|--------|------------|
| Linker errors | Missing dependencies or incorrect paths | Run `cargo update` and check Cargo.toml |
| Compilation errors | Syntax errors or type mismatches | Check file:line references in error output |
| Out of memory | Insufficient RAM for release build | Close other applications or use debug build |
| Timeout | Slow build or infinite loop | Check for circular dependencies |
| Missing binary | Crate not included in workspace | Verify workspace members in Cargo.toml |

## Build Optimization Tips

- Use `--release` flag for production builds
- Use `--debug` for faster builds during development
- Parallel builds are automatic with cargo
- Consider `cargo build --release --bin <name>` for single binary
- Use `cargo clean` before rebuilding if experiencing cache issues

## Verification Examples

```bash
# Successful build output
   Compiling wireframe-kernel v0.1.0
   Compiling wireframe-context v0.1.0
   Compiling wireframe-orchestrator v0.1.0
    Finished release [optimized] target(s) in 2m 45s

# Verify binaries exist
$ ls target/release/
kernel  context  orchestrator  sandbox
```

## Integration

This skill integrates with:
- `/run-rust-tests` - Run tests before building release
- `/check-rust-quality` - Check code quality before building
- `/final-checks` - Comprehensive verification before release
