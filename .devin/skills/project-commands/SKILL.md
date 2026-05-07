---
name: project-commands
description: Quick reference for Wireframe-AI development commands
allowed-tools:
  - exec
triggers:
  - model
---

# Wireframe-AI Project Commands

Quick reference for common Wireframe-AI development commands.

## Build Commands

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Check without building
cargo check
```

## Test Commands

```bash
# Run all tests
cargo test

# Run specific test
cargo test <test_name>

# Run tests with output
cargo test -- --nocapture
```

## Linting & Formatting

```bash
# Run Clippy linter
cargo clippy

# Format code
cargo fmt

# Check formatting without modifying
cargo fmt --check
```

## Running the System

```bash
# Start NATS server (required for Wireframe-AI)
nats-server

# Run kernel in another terminal
cargo run --bin kernel

# Run specific module
cargo run --bin <module_name>
```

## Database

```bash
# The context database is managed by the context module
# Located at: wireframe_ai_context.db
# Use SQLite tools to inspect if needed
```

## Git Workflow

```bash
# Check status
git status

# View diff
git diff

# View staged diff
git diff --staged

# Commit (agents should auto-commit per AGENTS.md)
git commit -m "message"
```

## Useful Aliases

Consider adding these to your shell config:
```bash
alias wb='cargo build --release'
alias wt='cargo test'
alias wc='cargo clippy'
alias wf='cargo fmt'
```

## Getting Help

- Use `/project-routing` to select the right agent/skill for your task
- Use `/wireframe-workflow` for file dependency awareness
- Use `/quality-checklist` for quality checks
- See AGENTS.md for project context and architectural decisions
