---
name: powershell-windows
description: PowerShell Windows patterns. Critical pitfalls, operator syntax, error handling.
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

# PowerShell Windows Patterns

> Critical patterns and pitfalls for Windows PowerShell.

## Purpose

Provide essential guidance for writing robust PowerShell scripts on Windows, focusing on critical syntax rules, common pitfalls, and cross-platform compatibility. Ensure scripts avoid common errors that cause runtime failures or unexpected behavior.

## When to Use

Use this skill when:
- Writing PowerShell scripts for Windows automation
- Handling Windows-specific file operations
- Working with JSON data in PowerShell
- Implementing error handling in scripts
- Managing arrays and collections
- Writing cross-platform compatible scripts

## Protocol

### Step 1: Setup Script Structure

1. **Initialize Script**
   ```powershell
   # Strict mode
   Set-StrictMode -Version Latest
   $ErrorActionPreference = "Continue"

   # Paths
   $ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
   ```

2. **Error Handling Framework**
   - Use try/catch/finally for error handling
   - Set ErrorActionPreference appropriately (Stop for dev, Continue for prod)
   - Don't return inside try block
   - Use finally for cleanup

### Step 2: Apply Critical Syntax Rules

1. **Operator Syntax**
   - Each cmdlet call MUST be in parentheses when using logical operators
   - Check: `if ((Test-Path "a") -or (Test-Path "b"))`

2. **Unicode Restriction**
   - Use ASCII characters only in scripts
   - Replace Unicode/emoji with ASCII equivalents
   - Check: Use [OK] [+] instead of ✓

3. **Null Safety**
   - Always check for null before accessing properties
   - Check: `$array -and $array.Count -gt 0`

### Step 3: Handle Data Operations

1. **String Interpolation**
   - Store complex expressions in variables first
   - Avoid nested subexpressions in strings
   - Pattern: `$value = $obj.prop.sub; Write-Output "Value: $value"`

2. **JSON Operations**
   - Always specify `-Depth` for nested objects
   - Check: `ConvertTo-Json -Depth 10`
   - Use `-Raw` for file operations

### Step 4: Handle File Operations

1. **Path Construction**
   - Use Join-Path for cross-platform safety
   - Avoid hardcoded paths
   - Use environment variables for user paths

2. **File Operations**
   - Specify encoding explicitly (UTF8)
   - Use -Raw for reading entire files
   - Handle file not found errors

### Step 5: Test and Validate

1. **Test Common Errors**
   - Verify parentheses in logical operators
   - Check for Unicode characters
   - Test null handling
   - Validate JSON depth parameter

2. **Cross-Platform Testing**
   - Test on Windows and Unix if cross-platform
   - Verify path separators work correctly
   - Check encoding compatibility

---

## 1. Operator Syntax Rules

### CRITICAL: Parentheses Required

| Wrong | Correct |
|----------|-----------|
| `if (Test-Path "a" -or Test-Path "b")` | `if ((Test-Path "a") -or (Test-Path "b"))` |
| `if (Get-Item $x -and $y -eq 5)` | `if ((Get-Item $x) -and ($y -eq 5))` |

**Rule:** Each cmdlet call MUST be in parentheses when using logical operators.

---

## 2. Unicode/Emoji Restriction

### CRITICAL: No Unicode in Scripts

| Purpose | Don't Use | Use |
|---------|-------------|--------|
| Success | ✓ | [OK] [+] |
| Error | ✗ 🔴 | [!] [X] |
| Warning | ⚠️ 🟡 | [*] [WARN] |
| Info | ℹ️ 🔵 | [i] [INFO] |
| Progress | ⏳ | [...] |

**Rule:** Use ASCII characters only in PowerShell scripts.

---

## 3. Null Check Patterns

### Always Check Before Access

| Wrong | Correct |
|----------|-----------|
| `$array.Count -gt 0` | `$array -and $array.Count -gt 0` |
| `$text.Length` | `if ($text) { $text.Length }` |

---

## 4. String Interpolation

### Complex Expressions

| ❌ Wrong | ✅ Correct |
|----------|-----------|
| `"Value: $($obj.prop.sub)"` | Store in variable first |

**Pattern:**
```
$value = $obj.prop.sub
Write-Output "Value: $value"
```

---

## 5. Error Handling

### ErrorActionPreference

| Value | Use |
|-------|-----|
| Stop | Development (fail fast) |
| Continue | Production scripts |
| SilentlyContinue | When errors expected |

### Try/Catch Pattern

- Don't return inside try block
- Use finally for cleanup
- Return after try/catch

---

## 6. File Paths

### Windows Path Rules

| Pattern | Use |
|---------|-----|
| Literal path | `C:\Users\User\file.txt` |
| Variable path | `Join-Path $env:USERPROFILE "file.txt"` |
| Relative | `Join-Path $ScriptDir "data"` |

**Rule:** Use Join-Path for cross-platform safety.

---

## 7. Array Operations

### Correct Patterns

| Operation | Syntax |
|-----------|--------|
| Empty array | `$array = @()` |
| Add item | `$array += $item` |
| ArrayList add | `$list.Add($item) | Out-Null` |

---

## 8. JSON Operations

### CRITICAL: Depth Parameter

| ❌ Wrong | ✅ Correct |
|----------|-----------|
| `ConvertTo-Json` | `ConvertTo-Json -Depth 10` |

**Rule:** Always specify `-Depth` for nested objects.

### File Operations

| Operation | Pattern |
|-----------|---------|
| Read | `Get-Content "file.json" -Raw | ConvertFrom-Json` |
| Write | `$data | ConvertTo-Json -Depth 10 | Out-File "file.json" -Encoding UTF8` |

---

## 9. Common Errors

| Error Message | Cause | Fix |
|---------------|-------|-----|
| "parameter 'or'" | Missing parentheses | Wrap cmdlets in () |
| "Unexpected token" | Unicode character | Use ASCII only |
| "Cannot find property" | Null object | Check null first |
| "Cannot convert" | Type mismatch | Use .ToString() |

---

## 10. Script Template

```powershell
# Strict mode
Set-StrictMode -Version Latest
$ErrorActionPreference = "Continue"

# Paths
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# Main
try {
    # Logic here
    Write-Output "[OK] Done"
    exit 0
}
catch {
    Write-Warning "Error: $_"
    exit 1
}
```

---

> **Remember:** PowerShell has unique syntax rules. Parentheses, ASCII-only, and null checks are non-negotiable.

---

## Common Pitfalls
- Forgetting parentheses around cmdlet calls in logical operators
- Using Unicode/emoji characters in scripts
- Not checking for null before accessing properties
- Not using -Depth parameter with ConvertTo-Json
- Using sync methods in async contexts
- Not handling errors properly
- Hardcoding paths instead of using Join-Path

## Best Practices
- Always wrap cmdlet calls in parentheses when using logical operators
- Use ASCII characters only in scripts
- Check for null before accessing object properties
- Always specify -Depth for nested JSON objects
- Use Join-Path for cross-platform path construction
- Use try/catch/finally for error handling
- Set ErrorActionPreference appropriately (Stop for dev, Continue for prod)

## Integration

This skill integrates with:
- `/filesystem-operations` - For Windows file system operations
- `/windows-api-validation` - For Windows API programming patterns
- `/karpathy-guidelines` - For Think Before Coding principle

