---
name: git-commit
description: Automatic git commit workflow for Wireframe-AI agents
allowed-tools:
  - exec
triggers:
  - model
---

# Git Commit Workflow for Wireframe-AI

All agents MUST automatically git commit when finishing a task.

## When to Commit

- After completing each feature, fix, or refactoring
- After passing all relevant tests
- After any substantial code change (>30 lines OR 3+ files modified)

## Commit Protocol

1. Stage all relevant changes: `git add <files>`
2. Commit with descriptive message: `git commit -m "<message>"`
3. Use conventional commit format when applicable
4. Reference team IDs when applicable: `// TEAM_XXX: <reason>`

## Commit Message Format

Use conventional commit format:
- `feat: add new feature`
- `fix: resolve bug in module`
- `refactor: improve code structure`
- `docs: update documentation`
- `test: add tests for feature`

Include team ID reference in code comments when applicable:
```rust
// TEAM_XXX: Reason for change
```

## Before Committing

- Ensure project builds cleanly
- Ensure all tests pass
- Ensure baseline/golden tests pass (if applicable)
