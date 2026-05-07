# Feature Enhancement - Advanced Topics

Advanced topics including edge case handling, failure modes, performance considerations, security notes, and guardrails.

## Edge Case Handling

### Breaking Changes
**Scenario:** Changes affect existing functionality
**Procedure:**
1. Identify all affected code paths
2. Document breaking changes clearly
3. Provide migration guide if needed
4. Get explicit user approval
5. Implement feature flags if gradual rollout needed
6. Add deprecation warnings for old behavior

### Conflicting Patterns
**Scenario:** New code conflicts with existing architecture
**Procedure:**
1. Document the conflict
2. Highlight trade-offs of both approaches
3. Propose architectural refactoring if justified
4. Get user decision on direction
5. Document decision rationale in ARCHITECTURE.md

### Dependency Conflicts
**Scenario:** New dependencies break existing code
**Procedure:**
1. Validate dependencies in isolation (new project)
2. Check version compatibility matrix
3. Look for alternative libraries if conflicts exist
4. Consider dependency vendoring if critical
5. Update all dependent code if version bump required

### Performance Regression
**Scenario:** Changes slow down existing features
**Procedure:**
1. Establish baseline metrics before changes
2. Profile critical paths after implementation
3. Identify regression source
4. Optimize or rollback if significant degradation
5. Add performance regression tests to CI

### Migration Complexity
**Scenario:** Large changes require data migration
**Procedure:**
1. Design migration strategy (big bang vs incremental)
2. Create migration scripts with rollback capability
3. Test migration on staging data
4. Plan maintenance window if needed
5. Document migration procedure
6. Verify data integrity post-migration

## Failure Modes

### Incomplete Understanding
**Symptom:** Missing context leads to wrong changes
**Prevention:**
- Read existing code thoroughly before editing
- Review documentation and comments
- Understand data flow and dependencies
- Ask clarifying questions if unclear
- Review git history for recent changes

### Breaking Existing Features
**Symptom:** Changes break working code
**Prevention:**
- Run full test suite before changes
- Run test suite after each major change
- Add regression tests for affected areas
- Use feature flags for risky changes
- Test in isolation before integration

### Dependency Hell
**Symptom:** New dependencies cause version conflicts
**Prevention:**
- Validate dependencies in isolation first
- Check transitive dependencies
- Use dependency lock files
- Prefer well-maintained libraries
- Document dependency versions in requirements

### Poor Integration
**Symptom:** New code doesn't fit existing patterns
**Prevention:**
- Review architecture before coding
- Follow existing code style and patterns
- Consult with team on architectural decisions
- Refactor existing code if patterns are outdated
- Document architectural decisions

### Rollback Difficulty
**Symptom:** Changes hard to undo
**Prevention:**
- Use git for version control
- Commit changes incrementally
- Write atomic commits with clear messages
- Test rollback procedure before deployment
- Keep database migrations reversible
- Maintain feature flags for quick disabling

## Performance Considerations

### Build Impact
- Minimize build time increases from new dependencies
- Prefer tree-shakeable libraries for frontend
- Use conditional compilation for optional features
- Monitor build times in CI

### Runtime Performance
- Benchmark critical paths before and after changes
- Profile hot paths with tools (perf, flamegraphs)
- Consider algorithmic complexity of new code
- Add performance regression tests
- Monitor production metrics post-deployment

### Bundle Size (Web)
- Monitor bundle growth for web applications
- Use code splitting for large features
- Lazy load non-critical components
- Analyze bundle with webpack-bundle-analyzer
- Set bundle size budgets in CI

### Database Queries
- Check for N+1 queries when adding features
- Add appropriate indexes for new queries
- Use query optimization tools (EXPLAIN ANALYZE)
- Implement query result caching where appropriate
- Monitor query performance in production

### Memory Usage
- Verify changes don't introduce memory leaks
- Profile memory usage for long-running processes
- Check for unintended object retention
- Use weak references for caches
- Monitor memory metrics in production

## Security Notes

### Input Validation
- Add validation for any new user inputs
- Use whitelist validation over blacklist
- Sanitize data before storage or display
- Validate data types and ranges
- Handle malformed input gracefully

### Authentication
- Ensure new endpoints respect existing auth rules
- Use strong authentication mechanisms (JWT, OAuth)
- Implement proper session management
- Handle token expiration and refresh
- Secure authentication endpoints

### Authorization
- Verify permission checks for new features
- Implement principle of least privilege
- Use role-based access control (RBAC)
- Check authorization on both client and server
- Audit authorization failures

### Data Sanitization
- Sanitize user data before storage or display
- Use parameterized queries to prevent SQL injection
- Escape output to prevent XSS
- Validate file uploads (type, size, content)
- Handle sensitive data encryption

### Dependency Security
- Audit new dependencies for vulnerabilities
- Use tools like `cargo-audit`, `npm audit`
- Keep dependencies updated
- Review dependency maintenance status
- Prefer libraries with security track record

### Secrets Management
- Never hardcode API keys or secrets
- Use environment variables for configuration
- Implement secrets rotation
- Use vault services for production secrets
- Audit secret access logs

## Guardrails

1. **Understand current state** - Read existing code, architecture, and tests before making changes
2. **User approval** - Get explicit approval for major or breaking changes
3. **Conflict warnings** - Warn on conflicting architectural decisions
4. **Incremental commits** - Commit changes incrementally with meaningful messages
5. **Test before deploy** - Test changes before updating preview or deploying
6. **Rollback planning** - Ensure rollback procedure is tested before deployment
7. **Documentation** - Update documentation alongside code changes
8. **Performance monitoring** - Establish baselines and monitor post-deployment
9. **Security review** - Review security implications of all changes
10. **Killswitch** - Create `~/.enhance-stop` to stop enhancement loop

## Related Resources

See `@[skills/enhancement/SKILL.md]` for when to use, subagent usage, pre-flight checks, and the enhancement process.

See `@[skills/enhancement/SKILL-examples.md]` for success criteria, rollback procedures, and related skills.