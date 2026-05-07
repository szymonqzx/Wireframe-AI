# Wireframe-AI Technical Patterns

This document provides Wireframe-AI specific technical patterns and implementation guidelines. For agent orchestration and behavioral standards, see [AGENTS.md](/AGENTS.md).

## Project Overview

Wireframe-AI is a modular, event-driven agentic system built with Rust. It uses NATS for inter-module communication, SQLite for persistence (via Context module), and a unified Provider system for LLM integrations. The architecture emphasizes loose coupling, state ownership, and scalable message processing.

**Tech Stack:**
- Core: Rust 1.75+, Tokio async runtime
- Messaging: NATS with JetStream for durability
- Database: SQLite (via Context module)
- Providers: Unified trait for OpenAI, Anthropic, etc.
- SDK: Python adapter for external integrations

## Critical Rules

### 0. Behavioral Standards

Wireframe-AI follows the Karpathy Guidelines for AI coding. These four principles directly address common AI agent mistakes: wrong assumptions, overcomplication, orthogonal edits, and vague execution.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks (simple typo fixes, obvious one-liners), use judgment — not every change needs the full rigor.

**Integration Point:** Always invoke `/karpathy-guidelines` before starting implementation work, alongside `/wireframe-workflow` and `/project-routing`.

#### The Four Principles

**1. Think Before Coding** - Don't assume. Don't hide confusion. Surface tradeoffs.

Before implementing:
- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them — don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

**2. Simplicity First** - Minimum code that solves the problem. Nothing speculative.

- No features beyond what was asked
- No abstractions for single-use code
- No "flexibility" or "configurability" that wasn't requested
- No error handling for impossible scenarios
- If you write 200 lines and it could be 50, rewrite it

**3. Surgical Changes** - Touch only what you must. Clean up only your own mess.

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting
- Don't refactor things that aren't broken
- Match existing style, even if you'd do it differently
- If you notice unrelated dead code, mention it — don't delete it

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused
- Don't remove pre-existing dead code unless asked

**4. Goal-Driven Execution** - Define success criteria. Loop until verified.

Transform tasks into verifiable goals:

| Instead of... | Transform to... |
|--------------|-----------------|
| "Add validation" | "Write tests for invalid inputs, then make them pass" |
| "Fix the bug" | "Write a test that reproduces it, then make it pass" |
| "Refactor X" | "Ensure tests pass before and after" |

For multi-step tasks, state a brief plan with verification checks.

#### Wireframe-AI Specific Applications

**When Adding a New Module:**
- Think Before Coding: Ask: module purpose, messages to publish/consume? Present options: new crate vs existing? Check: similar module exists?
- Simplicity First: Start minimal (message handler only). Don't add config/logging/metrics unless asked. Use existing patterns.
- Surgical Changes: Only add registration in kernel/interface/src/main.rs. Don't refactor other modules. Match topic naming convention.
- Goal-Driven Execution: 1) Create skeleton → verify: cargo build, 2) Register → verify: sys.module.online published, 3) Implement handler → verify: message processed, 4) Add tests → verify: cargo test passes

**When Modifying Schemas:**
- Think Before Coding: Ask: what fields change? Breaking change? Present tradeoffs: version vs backward compatibility? Check: which modules consume this?
- Simplicity First: Add new fields vs modifying existing. Don't add optional fields unless needed. Use existing envelope structure.
- Surgical Changes: Only modify specific schema file. Don't change unrelated schemas or refactor directory structure.
- Goal-Driven Execution: 1) Update schema → verify: cargo build, 2) Update consumers → verify: no compilation errors, 3) Add migration if needed → verify: migration runs, 4) Test message flow → verify: serialize/deserialize correctly

**When Debugging NATS Issues:**
- Think Before Coding: Ask: symptoms? Which module publishes/subscribes? Present hypotheses: subscription issue? Message format? Timing? Check: NATS server logs, module logs.
- Simplicity First: Start with basic connectivity check. Don't add retry/backpressure/monitoring unless needed. Use existing debugging patterns.
- Surgical Changes: Only fix specific subscription/publishing code. Don't refactor entire NATS setup. Match existing error handling.
- Goal-Driven Execution: 1) Reproduce → verify: consistent, 2) Identify root cause → verify: hypothesis confirmed, 3) Implement fix → verify: resolved, 4) Add regression test → verify: passes

#### Common Pitfalls

