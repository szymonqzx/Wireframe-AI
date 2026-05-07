# Windows API Development - Pitfalls and Best Practices

Common pitfalls, best practices, focus areas, and related skills for Windows API development.

## Common Pitfalls

- **Unchecked HRESULT** - Forgetting to check Windows API return values
- **Handle leaks** - Not calling CloseHandle or equivalent cleanup functions
- **UTF-16 mishandling** - Incorrect string encoding for Windows APIs expecting wide strings
- **Missing safety comments** - Unsafe blocks without documenting safety invariants
- **Version assumptions** - Using APIs without checking Windows version or documenting requirements
- **Buffer overflows** - Using unsafe C-style string functions instead of safe Rust alternatives
- **Race conditions** - Not handling asynchronous Windows operations properly

## Best Practices

1. **Always check HRESULT** - Never ignore Windows API return values
2. **Use scopeguard for handles** - Ensure cleanup even on early returns
3. **Document unsafe blocks** - Every unsafe block needs a safety comment
4. **Check Windows version** - Handle version-specific APIs gracefully
5. **Prefer safe APIs** - Use Rust-safe alternatives when available
6. **Test on multiple Windows versions** - Verify compatibility across Windows 10/11
7. **Use Windows SDK tools** - Application Verifier, DebugDiag for deep debugging

## Focus Areas for Windows API Development

When working with Windows APIs, prioritize:

1. **Process management** - Spawning, monitoring, terminating processes
2. **File system operations** - Junctions, symlinks, reparse points
3. **Handle management** - Proper cleanup and RAII patterns
4. **Error handling** - HRESULT conversion and context preservation
5. **String encoding** - UTF-16/UTF-8 conversion for Windows APIs
6. **Version compatibility** - Check API availability and document requirements
7. **Safety documentation** - Document invariants for all unsafe blocks

## Related Skills

- `../skills/code-fix/SKILL.md` - For fixing Windows API issues found during validation
- `../workflows/debug.md` - For debugging Windows handle leaks
- `../skills/code-review-checklist/SKILL.md` - For reviewing Windows API usage
- `../skills/error-handling/SKILL.md` - For HRESULT error handling patterns
- `../skills/filesystem-operations/SKILL.md` - For junction/symlink operations
- `../skills/rust-pro/SKILL.md` - For general Rust systems programming

## Related Resources

See `@[skills/windows-api-validation/SKILL.md]` for when to use, key patterns, and validation techniques.

See `@[skills/windows-api-validation/SKILL-advanced.md]` for edge case handling, failure modes, performance considerations, and security notes.