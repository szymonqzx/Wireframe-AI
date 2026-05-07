---
paths:
  - "**/*.rs"
  - "**/Cargo.toml"
---
# Rust Hooks for Wireframe-AI

Automated hooks for Rust development in Wireframe-AI using Devin CLI.

## Post-Edit Hooks

Configure these hooks in `.devin/config.json` under the `hooks` section:

### Auto-Format

Automatically format Rust files after edits:

```json
{
  "hooks": {
    "postEdit": [
      {
        "pattern": "**/*.rs",
        "command": "cargo fmt"
      }
    ]
  }
}
```

### Lint Checks

Run clippy after editing Rust files:

```json
{
  "hooks": {
    "postEdit": [
      {
        "pattern": "**/*.rs",
        "command": "cargo clippy -- -D warnings"
      }
    ]
  }
}
```

### Compilation Check

Verify compilation after changes (faster than `cargo build`):

```json
{
  "hooks": {
    "postEdit": [
      {
        "pattern": "**/*.rs",
        "command": "cargo check"
      }
    ]
  }
}
```

## Pre-Commit Hooks

### Full Quality Check

Run comprehensive checks before committing:

```json
{
  "hooks": {
    "preCommit": [
      {
        "command": "cargo fmt --check"
      },
      {
        "command": "cargo clippy -- -D warnings"
      },
      {
        "command": "cargo test"
      }
    ]
  }
}
```

## Session Start Hooks

### Environment Validation

Validate the development environment at session start:

```json
{
  "hooks": {
    "sessionStart": [
      {
        "command": "cargo --version"
      },
      {
        "command": "rustc --version"
      }
    ]
  }
}
```

## Wireframe-AI Specific Hooks

### NATS Server Check

Ensure NATS server is running before starting kernel:

```json
{
  "hooks": {
    "preExec": [
      {
        "pattern": "cargo run --bin kernel",
        "command": "pgrep -x nats-server || echo 'NATS server not running'"
      }
    ]
  }
}
```

### Schema Validation

Validate schema files after changes:

```json
{
  "hooks": {
    "postEdit": [
      {
        "pattern": "schemas/**/*.json",
        "command": "scripts/validate-schemas.sh"
      }
    ]
  }
}
```

## Hook Configuration Example

Complete `.devin/config.json` example:

```json
{
  "permissions": {
    "allow": [
      "Read(**)",
      "Exec(git status)",
      "Exec(git diff)",
      "Exec(git log)",
      "Exec(cargo build)",
      "Exec(cargo test)",
      "Exec(cargo check)",
      "Exec(cargo clippy)",
      "Exec(cargo fmt)",
      "Exec(cargo run)"
    ],
    "deny": [
      "Exec(rm -rf)",
      "Exec(sudo)",
      "Exec(cargo clean)"
    ],
    "ask": [
      "Write(**/.env*)",
      "Write(**/Cargo.lock)",
      "Write(wireframe_ai_context.db*)"
    ]
  },
  "hooks": {
    "postEdit": [
      {
        "pattern": "**/*.rs",
        "command": "cargo fmt"
      },
      {
        "pattern": "**/*.rs",
        "command": "cargo clippy -- -D warnings"
      }
    ],
    "preCommit": [
      {
        "command": "cargo fmt --check"
      },
      {
        "command": "cargo test"
      }
    ]
  }
}
```

## Notes

- Hooks are executed in the order specified
- Pattern matching uses glob patterns
- Commands run in the project root directory
- Failed hooks block the operation (except postEdit hooks which are non-blocking by default)
- Use `--check` flag for fmt in preCommit to fail if formatting is needed