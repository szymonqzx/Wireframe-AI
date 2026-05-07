---
name: error-handling
description: Error handling patterns using anyhow and thiserror in Rust applications
allowed-tools:
  - read
  - grep
  - glob
  - edit
  - write
triggers:
  - model
---

# Error Handling in Rust

Comprehensive patterns for error handling in Rust applications using thiserror for library errors and anyhow for application-level errors, including Windows HRESULT conversion and structured logging with tracing.

## When to Use
- Defining error types for library errors
- Converting Windows HRESULT errors to Rust Result
- Adding context to errors throughout the codebase
- Handling errors at application boundaries
- Logging errors with tracing throughout the codebase

## Key Patterns

### Library Errors (thiserror)
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LibraryError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Process spawn error: {0}")]
    ProcessSpawn(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error("Windows API error: {0}")]
    WindowsApi(String),
}
```

### Application Errors (anyhow)
```rust
use anyhow::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let config = load_config()
        .context("Failed to load configuration")?;

    let result = perform_operation(&config)
        .await
        .context("Failed to perform operation")?;

    Ok(())
}
```

### Windows HRESULT Error Handling
```rust
use windows::Win32::Foundation::HRESULT;
use windows::core::Error as WindowsError;

// Convert HRESULT to LibraryError
let result = unsafe { SomeWindowsAPI() };
if !SUCCEEDED(result) {
    let win_error = WindowsError::from_hresult(result)?;
    return Err(LibraryError::WindowsApi(win_error.to_string()));
}
```

### Error Conversion with From
```rust
// Automatic conversion with From trait
impl From<std::io::Error> for LibraryError {
    fn from(err: std::io::Error) -> Self {
        LibraryError::Io(err)
    }
}

// Use ? operator for clean propagation
let file = File::open(path)?; // Automatically converts to LibraryError
```

### Error Context
```rust
// Process spawning
let child = Command::new("executable")
    .spawn()
    .context("Failed to spawn executable - is it in PATH?")?;

// File operations
create_junction(&target, &dest)
    .context("Failed to create junction")?;

// Mount operations
mount_resource(&config)
    .await
    .context("Failed to mount resource")?;
```

## Domain-Specific Error Patterns

### External Process Errors
- Check if executable exists in PATH before spawning
- Handle installation errors gracefully
- Convert HRESULT errors to library errors
- Log external process error codes for debugging

### File System Errors
- Handle file operation failures (permissions, path length)
- Check if resources already exist before creating
- Handle cleanup failures on shutdown
- Track file handles for proper cleanup

### Process Management Errors
- Handle process spawn failures (executable not found)
- Track process exit codes
- Handle process kill failures during cleanup
- Use PID files for process tracking

### Configuration Errors
- Validate configuration fields
- Handle parsing errors with context
- Provide sensible defaults for missing config
- Document configuration schema

### Resource Limit Errors
- Handle out-of-memory scenarios gracefully
- Track actual usage vs configured limits
- Log allocation failures
- Implement graceful degradation

## Common Pitfalls
- Losing Windows HRESULT context when converting errors
- Not adding context to external operation errors
- Swallowing process spawn errors without logging
- Not handling cleanup errors on shutdown
- Overly generic error messages for failures
- Not logging errors at appropriate tracing levels

## Best Practices
- Use thiserror for library errors
- Use anyhow for application-level errors
- Add context with .context() for all external operations
- Log errors with tracing::error! for failures
- Log errors with tracing::warn! for recoverable issues
- Convert HRESULT errors to library errors with Windows API context
- Keep error messages actionable and specific
- Include file/line information in error context when helpful

## Error Logging with Tracing
```rust
use tracing::{error, warn, info, debug};

// Critical errors
error!("Failed to spawn memefs.exe: {:?}", err);

// Recoverable errors
warn!("Junction already exists, reusing: {:?}", path);

// Informational
info!("RAM disk mounted successfully at {}", mount_point);

// Debug for troubleshooting
debug!("Process spawned with PID: {}", pid);

// See: All source files use tracing for structured logging
```

## Related Files
- Library error type definitions in dedicated error module
- Configuration error handling in config module
- Process spawn and external process errors
- File system operation error handling
- Resource mount/unmount errors
- Application-level error handling in main

## Related Skills
- `../skills/code-review-checklist/SKILL.md` - For reviewing error handling patterns
- `../skills/code-fix/SKILL.md` - For refactoring error handling
- `../workflows/debug.md` - For debugging error conditions
- `../skills/windows-api-validation/SKILL.md` - For HRESULT error handling
- `../skills/rust-pro/SKILL.md` - For general Rust error handling patterns
