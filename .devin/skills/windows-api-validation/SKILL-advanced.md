# Windows API Development - Advanced Topics

Edge case handling, failure modes, performance considerations, and security notes for Windows API development.

## Edge Case Handling
- **API availability**: Windows version doesn't support API - check availability at runtime
- **Handle invalidation**: Handle becomes invalid after operation - recreate handle if needed
- **String encoding**: UTF-8 to UTF-16 conversion failures - handle encoding errors gracefully
- **Error propagation**: HRESULT errors need conversion to Rust errors - use consistent error types
- **Resource limits**: System resource exhaustion (handles, memory) - handle allocation failures

## Failure Modes
- **Handle leaks**: Not closing handles leads to resource exhaustion - use scopeguard or RAII
- **Buffer overflows**: Unsafe string operations cause memory corruption - use safe Rust alternatives
- **Race conditions**: Asynchronous operations without proper synchronization - use Windows synchronization primitives
- **Version incompatibility**: Using APIs not available on target Windows version - check version requirements
- **Invalid handles**: Using closed or invalid handles - validate handles before use

## Performance Considerations
- Handle reuse: Reuse handles when possible to reduce allocation overhead
- Batch operations: Group multiple API calls to reduce round-trips
- Async I/O: Use overlapped I/O for better performance with file/network operations
- Memory mapping: Use memory-mapped files for large file operations
- Caching: Cache frequently accessed system information

## Security Notes
- **Privilege escalation**: Ensure APIs are called with appropriate privileges
- **Symbolic link attacks**: Validate paths to prevent symlink attacks
- **Handle hijacking**: Protect handle inheritance to prevent unauthorized access
- **DLL injection**: Validate DLL loading paths to prevent malicious DLL loading
- **Token impersonation**: Carefully manage impersonation tokens to prevent privilege escalation

## Related Resources

See `@[skills/windows-api-validation/SKILL.md]` for when to use, key patterns, and validation techniques.

See `@[skills/windows-api-validation/SKILL-examples.md]` for common pitfalls, best practices, focus areas, and related skills.