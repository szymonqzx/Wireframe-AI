# Contributing to Wireframe-AI

Thank you for your interest in contributing to Wireframe-AI! This document provides guidelines and conventions for contributing to the project.

## Project Structure

Wireframe-AI is a modular, event-driven agentic system built with Rust. The project is organized into hierarchical workspaces for better build performance and separation of concerns.

### Workspace Organization

**Main Workspace** (Root `Cargo.toml`):
- Core modules (`modules/`): context-core, orchestrator-core, sandbox-core, interface-core
- Platform modules (`modules/`): event-sourcing-core, integrations-core, observability-core, provider-router-core, tenant-core, webhooks-core
- Plugins (`plugins/`): Extensible components for core modules
- Providers (`providers/`): LLM provider implementations
- SDK (`sdk/`): agentic-sdk and agentic-sdk-macros
- Adapter (`adapter/`): Rust and Python adapters
- Kernel (`kernel/`): Module orchestration and lifecycle

**Examples Workspace** (`examples/Cargo.toml`):
- Example modules demonstrating Wireframe-AI usage
- Built independently from main workspace
- Located in `examples/`

**Tools Workspace** (`tools/Cargo.toml`):
- TUI (Terminal User Interface) components
- CLI tools (wireframe-cli, wireframe-debug, wireframe-replay)
- Built independently from main workspace
- Located in `tools/`

## Module Naming Conventions

### Core Modules
- All core modules use `-core` suffix
- Format: `<function>-core`
- Examples: `context-core`, `orchestrator-core`, `sandbox-core`

### Platform Modules
- All platform modules use `-core` suffix
- Format: `<function>-core`
- Examples: `event-sourcing-core`, `integrations-core`, `observability-core`

### Plugins
- Organized by the core module they extend
- Format: `<module>/<category>/<name>`
- Examples: `plugins/context/storage-sqlite`, `plugins/orchestrator/planner-linear`

### Providers
- Named after the LLM service they integrate
- Format: `<provider-name>`
- Examples: `openai`, `anthropic`, `cohere`, `google`, `ollama`

## File Location Standards

### Module Structure
```
modules/<module-name>/
├── Cargo.toml          # Package configuration
├── Dockerfile          # Container build (if applicable)
├── src/
│   ├── lib.rs         # Library exports
│   └── main.rs        # Binary entry point (if applicable)
└── tests/             # Integration tests
```

### Plugin Structure
```
plugins/<module>/<category>/<plugin-name>/
├── Cargo.toml
└── src/
    └── lib.rs
```

### Documentation Structure
```
docs/
├── getting-started/    # Introduction and quick start
├── reference/          # API references and schemas
├── guides/             # How-to guides and tutorials
├── modules/            # Module-specific documentation
├── sdk/                # SDK documentation
├── operations/         # Deployment and operations
├── security/           # Security documentation
└── project/            # Project-level documentation
```

### Deployment Structure
```
deploy/
├── docker/             # Docker-related files
├── k8s/
│   └── manifests/      # Kubernetes manifests
└── monitoring/         # Monitoring configuration
    ├── grafana/
    └── prometheus/
```

### Scripts Structure
```
scripts/
├── *.ps1               # PowerShell scripts (Windows primary)
├── *.sh                # Bash scripts (Linux/macOS)
├── common.ps1          # Shared utilities
└── README.md           # Script documentation
```

### Tests Structure
```
tests/
├── rust/               # Rust-specific tests
│   ├── integration/    # Integration tests
│   └── benchmark_test.rs
└── python/             # Python-specific tests
    └── test_python_sdk.py
```

## Workspace Member Criteria

### Main Workspace Members
Include in main workspace if:
- Core system component (modules, plugins, providers, SDK)
- Required for building the main system
- Has dependencies on other workspace members

### Examples Workspace Members
Include in examples workspace if:
- Demonstrates Wireframe-AI usage
- Not required for main system build
- Educational or example code

### Tools Workspace Members
Include in tools workspace if:
- Development tool or utility
- Not required for main system build
- Can be built and used independently

## Code Style Guidelines

### Rust Code
- Follow Rust standard naming conventions
- Use `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` to lint
- Prefer `anyhow::Context` for error context in applications
- Use `thiserror` for library error types
- Never use `unwrap()` in production code
- Use `let` by default, `let mut` only when needed

### Module Implementation
- Use the `#[agentic_sdk::module]` macro for module registration
- Always enable `macros` feature: `agentic-sdk = { workspace = true, features = ["macros"] }`
- Clone envelope fields to avoid borrow checker issues
- Test plugin loading from configuration files
- Validate backward compatibility when making changes

### NATS Messaging
- Topic naming: `namespace.noun.verb` or `namespace.noun`, lowercase, dot-separated
- Never change root fields in message envelopes
- Use correlation IDs for request/response patterns
- Publish `sys.module.online` on startup, `sys.module.offline` on shutdown

### Documentation
- Add module-level documentation (`//!`) for public APIs
- Document public functions and structs with `///`
- Keep documentation up-to-date with code changes
- Use examples in documentation where helpful

## Development Workflow

### Setting Up
1. Clone the repository
2. Install Rust toolchain: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
3. Install PowerShell 7+ for cross-platform script support
4. Run `cargo build --release` to build the main workspace

### Building
```bash
# Build main workspace
cargo build --release

# Build examples workspace
cd examples && cargo build --release

# Build tools workspace
cd tools && cargo build --release
```

### Testing
```bash
# Run all tests
cargo test

# Run specific workspace tests
cargo test -p <package-name>

# Run with output
cargo test -- --nocapture
```

### Code Review
- Create descriptive pull requests
- Reference related issues
- Ensure all tests pass
- Update documentation as needed
- Follow the code style guidelines

## Commit Message Conventions

Use conventional commit format:
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code refactoring
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `test`: Test changes
- `chore`: Maintenance tasks

Examples:
- `feat(context-core): add LRU cache for state management`
- `fix(provider-router): handle missing provider gracefully`
- `refactor(workspace): create hierarchical workspaces`

## Getting Help

- Check existing documentation in `docs/`
- Review module code for examples
- Ask questions in issues or discussions
- Review `AGENTS.md` for agent-specific guidance

## License

By contributing to Wireframe-AI, you agree that your contributions will be licensed under the project's license.
