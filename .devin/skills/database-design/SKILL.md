---
name: database-design
description: Database design principles and decision-making. Schema design, indexing strategy, ORM selection, serverless databases.
allowed-tools:
  - read
  - grep
  - glob
  - edit
  - write
triggers:
  - model
---

# Database Design

"Learn to THINK, not copy SQL patterns."

## Purpose

Guide database design decisions through systematic analysis of requirements, context, and trade-offs. Ensure database choices align with deployment environment, data characteristics, and performance needs.

## When to Use

Use this skill when:
- Designing database schemas for new projects
- Selecting database technology (PostgreSQL, Neon, Turso, SQLite, etc.)
- Choosing ORM frameworks (Prisma, Drizzle, SQLAlchemy, TypeORM)
- Planning indexing strategies for performance
- Designing database migrations
- Optimizing query performance and N+1 issues
- Evaluating serverless database options

## Protocol

### Step 1: Context Analysis

1. **Understand Requirements**
   - Data volume and growth projections
   - Query patterns (read-heavy vs write-heavy)
   - Consistency requirements (strong vs eventual)
   - Deployment environment (edge, serverless, traditional)

2. **User Preferences**
   - ASK user about database preferences when unclear
   - Understand team expertise and existing infrastructure
   - Consider compliance and regulatory requirements

### Step 2: Technology Selection

1. **Database Choice**
   - SQLite: Simple apps, embedded, local-first
   - PostgreSQL: Complex apps, strong consistency, traditional deployment
   - Neon/Turso: Serverless, edge deployment, global replication
   - Don't default to PostgreSQL for everything

2. **ORM Selection**
   - Evaluate based on language ecosystem and complexity needs
   - Consider type safety, migration tooling, and query performance
   - Simple queries may not need complex ORM

### Step 3: Schema Design

1. **Structure Planning**
   - Normalize appropriately but avoid over-normalization
   - Define primary keys and relationships clearly
   - Plan for data growth and query patterns

2. **Index Strategy**
   - Plan indexing strategy early in design
   - Focus on frequently queried columns and join conditions
   - Consider composite indexes for multi-column queries

### Step 4: Migration Planning

1. **Safety First**
   - Consider migration safety and rollback strategies
   - Plan for zero-downtime deployments in production
   - Test migrations with realistic data volumes

2. **Performance Validation**
   - Test query performance with realistic data volumes
   - Use EXPLAIN ANALYZE to understand query plans
   - Address N+1 query problems before deployment

## Selective Reading Rule

**Read ONLY files relevant to the request!**

|| File | Description | When to Read |
||------|-------------|--------------|
|| `database-selection.md` | PostgreSQL vs Neon vs Turso vs SQLite | Choosing database |
|| `orm-selection.md` | Drizzle vs Prisma vs Kysely | Choosing ORM |
|| `schema-design.md` | Normalization, PKs, relationships | Designing schema |
|| `indexing.md` | Index types, composite indexes | Performance tuning |
|| `optimization.md` | N+1, EXPLAIN ANALYZE | Query optimization |
|| `migrations.md` | Safe migrations, serverless DBs | Schema changes |

## Decision Checklist

Before designing schema:

- [ ] Asked user about database preference?
- [ ] Chosen database for THIS context?
- [ ] Considered deployment environment?
- [ ] Planned index strategy?
- [ ] Defined relationship types?
- [ ] Evaluated ORM complexity vs needs?
- [ ] Planned migration strategy?

## Anti-Patterns

| Anti-Pattern | Why Bad | Correct Approach |
|-------------|---------|------------------|
| Default to PostgreSQL for simple apps | Overkill, adds operational complexity | Use SQLite for simple, embedded apps |
| Skip indexing on frequently queried columns | Slow queries, poor performance | Plan indexing strategy early |
| Use SELECT * in production queries | Wasted bandwidth, breaks query plans | Select only needed columns |
| Store JSON when structured data is better | No type safety, harder to query | Use structured columns when appropriate |
| Ignore N+1 query problems | Performance degradation as data grows | Address during design, not after deployment |
| Over-normalize schema | Complex joins, poor performance | Normalize appropriately, avoid over-normalization |
| Choose complex ORM when simple queries suffice | Unnecessary complexity, overhead | Use simple queries or lightweight ORM |
| Ignore migration safety | Downtime, data loss | Plan rollback strategies and test migrations |

## Best Practices

| Practice | Benefit | Example |
|----------|---------|---------|
| ASK user about database preferences | Aligns with team expertise and constraints | "Do you prefer PostgreSQL or SQLite for this project?" |
| Choose database based on deployment context | Optimizes for actual environment | Use Neon for edge deployment, SQLite for embedded |
| Plan indexing strategy early | Prevents performance issues | Index frequently queried columns and join conditions |
| Use parameterized queries | Prevents SQL injection | `SELECT * FROM users WHERE id = ?` |
| Consider migration safety | Enables safe deployments | Plan rollback strategies and test with realistic data |
| Normalize appropriately | Balances performance and data integrity | Normalize to 3NF, avoid over-normalization |
| Test query performance with realistic data | Catches performance issues early | Use EXPLAIN ANALYZE with production-like data volumes |
| Address N+1 queries during design | Prevents performance degradation | Use JOINs or batch queries instead of loops |

## Code Examples

**Example: Schema design with indexing**

```sql
-- Good: Properly indexed schema
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Index frequently queried column
CREATE INDEX idx_users_email ON users(email);

-- Composite index for multi-column queries
CREATE INDEX idx_users_email_created ON users(email, created_at);
```

**Example: Parameterized queries**

```rust
// Good: Parameterized query prevents SQL injection
let user = sqlx::query_as::<_, User>(
    "SELECT id, email FROM users WHERE id = ?"
)
.bind(user_id)
.fetch_one(&pool)
.await?;

// Bad: String concatenation (SQL injection risk)
let query = format!("SELECT id, email FROM users WHERE id = {}", user_id);
```

**Example: Addressing N+1 queries**

```rust
// Bad: N+1 query problem
for post in posts {
    let comments = fetch_comments(post.id).await?; // N+1 queries
}

// Good: Single query with JOIN
let posts_with_comments = sqlx::query_as::<_, PostWithComments>(
    "SELECT posts.*, comments.*
     FROM posts
     LEFT JOIN comments ON comments.post_id = posts.id"
)
.fetch_all(&pool)
.await?;
```

**Example: Migration with rollback**

```sql
-- Migration: Add user preferences column
-- Up migration
ALTER TABLE users ADD COLUMN preferences JSONB DEFAULT '{}';

-- Down migration (rollback)
ALTER TABLE users DROP COLUMN preferences;
```

## Integration

This skill integrates with:
- `/architecture` - For system-level architectural decisions
- `/database-migrations` - For detailed migration planning
- `/karpathy-guidelines` - For Think Before Coding principle
