# Feature Enhancement - Examples and Procedures

Success criteria, rollback procedures, and related skills for feature enhancement work.

## Success Criteria

- [ ] All tests pass (unit, integration, e2e)
- [ ] No regression in existing functionality
- [ ] Performance metrics within acceptable range
- [ ] Security review completed
- [ ] Documentation updated
- [ ] Code reviewed and approved
- [ ] Rollback procedure tested
- [ ] Feature flagged if needed for gradual rollout

## Rollback Procedure

```powershell
# If enhancement fails or causes issues:
# 1. Identify last good commit
git log --oneline -10

# 2. Rollback to last good state
git revert <commit-hash>

# 3. If revert fails, hard reset (use with caution)
git reset --hard <commit-hash>

# 4. Rebuild and test - adapt to your project
<build-command>
<test-command>

# 5. Document the rollback
git log --format="%H %s" -1 > rollback-log.txt
```

## Related Skills

- `architecture` - Architectural decision-making framework
- `brainstorming` - Socratic questioning protocol and user communication
- `clean-code` - Pragmatic coding standards
- `error-handling` - Error handling patterns using anyhow and thiserror
- `plan-writing` - Structured task planning with clear breakdowns
- `systematic-debugging` - Systematic debugging methodology

## Related Resources

See `@[skills/enhancement/SKILL.md]` for when to use, subagent usage, pre-flight checks, and the enhancement process.

See `@[skills/enhancement/SKILL-advanced.md]` for edge case handling, failure modes, performance considerations, security notes, and guardrails.