# Database Migration Patterns - Best Practices

Best practices for database migrations in Wireframe-AI.

## Best Practices

**DO:**
- Use sqlx-cli for migration management
- Separate schema and data migrations
- Test migrations against production-like data
- Use transactions for multi-step operations
- Validate schema changes with `/wireframe-workflow`
- Document irreversible migrations
- Back up production databases before major changes

**DON'T:**
- Manually modify production databases
- Mix DDL and DML in single migrations
- Add NOT NULL columns without defaults
- Skip testing on large datasets
- Edit deployed migration files
- Forget to update application code after schema changes

**Remember**: In SQLite, ALTER TABLE is limited. For complex changes, use the expand-contract pattern with table recreation.

## Related Resources

See `@[skills/database-migrations/SKILL.md]` for when to activate, core principles, safety checklist, SQLite patterns, tooling, and Wireframe-AI integration.

See `@[skills/database-migrations/SKILL-advanced.md]` for zero-downtime strategies, testing patterns, and advanced rollback techniques.