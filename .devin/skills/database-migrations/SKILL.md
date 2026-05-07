---
name: database-migrations
description: Database migration best practices for Wireframe-AI SQLite schema changes, data migrations, rollbacks, and zero-downtime deployments.
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

# Database Migration Patterns for Wireframe-AI

Safe, reversible database schema changes for Wireframe-AI SQLite production systems.

## When to Activate

- Creating or altering SQLite tables in Wireframe-AI
- Adding/removing columns or indexes
- Running data migrations (backfill, transform)
- Planning zero-downtime schema changes
- Setting up migration tooling for Wireframe-AI
- Modifying the Context module's database schema

## Core Principles

1. **Every change is a migration** — never alter production databases manually
2. **Migrations are forward-only in production** — rollbacks use new forward migrations
3. **Schema and data migrations are separate** — never mix DDL and DML in one migration
4. **Test migrations against production-sized data** — a migration that works on 100 rows may lock on 10M
5. **Migrations are immutable once deployed** — never edit a migration that has run in production

## Migration Safety Checklist

Before applying any migration:

- [ ] Migration has both UP and DOWN (or is explicitly marked irreversible)
- [ ] No full table locks on large tables (use transactions carefully)
- [ ] New columns have defaults or are nullable (never add NOT NULL without default)
- [ ] Indexes created with proper strategy
- [ ] Data backfill is a separate migration from schema change
- [ ] Tested against a copy of production data
- [ ] Rollback plan documented
- [ ] Schema validation passes with `/wireframe-workflow`

## SQLite Patterns

### Adding a Column Safely

```sql
-- GOOD: Nullable column, no lock
ALTER TABLE users ADD COLUMN avatar_url TEXT;

-- GOOD: Column with default (SQLite 3.35.0+ supports ALTER TABLE ADD COLUMN with DEFAULT)
ALTER TABLE users ADD COLUMN is_active INTEGER NOT NULL DEFAULT 1;

-- BAD: NOT NULL without default on existing table (requires full rewrite)
ALTER TABLE users ADD COLUMN role TEXT NOT NULL;
-- This requires copying the table and may lock
```

### Adding an Index

```sql
-- SQLite creates indexes without blocking writes
-- But for large tables, consider doing this in a separate migration
CREATE INDEX idx_users_email ON users (email);

-- For partial indexes (more efficient)
CREATE INDEX idx_users_active ON users (email) WHERE is_active = 1;
```

### Renaming a Column (Zero-Downtime)

SQLite has limited ALTER TABLE support. Use the expand-contract pattern:

```sql
-- Step 1: Add new column (migration 001)
ALTER TABLE users ADD COLUMN display_name TEXT;

-- Step 2: Backfill data (migration 002, data migration)
UPDATE users SET display_name = username WHERE display_name IS NULL;

-- Step 3: Update application code to read/write both columns
-- Deploy application changes

-- Step 4: Stop writing to old column, drop it (migration 003)
-- In SQLite, dropping requires recreating the table
BEGIN TRANSACTION;
CREATE TABLE users_new (
    id INTEGER PRIMARY KEY,
    display_name TEXT NOT NULL,
    -- other columns without username
);
INSERT INTO users_new SELECT id, display_name, ... FROM users;
DROP TABLE users;
ALTER TABLE users_new RENAME TO users;
COMMIT;
```

### Removing a Column Safely

```sql
-- Step 1: Remove all application references to the column
-- Step 2: Deploy application without the column reference
-- Step 3: Drop column in next migration (requires table recreation in SQLite)
BEGIN TRANSACTION;
CREATE TABLE users_new (
    id INTEGER PRIMARY KEY,
    -- columns without the dropped one
);
INSERT INTO users_new SELECT id, ... FROM users;
DROP TABLE users;
ALTER TABLE users_new RENAME TO users;
COMMIT;
```

### Large Data Migrations

