---
name: agentic-engineering
description: Operate as an agentic engineer using eval-first execution, decomposition, and cost-aware model routing for Wireframe-AI development.
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

# Agentic Engineering for Wireframe-AI

Use this skill for engineering workflows where AI agents perform most implementation work and humans enforce quality and risk controls in the Wireframe-AI codebase.

## Operating Principles

1. Define completion criteria before execution
2. Decompose work into agent-sized units
3. Route model tiers by task complexity
4. Measure with evals and regression checks
5. Respect Wireframe-AI's modular architecture and message contracts

## Eval-First Loop

1. Define capability eval and regression eval
2. Run baseline and capture failure signatures
3. Execute implementation
4. Re-run evals and compare deltas

## Task Decomposition

Apply the 15-minute unit rule:
- each unit should be independently verifiable
- each unit should have a single dominant risk
- each unit should expose a clear done condition
- each unit should respect module boundaries in Wireframe-AI

## Model Routing

- Haiku: classification, boilerplate transforms, narrow edits
- Sonnet: implementation and refactors
- Opus: architecture, root-cause analysis, multi-file invariants

## Session Strategy

- Continue session for closely-coupled units
- Start fresh session after major phase transitions
- Compact after milestone completion, not during active debugging

## Review Focus for AI-Generated Code

Prioritize:
- invariants and edge cases
- error boundaries
- security and auth assumptions
- hidden coupling and rollout risk
- NATS message contract compliance
- SQLite schema consistency
- Provider system integration

Do not waste review cycles on style-only disagreements when automated format/lint already enforce style.

## Cost Discipline

Track per task:
- model
- token estimate
- retries
- wall-clock time
- success/failure

Escalate model tier only when lower tier fails with a clear reasoning gap.

## Wireframe-AI Specific Considerations

### Module Development
- Always check `kernel/interface/src/main.rs` for module registration patterns
- Follow topic naming convention: `namespace.noun.verb` or `namespace.noun`
- Publish to `sys.module.online` on startup, `sys.module.offline` on shutdown
- Never change root fields in message envelopes, only payload

### Schema Changes
- Check `schemas/v1/` for current envelope contracts
- Maintain backward compatibility or provide migration path
- Validate schema changes with `/wireframe-workflow` skill

### Database Operations
- Context module owns all persistent state
- Use SQLite transactions for multi-step operations
- Follow database-design skill for schema changes

### NATS Integration
- Restart nats-server after configuration changes
- Use proper message serialization
- Handle connection failures gracefully

### Provider System
- Follow Provider trait for LLM provider integration
- Implement capability negotiation
- Test with multiple providers

## Verification Commands

```bash
cargo build --release  # Build
cargo test              # Test
cargo clippy            # Lint
cargo fmt               # Format
```

## Integration with Wireframe-AI Skills

- Use `/wireframe-workflow` for schema validation
- Use `/rust-pro` for Rust-specific guidance
- Use `/systematic-debugging` for debugging issues
- Use `/orchestration-patterns` for complex multi-module tasks