| Pitfall | Wrong | Right |
|---------|-------|-------|
| Assuming Without Asking | "Optimize database" → add caching/indexes without asking | "What's slow? Read/write? Which queries? How much data?" |
| Overengineering Simple Requests | "Add config file" → create validation/defaults/hot reload system | Add simple TOML file reading with basic error handling |
| Drive-by Refactoring | "Fix null pointer" → also improve messages, add logging, reformat | Only fix the null pointer issue |
| Vague Success Criteria | "Improve performance" → "I'll optimize the code" | "Target: reduce API response time from 500ms to <100ms. Will add caching, verify with benchmarks" |

#### When to Relax These Guidelines

For trivial tasks, use judgment:
- Simple typo fixes
- Obvious one-liners
- Adding a single import
- Fixing a clear syntax error

The goal is reducing costly mistakes on non-trivial work, not slowing down simple tasks.

### 0.1. Behavioral Modes

Adaptive AI operating modes that optimize performance for specific tasks. Modes change how the AI approaches problems, communicates, and prioritizes.

**Integration Point:** Use `/behavioral-modes` when adapting AI behavior based on task type (brainstorm, implement, debug, review, teach, ship).

#### Available Modes

**BRAINSTORM Mode** - Early project planning, feature ideation, architecture decisions
- Ask clarifying questions before assumptions
- Offer multiple alternatives (at least 3)
- Think divergently - explore unconventional solutions
- No code yet - focus on ideas and options
- Use visual diagrams to explain concepts

**IMPLEMENT Mode** - Writing code, building features, executing plans
- CRITICAL: Use `clean-code` skill standards - concise, direct, no verbose explanations
- Fast execution - minimize questions
- Use established patterns and best practices
- Write complete, production-ready code
- Include error handling and edge cases
- NO tutorial-style explanations - just code
- NO unnecessary comments - let code self-document
- NO over-engineering - solve the problem directly
- NO RUSHING - Quality > Speed. Read ALL references before coding

**DEBUG Mode** - Fixing bugs, troubleshooting errors, investigating issues
- Ask for error messages and reproduction steps
- Think systematically - check logs, trace data flow
- Form hypothesis → test → verify
- Explain the root cause, not just the fix
- Prevent future occurrences

**REVIEW Mode** - Code review, architecture review, security audit
- Be thorough but constructive
- Categorize by severity (Critical/High/Medium/Low)
- Explain the "why" behind suggestions
- Offer improved code examples
- Acknowledge what's done well

**TEACH Mode** - Explaining concepts, documentation, onboarding
- Explain from fundamentals
- Use analogies and examples
- Progress from simple to complex
- Include practical exercises
- Check understanding

**SHIP Mode** - Production deployment, final polish, release preparation
- Focus on stability over features
- Check for missing error handling
- Verify environment configs
- Run all tests
- Create deployment checklist

#### Mode Detection

Automatically detect appropriate mode based on request keywords:

| Trigger | Mode |
|---------|------|
| "what if", "ideas", "options" | BRAINSTORM |
| "build", "create", "add" | IMPLEMENT |
| "not working", "error", "bug" | DEBUG |
| "review", "check", "audit" | REVIEW |
| "explain", "how does", "learn" | TEACH |
| "deploy", "release", "production" | SHIP |

#### Best Practices

- Automatically detect appropriate mode from user request keywords
- Announce mode changes clearly when switching
- Respect mode-specific output styles (concise for IMPLEMENT, explanatory for TEACH)
- Use mode detection matrix as a guide, not a rigid rule
- Allow manual mode overrides when user explicitly requests
- Combine modes when appropriate (e.g., BRAINSTORM → IMPLEMENT)

### 1. Code Organization

- Many small files over few large files (200-400 lines typical, 800 max)
- High cohesion, low coupling between modules
- Organize by domain/module, not by type
- Each module is a separate crate in `modules/`
- State ownership: Context module owns all persistent state

### 2. Code Style

- No emojis in code, comments, or documentation
- Immutability preferred - use `let` by default, `let mut` only when needed
- Never `unwrap()` in production code - use `?` or handle errors
- Proper error handling with `anyhow::Context` for applications, `thiserror` for libraries
- Run `cargo fmt` and `cargo clippy -- -D warnings` before committing

### 3. NATS Messaging

- Topic naming: `namespace.noun.verb` or `namespace.noun`, lowercase, dot-separated
- Message envelopes: Never change root fields (`version`, `timestamp`, `source`)
- Module lifecycle: Publish `sys.module.online` on startup, `sys.module.offline` on shutdown
- Use correlation IDs for request/response patterns
- Implement backpressure with bounded channels

### 4. Schema Management

- Schemas defined in `schemas/` directory
- Never change root fields in message envelopes without versioning
- Provide migration paths for breaking changes
- Validate schemas before deployment
- Maintain backward compatibility for consumers

