# Database Migration Patterns - Advanced Topics

Advanced topics including zero-downtime strategies, testing patterns, and advanced rollback techniques for Wireframe-AI database migrations.

## Zero-Downtime Strategies

### Expand-Contract Pattern

For complex schema changes that SQLite's limited ALTER TABLE cannot handle:

1. **Expand**: Add new columns/tables without removing old ones
2. **Migrate**: Backfill data while both old and new structures exist
3. **Contract**: Remove old structures after application is updated

This allows zero-downtime deployments by maintaining compatibility during the transition.

### Blue-Green Deployments

For critical migrations:

1. Deploy new application version to green environment
2. Run migrations on green database
3. Validate green environment
4. Switch traffic to green
5. Keep blue environment for rollback

## Testing Patterns

### Production-Like Testing

Always test migrations against data that mirrors production:

```bash
# Copy production database to staging
sqlite3 production.db ".backup staging.db"

# Run migration on staging
sqlx migrate run --database-url sqlite://staging.db

# Validate results
```

### Performance Testing

For large tables, test migration performance:

```rust
#[cfg(test)]
#[tokio::test]
async fn test_migration_performance() {
    // Create table with 1M rows
    // Run migration
    // Measure time
    // Ensure < threshold
}
```

### Rollback Testing

Test both forward and rollback migrations:

```bash
# Run migration
sqlx migrate run

# Verify data integrity

# Rollback
sqlx migrate revert

# Verify data integrity
```

## Advanced Rollback Techniques

### Forward Rollbacks

Since migrations are forward-only in production, rollbacks are new migrations:

```sql
-- Original migration: Add column
-- 001_add_column.up.sql
ALTER TABLE users ADD COLUMN new_field TEXT;

-- Rollback migration (not DOWN migration):
-- 002_remove_column.up.sql
BEGIN TRANSACTION;
CREATE TABLE users_new (id INTEGER PRIMARY KEY, -- without new_field);
INSERT INTO users_new SELECT id, ... FROM users;
DROP TABLE users;
ALTER TABLE users_new RENAME TO users;
COMMIT;
```

### Data Restoration

For migrations that modify data:

```sql
-- Create backup table before migration
CREATE TABLE users_backup AS SELECT * FROM users;

-- Run migration

-- If rollback needed
DROP TABLE users;
CREATE TABLE users AS SELECT * FROM users_backup;
DROP TABLE users_backup;
```

## Related Resources

See `@[skills/database-migrations/SKILL.md]` for when to activate, core principles, safety checklist, SQLite patterns, tooling, and Wireframe-AI integration.

See `@[skills/database-migrations/SKILL-examples.md]` for best practices.