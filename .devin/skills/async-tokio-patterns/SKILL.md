---
name: async-tokio-patterns
description: Tokio async/await patterns for Rust applications
allowed-tools:
  - read
  - grep
  - glob
  - edit
  - write
  - exec
triggers:
  - model
---

# Tokio Async Patterns

Comprehensive patterns for async/await programming with Tokio in Rust, including process spawning, IPC communication, signal handling, and resource management.

## When to Use
- Spawning external processes
- IPC communication between components
- Signal handling for graceful shutdown
- Async file operations
- Managing resource lifecycle asynchronously

## Key Patterns

### Runtime Setup
```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Main entry point with async runtime
}
```

### Process Spawning
```rust
use tokio::process::Command;

let child = Command::new("executable")
    .arg("--option")
    .arg(&value)
    .spawn()?;

// Track process with PID file or handle
```

### IPC Communication
```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel(100);

tokio::spawn(async move {
    daemon::run(rx).await;
});
```

### Signal Handling
```rust
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    tokio::select! {
        _ = signal::ctrl_c() => {
            cleanup().await?;
        }
        result = run_application() => {
            result
        }
    }
}
```

### Resource Cleanup with scopeguard
```rust
use scopeguard::defer;

let child = Command::new("executable").spawn()?;
defer! {
    // Kill process on exit
    let _ = child.kill();
}
```

## Domain-Specific Patterns

### Async Process Management
- Spawn external processes as subprocess
- Monitor process health via PID files or handles
- Kill subprocess on graceful shutdown
- Handle abnormal termination scenarios

### IPC for Component Communication
- Use mpsc channels for daemon communication
- Send status updates (memory usage, state)
- Handle component lifecycle independently

### Async File Operations
- Prefer async I/O for file operations where possible
- Batch operations when monitoring resources
- Use async-aware file system operations

## Common Pitfalls
- Blocking async runtime with synchronous API calls
- Not killing processes on shutdown (orphan processes)
- Leaking resources when process crashes
- Not propagating cancellation to spawned tasks
- Forgetting to await spawned tasks

## Performance Considerations
- Use `tokio::sync::mpsc` for IPC
- Prefer async I/O for file operations where possible
- Batch operations when monitoring resources
- Keep signal handling lightweight

## Related Files
- Process spawning and lifecycle management
- IPC channel implementation
- Signal handling and shutdown coordination
- Daemon async loops
- Resource async management
- Async file operations

## Related Skills
- `../workflows/debug.md` - For detecting async resource leaks
- `../skills/code-review-checklist/SKILL.md` - For reviewing async code patterns
- `../skills/code-fix/SKILL.md` - For refactoring async code
- `../skills/rust-pro/SKILL.md` - For general Rust async programming
- `../skills/error-handling/SKILL.md` - For error handling in async contexts
