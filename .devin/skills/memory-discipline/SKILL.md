---
name: memory-discipline
description: Memory discipline for Wireframe-AI - what to store, what to avoid, and how to use MCP memory effectively
allowed-tools:
  - read
  - grep
  - glob
triggers:
  - model
---

# Memory Discipline

## Purpose

Establish disciplined memory usage for Wireframe-AI development: store durable facts that help future sessions, avoid noise and secrets, and use MCP memory effectively for cross-session knowledge sharing.

## When to Use

**Use this skill when:**
- Starting a new session in the Wireframe-AI repo
- Making decisions about what to store in memory
- Reviewing memory contents for relevance
- Cleaning up obsolete or incorrect memories
- Establishing memory patterns for a new project

**Do NOT use when:**
- Storing secrets or credentials (use vault instead)
- Saving temporary debugging notes (use local files instead)
- Recording one-off conversations (not durable facts)
- Caching file contents (use read tool instead)

## Protocol

### Phase 1: Pre-Work Memory Query

1. **Query memory for repo context**
   - Query for Wireframe-AI repo facts
   - Query for user preferences
   - Query for architectural decisions
   - Query for recurring commands

2. **Load facts into context**
   - Extract durable facts from memory
   - Verify facts are still accurate
   - Use facts to guide initial decisions

3. **Identify knowledge gaps**
   - Note missing information that should be in memory
   - Plan to add new facts after work completes

### Phase 2: During Work

1. **Reference stored facts**
   - Use stored facts when making decisions
   - Don't re-learn what's already in memory
   - Focus on new observations

2. **Identify new durable facts**
   - Note architectural decisions made
   - Record user preferences discovered
   - Document recurring patterns found

### Phase 3: Post-Work Memory Update

1. **Review new facts**
   - Verify facts are durable and reusable
   - Ensure facts are small and explicit
   - Check facts are not secrets or noise

2. **Write back durable facts**
   - Add new entities/relations/observations
   - Update existing facts if changed
   - Delete obsolete facts

3. **Quality check**
   - Run memory quality checklist
   - Ensure memory remains curated and useful

## What to Store

Store durable facts that help future sessions:

| Category | Examples | Why Store |
|----------|----------|-----------|
| Repo-specific facts | "This repo uses Rust + Python with NATS message bus" | Establish tech stack context |
| Repo-specific facts | "Run cargo build --release for production builds" | Document build commands |
| Repo-specific facts | "NATS server must be running before starting modules" | Document dependencies |
| Architecture decisions | "Message envelope: never change root fields, only payload" | Preserve design decisions |
| Architecture decisions | "Context module owns all persistent state" | Document ownership patterns |
| User preferences | "User prefers minimal diffs over rewrites" | Adapt to user style |
| User preferences | "User wants automatic git commits after task completion" | Automate workflows |
| Recurring commands | "Build: cargo build --release" | Quick reference |
| Recurring commands | "Test: cargo test" | Quick reference |
| Test accounts and setup | "Test databases in modules/context/test_*.db" | Document test infrastructure |

**Examples by category:**
- Repo-specific: "Uses Rust + Python with NATS", "Build: cargo build --release", "Topic naming: namespace.noun.verb"
- Architecture: "Envelope: never change root fields", "Context module owns all persistent state"
- User preferences: "Prefers minimal diffs over rewrites", "Wants automatic git commits"
- Commands: "Build: cargo build --release", "Test: cargo test", "Lint: cargo clippy && cargo fmt"
- Test setup: "Test databases in modules/context/test_*.db"

## What NOT to Store

| Category | Examples | Why Avoid |
|----------|----------|-----------|
| Secrets | API keys, passwords, credentials | Security risk |
| Noise | One-off chat conversations | Not durable facts |
| Debug data | Raw logs, temporary notes | Not reusable |
| Speculation | Guesses about root causes | May be incorrect |
| Private data | Customer information | Privacy violation |
| Transient data | Temporary errors, Git hashes | Not durable |
| Cache data | File contents, large outputs | Use tools instead |

Never store: Secrets/API keys/credentials, one-off chat noise, raw logs/debug output, speculative guesses, temporary debugging notes, private customer data, transient errors, Git commit hashes (use git log), file contents (use read tool).

## Memory Usage Pattern

**Before starting work:** Query memory for repo/user, load durable facts into context, use facts to guide decisions.

**During work:** Reference stored facts when making decisions, don't re-learn what's already in memory, focus on new observations.

**At the end:** Write back only durable facts that help future sessions, don't save secrets/temporary notes, keep facts small/explicit, delete obsolete facts.

## Memory Server

Wireframe-AI uses `@modelcontextprotocol/server-memory` for knowledge-graph memory with entities, relations, and observations.

**Configuration:** Already configured in `.devin/config.json`

**Available operations:**
- Query existing memories
- Create new entities/relations/observations
- Update existing memories
- Delete obsolete memories

## MCP Memory Operations

**Query memory for repo context:**

```python
# Query for Wireframe-AI repo facts
mcp_call_tool(
    server_name="omega-memory",
    tool_name="query_graph",
    arguments={
        "query": "MATCH (e:Entity {name: 'Wireframe-AI'}) RETURN e"
    }
)

# Query for user preferences
mcp_call_tool(
    server_name="omega-memory",
    tool_name="query_graph",
    arguments={
        "query": "MATCH (e:Entity {name: 'User'}) RETURN e"
    }
)
```

**Create new memory entities:**

```python
# Create entity for architectural decision
mcp_call_tool(
    server_name="omega-memory",
    tool_name="create_entity",
    arguments={
        "entity": {
            "name": "EnvelopeImmutableFields",
            "entityType": "ArchitecturalDecision",
            "observation": "Message envelope root fields (version, timestamp, source) are immutable"
        }
    }
)

# Create relation
mcp_call_tool(
    server_name="omega-memory",
    tool_name="create_relation",
    arguments={
        "from": "Wireframe-AI",
        "to": "EnvelopeImmutableFields",
        "relationType": "HAS_DECISION"
    }
)
```

**Delete obsolete memories:**

```python
# Delete obsolete fact
mcp_call_tool(
    server_name="omega-memory",
    tool_name="delete_entity",
    arguments={
        "entityName": "OldApproach"
    }
)
```

## Example Memory Facts for Wireframe-AI

**Entities:** Wireframe-AI (repo), Rust/Python (languages), NATS (message bus), Kernel/Context/Orchestrator/Sandbox (modules)

**Relations:** Wireframe-AI uses Rust/Python/NATS, Kernel depends on NATS, Context module owns state, Orchestrator coordinates agents

**Observations:** "Build: cargo build --release", "Test: cargo test", "Schema contracts in schemas/v1/", "Never change envelope root fields", "Topic naming: namespace.noun.verb", "User prefers automatic git commits"

## Memory Quality Checklist

Before writing to memory:
- [ ] Is this a durable fact?
- [ ] Will this help future sessions?
- [ ] Is this small and explicit?
- [ ] Is this not a secret?
- [ ] Is this not temporary noise?
- [ ] Is this not a guess?

Memory becomes useful when it's curated, not when it's a pile of context sludge.

## Integration

**Related skills:**
- **superpowers:using-superpowers** - Establish how to find and use skills (invokes this skill)
- **superpowers:architecture** - Architectural decisions should be stored in memory
- **superpowers:project-commands** - Recurring commands should be stored in memory

**Workflow context:**
- Use at the start of every session to load context
- Use throughout the session to reference facts
- Use at the end of session to update memory
- Use when cleaning up obsolete memories
- Use when establishing memory patterns for new projects
