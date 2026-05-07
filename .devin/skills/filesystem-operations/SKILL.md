---
name: filesystem-operations
description: Windows file system operations including junctions, symlinks, and path handling
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

# Windows File System Operations

## When to Use
- Creating junction points or symbolic links on Windows
- Managing junction/symlink lifecycle (creation, removal, verification)
- Handling Windows-specific path operations
- Working with hash-based or unique subdirectory structures
- Managing PID files for process tracking
- Implementing directory redirection or transparent path mapping

## Key Patterns

### Path Handling with Unique Identifiers
```rust
use std::path::{Path, PathBuf};

// Compute unique identifier for subdirectory (hash, UUID, etc.)
let unique_id = compute_unique_identifier(&current_dir)?;
let target_path = PathBuf::from(base_path)
    .join(&unique_id);

// Example: SHA2 hash, UUID, or project-specific identifier
```

### Junction Point Creation
```rust
// Create junction from source to target directory
// Uses NTFS junction points for transparent directory redirection
let source_dir = Path::new("source");
let target_dir = PathBuf::from(base_path)
    .join(&unique_id);

create_junction(source_dir, &target_dir)?;

// Junctions are directory-specific and work on all Windows versions
```

### Junction Removal
```rust
// Remove junction before unmounting or cleanup
// Critical for cleanup - must happen before resource termination
remove_junction(source_dir)?;

// Always remove junctions before cleaning up the target
```

### Windows-Specific Junction APIs
```rust
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::*;

// Junction creation uses Windows APIs:
// - CreateFileW with FILE_FLAG_BACKUP_SEMANTICS
// - FSCTL_SET_REPARSE_POINT to set junction data
// - FSCTL_DELETE_REPARSE_POINT to remove junction

// See: src/link.rs for Windows API implementation
```

### Unique Identifier Computation
```rust
// Compute unique identifier for directory isolation
// Prevents conflicts between multiple instances or projects
use sha2::{Sha256, Digest};
use hex;

fn compute_unique_identifier(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(path.canonicalize()?.to_string_lossy().as_bytes());
    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}

// Alternative: UUID, timestamp, or project-specific naming scheme
```

### PID File Management
```rust
// Track external process with PID file for cleanup
let pid_file = PathBuf::from(".project-dir")
    .join("process.pid");

std::fs::write(&pid_file, child.id().to_string())?;

// Read PID file for cleanup
let pid = std::fs::read_to_string(&pid_file)?;
let pid: u32 = pid.trim().parse()?;

// Use PID files to track processes that need cleanup on exit
```

## Project-Specific File Operations

### Junction Lifecycle Management
- Create junction before spawning dependent processes
- Junction target: [base_path]/[unique_id] (isolated subdirectory)
- Junction source: [source_directory] (directory to redirect)
- Remove junction before cleanup/unmounting
- Handle both absolute and relative junction targets
- Verify junction creation succeeded before proceeding

### Example Path Structure
```
[base_path]/                 # Base mount point or storage location
└── [unique_id]             # Unique identifier (hash, UUID, etc.)
    ├── source/             # Junction points here
    ├── build/              # Build artifacts
    └── cache/              # Cache or temporary files
```

### Configuration File Handling
```rust
// Load configuration from project root
let config_path = Path::new("config.toml");
let config = if config_path.exists() {
    load_config_from_path(config_path)?
} else {
    // Use sensible defaults
    Config::default()
};

// Adapt configuration file name and format to your project
```

### Cleanup Path Management
```rust
// Cleanup sequence on shutdown:
// 1. Remove junction points or symlinks
// 2. Terminate external processes
// 3. Remove PID files
// 4. Unmount or cleanup storage resources

// Use scopeguard or signal handlers for cleanup coordination
```

## Common Pitfalls
- Forgetting to remove junction before cleanup (causes orphaned junctions)
- Creating junction before target is ready/mounted
- Not handling junction creation failures (permissions, path length)
- Path separator issues when constructing paths
- Not cleaning up PID files on abnormal termination
- Case sensitivity issues with identifier comparison
- Maximum path length limits (260 chars on Windows for junction targets)

## Windows-Specific Considerations

### Junction vs Symbolic Link
- NTFS junction points are often preferred over symbolic links for directory redirection
- Junctions work on all Windows versions without admin privileges
- Junctions are directory-specific (not for files)
- Junctions are transparent to applications (they see the redirected path as a normal directory)

### Path Length Limits
- Windows MAX_PATH is 260 characters by default
- Junction targets must fit within this limit
- Use \\?\ prefix for long paths if needed
- See: src/link.rs for path length handling

### Permission Requirements
- Junction creation requires SE_CREATE_SYMBOLIC_LINK_PRIVILEGE (usually granted)
- No admin privileges required for junction operations
- Junction removal requires write access to parent directory

## Testing File Operations
```rust
#[cfg(test)]
#[cfg(windows)]
mod tests {
    use tempfile::TempDir;

    #[test]
    fn test_junction_creation() {
        let temp_dir = TempDir::new()?;
        let target = temp_dir.path().join("target");
        let ramdisk = temp_dir.path().join("ramdisk");

        std::fs::create_dir(&target)?;
        std::fs::create_dir(&ramdisk)?;

        create_junction(&target, &ramdisk)?;
        assert!(target.exists());

        remove_junction(&target)?;
        assert!(!target.exists());
    }
}

// See: tests/integration_test.rs for integration tests
```

## Related Files
- Adapt these to your project structure:
- Junction/symlink creation and removal module
- PID file management for process tracking
- Storage mount point path management
- Configuration file path handling
- Unique identifier computation and path utilities
- General file operations module

## Related Skills
- `../workflows/debug.md` - For detecting junction cleanup leaks
- `../skills/code-review-checklist/SKILL.md` - For reviewing file operation code
- `../skills/windows-api-validation/SKILL.md` - For validating junction Windows API usage
