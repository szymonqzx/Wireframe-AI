# Parallel Search - Examples and Production

Detailed examples, metrics, troubleshooting, and production considerations for parallel search.

## Example Workflows

### Example 1: Module Dependency Mapping
- Turn 1: Search for module declarations and imports
- Turn 2: Search for function calls between modules
- Turn 3: Read key module files
- Turn 4: Generate dependency graph

### Example 2: Security Pattern Detection
- Turn 1: Search for security-sensitive functions (exec, eval, etc.)
- Turn 2: Search for SQL query patterns
- Turn 3: Search for user input handling
- Turn 4: Generate security report

### Example 3: Performance Bottleneck Identification
- Turn 1: Search for synchronous I/O operations
- Turn 2: Search for nested loops
- Turn 3: Search for database queries in loops
- Turn 4: Generate performance analysis

## Metrics

- **Search latency:** Target <2 seconds per turn
- **Total time:** Target <10 seconds for 4-turn workflow
- **Files processed:** Typically 50-200 files per search
- **Pattern matches:** Varies by pattern and codebase size

## Troubleshooting

### Too Many Results
- Narrow search scope with path filters
- Use more specific patterns
- Add language-specific filters

### Too Few Results
- Broaden search pattern
- Remove path restrictions
- Check for case sensitivity

### Slow Performance
- Ensure tools are running in parallel
- Check file system performance
- Limit search scope to relevant directories

## Production Considerations

- **Resource usage:** Parallel searches can be CPU-intensive
- **Network:** For distributed codebases, consider caching
- **Rate limiting:** Some systems may limit tool call frequency
- **Error handling:** Handle tool failures gracefully
- **Output size:** Large result sets may need pagination

## Related Resources

See `@[skills/parallel-search/SKILL.md]` for when to use, core principles, tool set, search patterns, and workflow.

See `@[skills/parallel-search/SKILL-advanced.md]` for performance optimization and integration with other skills.