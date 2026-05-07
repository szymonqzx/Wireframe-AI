# Test-Driven Development - Anti-Patterns and Rules

Testing anti-patterns to avoid and final rules for TDD.

## Testing Anti-Patterns

When adding mocks or test utilities, read @testing-anti-patterns.md to avoid common pitfalls:
- Testing mock behavior instead of real behavior
- Adding test-only methods to production classes
- Mocking without understanding dependencies

## Final Rule

```
Production code → test exists and failed first
Otherwise → not TDD
```

No exceptions without your human partner's permission.

## Related Resources

See `@[skills/test-driven-development/SKILL.md]` for TDD overview, the iron law, red-green-refactor cycle, and why order matters.

See `@[skills/test-driven-development/SKILL-advanced.md]` for common rationalizations, red flags, bug fix examples, verification checklists, and debugging integration.