```sql
-- BAD: Updates all rows in one transaction (locks table)
UPDATE users SET normalized_email = LOWER(email);

-- GOOD: Batch update with progress
-- SQLite doesn't have SKIP LOCKED, so use LIMIT in a loop
-- This should be done in application code, not pure SQL
```

### Application-Level Batch Migration

```rust
use sqlx::SqlitePool;

async fn backfill_data(pool: &SqlitePool) -> Result<()> {
    let batch_size = 1000;
    let mut offset = 0;

    loop {
        let result = sqlx::query(
            "UPDATE users SET normalized_email = LOWER(email)
             WHERE normalized_email IS NULL
             LIMIT ? OFFSET ?"
        )
        .bind(batch_size)
        .bind(offset)
        .execute(pool)
        .await?;

        if result.rows_affected() == 0 {
            break;
        }

        offset += batch_size;
    }

    Ok(())
}
```

## Migration Tooling for Wireframe-AI

### Using sqlx-cli

```bash
# Install sqlx-cli
cargo install sqlx-cli

# Create a new migration
sqlx migrate add add_user_avatar

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Build offline (requires .sqlx directory)
cargo build --offline
```

### Migration File Structure

```text
migrations/
├── 20240115000000_add_user_avatar.up.sql
├── 20240115000000_add_user_avatar.down.sql
├── 20240115000001_backfill_display_names.up.sql
├── 20240115000001_backfill_display_names.down.sql
└── ...
```

### Example Migration Files

```sql
-- migrations/20240115000000_add_user_avatar.up.sql
ALTER TABLE users ADD COLUMN avatar_url TEXT;
CREATE INDEX IF NOT EXISTS idx_users_avatar ON users (avatar_url);

-- migrations/20240115000000_add_user_avatar.down.sql
DROP INDEX IF EXISTS idx_users_avatar;
-- SQLite doesn't support DROP COLUMN directly, so recreate table
BEGIN TRANSACTION;
CREATE TABLE users_new (
    id INTEGER PRIMARY KEY,
    -- all columns except avatar_url
);
INSERT INTO users_new SELECT id, ... FROM users;
DROP TABLE users;
ALTER TABLE users_new RENAME TO users;
COMMIT;
```

## Wireframe-AI Context Module Integration

### Context Module Schema Management

The Context module owns all persistent state in Wireframe-AI. When modifying its schema:

1. **Check current schema** in `kernel/modules/context/src/storage.rs`
2. **Create migration** using sqlx-cli
3. **Validate schema** with `/wireframe-workflow` skill
4. **Test migration** against production-like data
5. **Update Context module** to handle new schema
6. **Run integration tests** to verify compatibility

### Example: Adding State Metadata

```sql
-- Migration: Add metadata column to states table
-- migrations/20240115000000_add_state_metadata.up.sql
ALTER TABLE states ADD COLUMN metadata TEXT;
CREATE INDEX IF NOT EXISTS idx_states_metadata ON states (metadata);

-- migrations/20240115000000_add_state_metadata.down.sql
DROP INDEX IF EXISTS idx_states_metadata;
BEGIN TRANSACTION;
CREATE TABLE states_new (
    id INTEGER PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    value TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
INSERT INTO states_new SELECT id, key, value, created_at, updated_at FROM states;
DROP TABLE states;
ALTER TABLE states_new RENAME TO states;
COMMIT;
```

### Rust Integration with sqlx

```rust
use sqlx::{SqlitePool, migrate::MigrateDatabase};

async fn setup_database(database_url: &str) -> Result<SqlitePool> {
    // Create database if it doesn't exist
    if !Sqlite::database_exists(database_url).await? {
        Sqlite::create_database(database_url).await?;
    }

    // Connect to database
    let pool = SqlitePool::connect(database_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

#[cfg(test)]
async fn setup_test_pool() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}
```

## Additional Resources

For advanced topics including zero-downtime strategies, testing patterns, and advanced rollback techniques, see `@[skills/database-migrations/SKILL-advanced.md]`.

For best practices, see `@[skills/database-migrations/SKILL-examples.md]`.