### 5. Security

- No hardcoded secrets - use vault system for provider credentials
- Provider credentials in `vault/`, never in source code
- Validate all message payloads and user inputs
- Parameterized queries only (sqlx)
- Every `unsafe` block must have `// SAFETY:` comment
- NATS connections must use authentication in production

### 6. Testing

- TDD preferred: Write tests before implementation
- 80% minimum coverage target
- Unit tests in `#[cfg(test)]` modules
- Integration tests in `tests/` directory
- Mock NATS and database for unit tests

### 7. Memory Discipline

Establish disciplined memory usage for Wireframe-AI development: store durable facts that help future sessions, avoid noise and secrets, and use MCP memory effectively for cross-session knowledge sharing.

**Integration Point:** Use `/memory-discipline` at the start of every session to load context, throughout the session to reference facts, and at the end of session to update memory.

#### What to Store

Store durable facts that help future sessions:

| Category | Examples | Why Store |
|----------|----------|-----------|
| Repo-specific facts | "Uses Rust + Python with NATS", "Build: cargo build --release" | Establish tech stack context, document commands |
| Architecture decisions | "Envelope: never change root fields", "Context module owns all persistent state" | Preserve design decisions, document ownership |
| User preferences | "Prefers minimal diffs over rewrites", "Wants automatic git commits" | Adapt to user style, automate workflows |
| Recurring commands | "Build: cargo build --release", "Test: cargo test" | Quick reference |
| Test accounts and setup | "Test databases in modules/context/test_*.db" | Document test infrastructure |

#### What NOT to Store

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

#### Memory Usage Pattern

**Before starting work:** Query memory for repo/user, load durable facts into context, use facts to guide decisions.

**During work:** Reference stored facts when making decisions, don't re-learn what's already in memory, focus on new observations.

**At the end:** Write back only durable facts that help future sessions, don't save secrets/temporary notes, keep facts small/explicit, delete obsolete facts.

#### Memory Quality Checklist

Before writing to memory:
- [ ] Is this a durable fact?
- [ ] Will this help future sessions?
- [ ] Is this small and explicit?
- [ ] Is this not a secret?
- [ ] Is this not temporary noise?
- [ ] Is this not a guess?

Memory becomes useful when it's curated, not when it's a pile of context sludge.

## File Structure

```
Wireframe-AI/
├── modules/              # Rust modules (domain-organized)
│   ├── context/         # State ownership module
│   ├── orchestrator/    # Task orchestration module
│   └── sandbox/         # Code execution module
├── schemas/             # Message envelope schemas
├── kernel/              # Module orchestration and lifecycle
├── sdk/                 # Python SDK for external integrations
├── adapter/             # Python adapters
├── provider-core/       # Provider trait and capability system
├── providers/           # Provider implementations
├── config/              # Configuration files
├── docs/                # Documentation
└── tools/               # Development tools
```

## Key Patterns

### NATS Message Envelope

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope<T> {
    pub version: String,           // Immutable - never change
    pub timestamp: i64,            // Immutable - never change
    pub source: String,            // Immutable - never change
    pub correlation_id: String,    // Immutable - never change
    pub payload: T,                // Mutable - only modify this
}
```

### Module Lifecycle

```rust
pub async fn start_module(&self) -> anyhow::Result<()> {
    self.nats.publish("sys.module.online", &self.module_info).await?;
    // Initialize subscriptions
    // Start workers
    Ok(())
}

pub async fn stop_module(&self) -> anyhow::Result<()> {
    self.nats.publish("sys.module.offline", &self.module_info).await?;
    // Stop workers
    // Cleanup resources
    Ok(())
}
```

### Error Handling

```rust
// Application code with anyhow
use anyhow::Context;

fn load_config(path: &str) -> anyhow::Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {path}"))?;
    toml::from_str(&content)
        .with_context(|| format!("failed to parse {path}"))
}

// Library code with thiserror
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid config format: {0}")]
    Parse(String),
}
```

### Provider Integration

```rust
// Check capabilities before use
async fn generate_response(provider: &dyn Provider, request: ChatRequest) -> anyhow::Result<ChatResponse> {
    let caps = provider.capabilities();
    if request.requires_streaming && !caps.supports_streaming {
        return Err(anyhow!("Provider does not support streaming"));
    }
    provider.chat_completion(request).await
}

// Load credentials from vault
let credentials = vault.get_provider_credentials("openai")
    .context("Failed to load OpenAI credentials")?;
let provider = OpenAIProvider::new(credentials.api_key)?;
```

### Database Access (via Context)

```rust
// Only Context module should access database directly
// Other modules use Context API

