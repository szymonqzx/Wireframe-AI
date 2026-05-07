# Testing Strategy - Anti-Patterns and Metrics

Testing anti-patterns to avoid, metrics to track, and related resources.

## Testing Anti-Patterns

**Avoid:**
- Testing implementation details (test behavior, not code)
- Over-mocking (test real interactions where possible)
- Shared test state (tests should be independent)
- Sleep-based synchronization (use explicit waits)
- Test code duplication (use fixtures and helpers)
- Ignoring failing tests (fix or skip explicitly)

## Testing Metrics

**Track:**
- Test coverage percentage (aim for >80%)
- Test execution time (unit <1s, integration <10s, E2E <5min)
- Flaky test rate (aim for 0%)
- Regression test pass rate (aim for 100%)

## Related Skills

- `run-rust-tests` - Execute Rust test suite
- `run-tests` - Run tests after changes
- `tdd-workflow` - Test-Driven Development methodology
- `quality-checklist` - Quality assurance protocol
- `systematic-debugging` - Debug test failures

## Resources

- Rust Testing Book: https://doc.rust-lang.org/book/ch11-00-testing.html
- pytest Documentation: https://docs.pytest.org/
- Testing Best Practices: https://testing.googleblog.com/

## Related Resources

See `@[skills/testing/SKILL.md]` for testing philosophy, the testing pyramid, and Wireframe-AI testing patterns.

See `@[skills/testing/SKILL-advanced.md]` for running tests, test data management, quality gates, CI/CD integration, and test maintenance.