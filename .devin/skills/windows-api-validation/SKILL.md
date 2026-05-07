---
name: windows-api-validation
description: Windows API programming patterns, HRESULT error handling, and validation in Rust
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

# Windows API Development

Comprehensive patterns for Windows API programming in Rust using the windows crate, including HRESULT error handling, handle management, and validation techniques.

## When to Use
- Adding new Windows API calls to the codebase
- Modifying Win32 API usage or error handling
- Investigating Windows-specific bugs
- Reviewing code for Windows API best practices
- Supporting new Windows versions
- Auditing unsafe blocks and handle management
- Creating junction points or reparse points
- Managing process handles and lifecycle

## Key Patterns

### HRESULT Error Handling

```rust
use windows::Win32::Foundation::HRESULT;

let result = unsafe { SomeWindowsAPI() };
if !SUCCEEDED(result) {
    return Err(Error::from_hresult(result));
}
```

Always check HRESULT return values from Windows APIs. Use `SUCCEEDED()` and `FAILED()` macros or `.ok()` for conversion.

### Handle Management with scopeguard

```rust
use windows::Win32::Foundation::HANDLE;
use scopeguard::defer;

let handle = unsafe { CreateHandle() };
defer! {
    unsafe { CloseHandle(handle); }
}
```

Use scopeguard or similar patterns to ensure handles are closed even on early returns or panics.

### Junction Creation Windows APIs

```rust
use windows::Win32::Storage::FileSystem::*;

// Open directory handle for junction creation
let handle = unsafe {
    CreateFileW(
        target_path,
        GENERIC_READ | GENERIC_WRITE,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        None,
        OPEN_EXISTING,
        FILE_FLAG_BACKUP_SEMANTICS,
        HANDLE::default(),
    )
}?;

// Set reparse point data for junction
unsafe {
    DeviceIoControl(
        handle,
        FSCTL_SET_REPARSE_POINT,
        reparse_data,
        // ... additional parameters
    )
}?;
```

### Process Management Windows APIs

```rust
use windows::Win32::System::Threading::*;

// Open process handle for termination
let process_handle = unsafe {
    OpenProcess(PROCESS_TERMINATE, false, pid)
}?;

// Terminate process
unsafe {
    TerminateProcess(process_handle, 1);
}

// Close handle with scopeguard
use scopeguard::defer;
defer! {
    unsafe { let _ = CloseHandle(process_handle); }
}
```

### Safe Wrappers for Windows APIs

```rust
/// # Safety
///
/// Caller must ensure:
/// - The pointer is valid and aligned
/// - The memory is properly initialized
/// - No other thread is accessing this memory concurrently
unsafe fn windows_api_wrapper() -> Result<()> {
    let result = unsafe { DangerousWindowsAPI() };
    if !SUCCEEDED(result) {
        return Err(Error::from_hresult(result));
    }
    Ok(())
}
```

### Handle Management with RAII

```rust
use windows::Win32::Foundation::HANDLE;

// Implement Drop for custom handle types
struct ScopedHandle(HANDLE);

impl Drop for ScopedHandle {
    fn drop(&mut self) {
        unsafe { let _ = CloseHandle(self.0); }
    }
}
```

### UTF-16 String Conversion

```rust
use std::os::windows::ffi::OsStrExt;

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
```

Windows APIs often use UTF-16 (W suffix). Use proper encoding/decoding with `OsString`, `OsStr`, or manual UTF-16 conversion.

### Unsafe Block Documentation

```rust
/// # Safety
///
/// Caller must ensure:
/// - The pointer is valid and aligned
/// - The memory is properly initialized
/// - No other thread is accessing this memory concurrently
unsafe fn dangerous_function(ptr: *mut u8) {
    // ...
}
```

All unsafe blocks must have safety comments documenting the invariants that must be upheld.

### Windows Version Compatibility

```rust
// Check for API availability at runtime
use windows::Win32::Foundation::GetProcAddress;

let module = LoadLibraryW("kernel32.dll")?;
let func = GetProcAddress(module, w!("ReOpenFile"))?;
if func.is_null() {
    // API not available on this Windows version
    return Err(Error::UnsupportedWindowsVersion);
}
```

Document minimum Windows version requirements for version-specific APIs and handle unavailability gracefully.

## Validation Techniques

### 1. HRESULT Checking

Check all Windows API calls that return HRESULT:

```powershell
# Find unchecked HRESULT calls
rg "HRESULT" src/ --type rust -A 2 | rg -v "SUCCEEDED|FAILED|\.ok\(\)|unwrap\(\)|expect\("
```

### 2. Handle Leak Detection

Identify potentially unclosed Windows handles:

```powershell
# Find handle declarations without cleanup
rg ": HANDLE|: HWND|: HINSTANCE" src/ --type rust
# Verify each has corresponding CloseHandle/DestroyWindow/etc.
```

### 3. Version-Specific API Documentation

Check for APIs requiring specific Windows versions:

- `ReOpenFile` - Windows Vista+
- `GetFinalPathNameByHandleW` - Windows Vista+
- `CreateSymbolicLinkW` - Windows Vista+
- `SetFileInformationByHandle` - Windows Vista+

Document version requirements in code comments or documentation.

### 4. UTF-16 String Handling

Verify proper encoding for Windows APIs:

```powershell
# Find UTF-16 API calls
rg "W\(" src/ --type rust
# Check for proper encode_wide/encode_utf16 usage
```

### 5. Security API Checks

Avoid dangerous API patterns:

- `strcpy`, `strcat`, `sprintf` - Buffer overflows
- `CopyMemory`, `RtlCopyMemory` - No size checking
- `GetVersion()` - Deprecated
- `CreateFile.*GENERIC_ALL` - Overly permissive

### 6. Unsafe Block Validation

Ensure all unsafe blocks have safety comments:

```powershell
# Find undocumented unsafe blocks
rg "unsafe" src/ --type rust -B 1 -A 5
# Verify each has "# Safety:" or similar comment
```

## Additional Resources

For edge case handling, failure modes, performance considerations, and security notes, see `@[skills/windows-api-validation/SKILL-advanced.md]`.

For common pitfalls, best practices, focus areas, and related skills, see `@[skills/windows-api-validation/SKILL-examples.md]`.