impl Context {
    pub async fn get_state<T>(&self, key: &str) -> anyhow::Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let row = sqlx::query_as::<_, (String, Vec<u8>)>(
            "SELECT key, value FROM state WHERE key = $1"
        )
        .bind(key)
        .fetch_optional(&self.db)
        .await?;

        match row {
            Some((_, value)) => Ok(Some(serde_json::from_slice(&value)?)),
            None => Ok(None),
        }
    }
}
```

### Windows File System Operations

**Integration Point:** Use `/filesystem-operations` when creating junction points or symbolic links on Windows, managing junction/symlink lifecycle, or handling Windows-specific path operations.

#### Junction Point Creation

```rust
// Create junction from source to target directory
// Uses NTFS junction points for transparent directory redirection
let source_dir = Path::new("source");
let target_dir = PathBuf::from(base_path).join(&unique_id);

create_junction(source_dir, &target_dir)?;

// Junctions are directory-specific and work on all Windows versions
```

#### Junction Removal

```rust
// Remove junction before unmounting or cleanup
// Critical for cleanup - must happen before resource termination
remove_junction(source_dir)?;

// Always remove junctions before cleaning up the target
```

#### Unique Identifier Computation

```rust
// Compute unique identifier for directory isolation
// Prevents conflicts between multiple instances or projects
use sha2::{Sha256, Digest};
use hex;

fn compute_unique_identifier(path: &Path) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(path.canonicalize()?.to_string_lossy().as_bytes());
    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}
```

#### Junction Lifecycle Management

- Create junction before spawning dependent processes
- Junction target: [base_path]/[unique_id] (isolated subdirectory)
- Junction source: [source_directory] (directory to redirect)
- Remove junction before cleanup/unmounting
- Handle both absolute and relative junction targets
- Verify junction creation succeeded before proceeding

#### Windows-Specific Considerations

**Junction vs Symbolic Link:**
- NTFS junction points are often preferred over symbolic links for directory redirection
- Junctions work on all Windows versions without admin privileges
- Junctions are directory-specific (not for files)
- Junctions are transparent to applications (they see the redirected path as a normal directory)

**Path Length Limits:**
- Windows MAX_PATH is 260 characters by default
- Junction targets must fit within this limit
- Use \\?\ prefix for long paths if needed

**Permission Requirements:**
- Junction creation requires SE_CREATE_SYMBOLIC_LINK_PRIVILEGE (usually granted)
- No admin privileges required for junction operations
- Junction removal requires write access to parent directory

#### Common Pitfalls

- Forgetting to remove junction before cleanup (causes orphaned junctions)
- Creating junction before target is ready/mounted
- Not handling junction creation failures (permissions, path length)
- Path separator issues when constructing paths
- Not cleaning up PID files on abnormal termination
- Case sensitivity issues with identifier comparison
- Maximum path length limits (260 chars on Windows for junction targets)

## Environment Variables

```bash
# Required
NATS_URL=nats://localhost:4222
DATABASE_URL=sqlite:wireframe_ai_context.db

# Optional
RUST_LOG=debug
NATS_AUTH_USER=
NATS_AUTH_PASSWORD=
```

## Available Commands

### Skills (invoke with `/skill-name` or let agent choose)

**Core Workflows:**
- `/karpathy-guidelines` - Andrej Karpathy-inspired behavioral standards for AI coding
- `/wireframe-workflow` - Wireframe-AI development workflow
- `/quality-checklist` - Quality checks and anti-slop protocol
- `/orchestration-patterns` - Swarm orchestration patterns
- `/project-routing` - Agent and skill selection
- `/final-checks` - Final verification before completion

**Development:**
- `/implementation` - Systematic feature implementation
- `/architecture` - Architectural decision-making
- `/systematic-debugging` - 4-phase debugging methodology
- `/enhancement` - Add or update features

**Code Quality:**
- `/rust-pro` - Rust patterns and best practices
- `/check-rust-quality` - Run clippy and fmt
- `/code-fix` - Systematic code review and fixing
- `/code-review-checklist` - Code review guidelines and best practices

**Testing & Building:**
- `/run-rust-tests` - Run Rust test suite
- `/build-release` - Build in release mode

### Agents (automatically selected via `/intelligent-routing` or manual invocation)

**Intelligent Agent Routing:** The AI automatically analyzes user requests and routes them to the most appropriate specialist agent(s) without requiring explicit user mentions. The AI acts as an intelligent Project Manager, analyzing each request and automatically selecting the best specialist(s) for the job.

**Integration Point:** Use `/intelligent-routing` to automatically analyze user requests and determine appropriate specialist agent selection.

#### Agent Selection Matrix

| User Intent | Keywords | Selected Agent(s) | Auto-invoke? |
|-------------|----------|-------------------|--------------|
| Rust code review | "review rust code", "check rust patterns" | `rust-reviewer` | YES |
| Architecture design | "architecture", "design system", "event-driven" | `architect` | YES |
| Security review | "security", "vulnerability", "audit" | `security-reviewer` | YES |
| Performance optimization | "slow", "optimize", "performance" | `performance-optimizer` | YES |
| Backend development | "backend", "API", "server", "NATS" | `backend-specialist` | YES |
| Database design | "schema", "database", "migration", "SQLite" | `database-architect` | YES |

#### Domain Detection Rules

| Domain | Patterns | Agent |
|--------|----------|-------|
| Rust code | rust, ownership, lifetime, tokio, async | `rust-reviewer` |
| Architecture | architecture, design, system, event-driven | `architect` |
| Security | security, vulnerability, audit, credentials | `security-reviewer` |
| Performance | slow, optimize, performance, benchmark | `performance-optimizer` |
| Backend | backend, API, server, NATS, module | `backend-specialist` |
| Database | database, schema, migration, SQLite, query | `database-architect` |

#### Implementation Rules

**Rule 1: Silent Analysis**
- ✅ Analyze silently
- ✅ Inform which agent is being applied
- ❌ Avoid verbose meta-commentary

**Rule 2: Inform Agent Selection**
When auto-selecting an agent, inform the user concisely:
```markdown
**Applying knowledge of `@rust-reviewer`...**

