# Parallel Search - Performance and Integration

Performance optimization techniques and integration patterns for parallel search.

## Performance Optimization

### Batch Tool Calls

Always batch independent tool calls:

```rust
// GOOD: 8 parallel searches
rg --json "pattern1" --type rust &
rg --json "pattern2" --type rust &
rg --json "pattern3" --type rust &
rg --json "pattern4" --type rust &
rg --json "pattern5" --type rust &
rg --json "pattern6" --type rust &
rg --json "pattern7" --type rust &
rg --json "pattern8" --type rust &
wait

// BAD: Sequential searches
rg --json "pattern1" --type rust
rg --json "pattern2" --type rust
rg --json "pattern3" --type rust
rg --json "pattern4" --type rust
```

### JSON Output Parsing

Use JSON output for programmatic parsing:

```rust
// Parse ripgrep JSON output
let results: Vec<RgResult> = serde_json::from_str(&output)?;

// Extract file paths, line numbers, match context
let files: Vec<String> = results.iter().map(|r| r.path.clone()).collect();
```

### Limit Search Scope

Use language and path filters to limit scope:

```bash
# Search only Rust files in modules directory
rg --json "pattern" --type rust modules/

# Search only Python files in adapter directory
rg --json "pattern" --type python adapter/
```

## Integration with Other Skills

### Orchestration Patterns

Use parallel search as the discovery phase in orchestration:

```rust
// Phase 1: Parallel search (this skill)
// Phase 2: Spawn subagents for deep analysis
// Phase 3: Synthesize findings
```

### Tree-sitter Analysis

Use parallel search to identify files for Tree-sitter analysis:

```rust
// Turn 1: Find Rust files with parallel search
// Turn 2: Run Tree-sitter on identified files
// Turn 3: Analyze AST results
```

### CodeQL Analysis

Use parallel search to identify patterns for CodeQL queries:

```rust
// Turn 1: Find security-sensitive patterns
// Turn 2: Run CodeQL security queries
// Turn 3: Analyze results
```

## Related Resources

See `@[skills/parallel-search/SKILL.md]` for when to use, core principles, tool set, search patterns, and workflow.

See `@[skills/parallel-search/SKILL-examples.md]` for detailed examples, metrics, troubleshooting, and production considerations.