[Proceed with specialized response]
```

**Rule 3: Seamless Experience**
The user should not notice a difference from talking to the right specialist directly.

**Rule 4: Override Capability**
User can still explicitly mention agents to override auto-selection.

#### Available Specialist Agents

- **rust-reviewer** - Rust code review with Wireframe-AI patterns
- **architect** - Event-driven architecture and system design
- **security-reviewer** - Rust and Provider system security
- **performance-optimizer** - Rust systems performance optimization
- **backend-specialist** - Rust modules and NATS messaging
- **database-architect** - SQLite schema and data modeling

## Git Workflow

- Conventional commits: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`
- Never commit to main directly
- PRs require review
- All tests must pass before merge
- Run `cargo fmt`, `cargo clippy`, and `cargo test` before committing
- Use `/git-commit` skill for automated commit workflow

## Quick Reference for Agents

### When Starting a New Feature

1. Use `/karpathy-guidelines` to establish behavioral standards
2. Use `/project-routing` to identify the right approach
3. Use `/orchestration-patterns` for swarm research (1 worker at a time)
4. Use `/architecture` for design decisions
5. Use `/implementation` for systematic development
6. Use `/quality-checklist` before committing
7. Use `/final-checks` before completion

### When Debugging

1. Use `/karpathy-guidelines` to establish behavioral standards (Think Before Coding, Goal-Driven Execution)
2. Use `/systematic-debugging` for 4-phase methodology
3. Use `/rust-pro` for Rust-specific patterns
4. Use `/run-rust-tests` to verify fixes

### Before Completing Work

1. Use `/karpathy-guidelines` to review for surgical changes and simplicity
2. Run `/check-rust-quality` for linting
3. Run `/run-rust-tests` for testing
4. Use `/quality-checklist` for comprehensive review
5. Use `/final-checks` for verification

### Wireframe-AI Specific Patterns to Remember

- **Topic naming**: `namespace.noun.verb` (lowercase, dot-separated)
- **Message envelopes**: Never change root fields, only payload
- **Module lifecycle**: Publish `sys.module.online`/`offline`
- **State ownership**: Context module owns all persistent state
- **Provider credentials**: Use vault, never hardcode
- **Unsafe code**: Must have `// SAFETY:` comment
- **Error handling**: Use `?` operator, never `unwrap()` in production

## References

- [AGENTS.md](../AGENTS.md) - Agent orchestration, workflow, and behavioral standards
- [.devin/rules/rust-coding-style.md](rules/rust-coding-style.md) - Rust coding conventions
- [.devin/rules/rust-patterns-wireframe.md](rules/rust-patterns-wireframe.md) - Wireframe-AI specific patterns
- [.devin/rules/rust-security.md](rules/rust-security.md) - Security guidelines
- [.devin/rules/rust-testing-wireframe.md](rules/rust-testing-wireframe.md) - Testing patterns
- [docs/getting-started/Project-Architecture.md](../docs/getting-started/Project-Architecture.md) - System architecture
- [docs/getting-started/Project-Core.md](../docs/getting-started/Project-Core.md) - Core concepts