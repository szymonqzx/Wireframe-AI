# Wireframe AI — Distributed Agent Harness

A modular, message-bus-driven distributed agent system for building and running AI agents that collaborate across processes, languages, and machines.

[![asciicast](https://img.shields.io/badge/demo-terminal-blue)](scripts/run-demo.ps1)

## Quick Start

```bash
# One command: build everything + start all modules in separate windows
# Windows:
.\scripts\start-all.ps1

# Linux/macOS:
./scripts/start-all.sh

# Or step by step:
# 1. Start NATS
docker run -p 4222:4222 nats:latest

# 2. Start context module
cargo run -p wireframe-ai-context-core &

# 3. Start orchestrator (optional — falls back to direct adapter)
cargo run -p wireframe-ai-orchestrator-core &

# 4. Run the interface
cargo run -p wireframe-ai-interface "Write a Python script to sort a CSV file"
```

### Launch all modules at once

**Windows (PowerShell):**
```bash
.\scripts\start-all.ps1                    # release mode (default)
.\scripts\start-all.ps1 -BuildMode debug   # debug builds
.\scripts\start-all.ps1 -SkipOrchestrator  # without orchestrator
.\scripts\start-all.ps1 -SkipAdapter       # without Python adapter
.\scripts\start-all.ps1 -SkipBuild         # reuse existing binaries
```

**Linux/macOS (Bash):**
```bash
./scripts/start-all.sh                    # release mode (default)
./scripts/start-all.sh --debug            # debug builds
./scripts/start-all.sh --skip-orchestrator # without orchestrator
./scripts/start-all.sh --skip-adapter     # without Python adapter
./scripts/start-all.sh --skip-build       # reuse existing binaries
```

Both scripts open each module in its own terminal window with a labeled title bar, so you can monitor their output independently.

Each window is color-labeled in the title bar (`wireframe-context`, `wireframe-orchestrator`, `wireframe-sandbox`, `wireframe-adapter-python`). The interface opens in the current terminal for input. Press Ctrl+C in the interface to shut everything down cleanly.

## Architecture

```
Wireframe-AI/
├── kernel/                    ← what the end user always downloads
│   ├── nats                   (the bus binary — download separately)
│   └── interface/             (the only truly required Rust module)
├── modules/                   ← official optional modules, mix and match
│   ├── context-core/          (session memory, enrichment with plugin support)
│   ├── orchestrator-core/     (fan-out / fan-in coordinator with plugin support)
│   └── sandbox-core/          (MCP server: file and shell tools with plugin support)
├── sdk/                       ← what third-party module authors import
│   ├── agentic-sdk/           (Rust crate: envelope, Module trait, NATS helpers)
│   └── agentic-sdk-py/        (Python package: same for adapter authors)
├── schemas/                   ← the public API, versioned and documented
│   └── v1/
│       ├── TOPICS.md           (every topic, what it carries, who uses it)
│       ├── envelope.json
│       ├── task_submitted.json
│       ├── task_enriched.json
│       ├── task_complete.json
│       ├── agent_job.json
│       ├── agent_result.json
│       ├── provider_describe.json
│       ├── provider_metadata.json
│       ├── provider_status.json
│       ├── provider_list_request.json
│       └── provider_list_response.json
├── adapter/
│   ├── rust/                  (Rust adapter with Provider trait)
│   └── python/                (Python reference adapter - legacy)
├── provider-core/             ← Provider trait and core infrastructure
├── providers/                 ← LLM provider implementations
│   ├── openai/                (OpenAI-compatible HTTP provider)
│   ├── anthropic/             (Anthropic HTTP provider - future)
│   └── local/                 (Local model stdio provider - future)
└── docs/
    ├── getting-started/
    ├── reference/
    ├── guides/
    ├── tui/
    ├── architecture/
    ├── integrations/
    ├── operations/
    ├── security/
    ├── planning/
    └── project/
```

## Communication Model

All modules communicate through **NATS** using a uniform JSON envelope. No module talks directly to another — they only publish and subscribe to typed topics.

| Topic | Purpose |
|---|---|
| `task.submitted` | User submits a new request (Interface → Context) |
| `task.enriched` | Task enriched with memory/context (Context → Orchestrator) |
| `agent.job` | Self-contained job for an adapter (Orchestrator → Adapter) |
| `agent.result` | Result from a completed agent job (Adapter → Orchestrator) |
| `task.complete` | Final result (Orchestrator → Interface) |
| `sys.module.online` | Module announces itself on startup |
| `sys.module.offline` | Module announces graceful shutdown |

## Session Management

Wireframe-AI uses `correlation_id` and `correlation_parent` for tracking conversations across the distributed pipeline.

### Design

- **`correlation_id`** — Uniquely identifies a single message/envelope in the NATS bus. Generated fresh for each published message.
- **`correlation_parent`** — References the `correlation_id` of the parent message that triggered this one. Forms a tree of causality.
- **`session_id`** — Identifies a user conversation. Carried from `task.submitted` through the entire pipeline.

### Adapter Sessions

The reasoning adapter maintains a `SessionManager` for multi-turn conversations with LLM providers. Because `AgentJob` does not carry a dedicated `session_id` field, the adapter uses `correlation_parent` as the session identifier:

```rust
let session_id = session_manager
    .ensure_session(Some(&job.correlation_parent), &provider_name, &model);
```

This design decision was made because:
1. Each `AgentJob` is spawned from a single `task.enriched`, so `correlation_parent` is stable for the lifetime of the job.
2. Multiple `AgentJob` messages with the same `correlation_parent` share a session, enabling multi-turn tool use within one task.
3. The orchestrator controls correlation assignment, making it the natural authority for session boundaries.

### Future Considerations

If the orchestrator needs explicit session management (e.g., for cross-task memory, user-level sessions, or TTL-based eviction), a dedicated `session_id` field could be added to `AgentJob`. The adapter would then use that field instead of `correlation_parent`.

## LLM Providers

Wireframe-AI uses a unified Provider trait for LLM communication, supporting multiple providers through a common interface.

### Provider System

The Rust adapter implements a provider system with:
- **Provider trait** - Unified interface for all LLM providers
- **Session management** - Multi-turn conversation support
- **Capability negotiation** - Provider discovery and feature detection
- **Streaming responses** - Real-time token streaming
- **Multiple transports** - HTTP (cloud APIs) and stdio (local models)

### Available Providers

| Provider | Transport | Status |
|---|---|---|
| `openai` | HTTP | ✅ Implemented |
| `anthropic` | HTTP | 🚧 Planned |
| `local` | stdio | 🚧 Planned |

### Provider Configuration

Providers are configured in the reasoning adapter via the `ProviderRegistry`. The `model_config` field in an AgentJob specifies which provider to use:

```json
{
  "model_config": {
    "provider": "openai",
    "model_name": "gpt-4o",
    "temperature": 0.7,
    "max_tokens": 4096
  }
}
```

### Adding a New Provider

To add a new LLM provider:

1. Create a new crate in `providers/<provider-name>/`
2. Implement the `Provider` trait from `provider-core`
3. Register the provider in the reasoning adapter's `ProviderRegistry`
4. Add the provider to the workspace in `Cargo.toml`

See `docs/getting-started/Provider-System.md` for detailed implementation guidance.

### Legacy Python Adapter

The Python adapter with JSON config remains available as a legacy implementation during the migration period. New development should use the Rust adapter with the Provider system.

## Selfdev Mode

Wireframe-AI supports **selfdev mode** (self-development), allowing agents to modify their own source code at runtime, recompile, and restart with the new version. This enables autonomous optimization and bug fixing.

### Selfdev Capabilities

When selfdev mode is enabled, agents can:
- **Read their own source code** via the `read_source` tool
- **Modify their source code** via the `write_source` tool
- **Compile themselves** via the `compile_self` tool
- **Restart with new binary** via the `restart_self` tool

### Execution Modes

Wireframe-AI supports three execution modes:

| Mode | Description | Use Case |
|------|-------------|----------|
| **Sandbox** | Isolated execution in sandbox environment | Production, security-critical |
| **Direct** | Direct execution on host PC | Development, selfdev |
| **Hybrid** | Sandbox for normal ops, direct for selfdev | Balanced security and flexibility |

### Enabling Selfdev Mode

Set the following environment variables:

```bash
# Enable selfdev mode
export WIREFRAME_AI_SELFDEV=true

# Set source root (defaults to current directory)
export WIREFRAME_AI_SOURCE_ROOT=/path/to/wireframe-ai

# Set execution mode (optional, defaults to direct)
export WIREFRAME_AI_EXECUTION_MODE=direct
```

### Selfdev Intent Detection

Selfdev mode activates automatically when the agent's prompt suggests code modification. Keywords include:
- "edit my code"
- "modify myself"
- "improve my implementation"
- "change my source"
- "update my code"

### Process Manager

The process manager (`scripts/process-manager.ps1`) handles module rebuilds and restarts:

```bash
# Auto-restart a module after compilation
.\scripts\process-manager.ps1 -Module wireframe-adapter-rust -AutoRestart

# Interactive mode (wait for manual restart requests)
.\scripts\process-manager.ps1 -Module wireframe-adapter-rust
```

### Security Considerations

Selfdev mode increases the attack surface. Mitigations include:
- **Intent detection**: Selfdev only activates when explicitly requested
- **Audit logging**: All selfdev operations logged to `~/.wireframe-ai/selfdev.log`
- **Rollback capability**: Previous binaries kept for recovery
- **Optional approval**: Human approval workflow for production
- **Rate limiting**: Prevents abuse and infinite self-modification loops

### Shell Command Allowlist

The adapter enforces a strict allowlist for `shell_exec` to prevent command injection. Only specific safe commands are permitted.

**Allowed commands:** `ls`, `dir`, `cd`, `pwd`, `echo`, `cat`, `type`, `head`, `tail`, `grep`, `find`, `git`, `cargo`, `npm`, `pip`, `python`, `python3`, `node`, `rustc`, `clang`, `gcc`, `mkdir`, `rmdir`, `rm`, `del`, `cp`, `copy`, `mv`, `move`, `touch`, `file`, `stat`, `wc`, `sort`, `uniq`, `cut`, `awk`, `sed`

**Expansion process:** To add a new command to the allowlist:

1. Edit `ALLOWED_COMMANDS` in `adapter/rust/src/main.rs`
2. Add the base command name (e.g., `"jq"`) to the array
3. Run safety checks: `cargo test -p wireframe-adapter-rust --bin wireframe-adapter`
4. Verify no new dangerous patterns are introduced
5. Commit with a note explaining the use case

Commands are validated for:
- **Base command presence** in `ALLOWED_COMMANDS`
- **Dangerous pattern blocking** (`|`, `&&`, `||`, `;`, `$(`, `` ` ``, newlines)
- **Argument metacharacter filtering** (`$`, `` ` ``, `\`, `"`, `'`, `<`, `>`, `*`, `?`, `[`, `]`, `{`, `}`)

### NATS Coordination

Selfdev uses NATS for coordination:
- `module.rebuild.request` - Request to rebuild a module
- `module.rebuild.status` - Build status updates
- `module.restart.request` - Request to restart with new binary
- `module.restart.ack` - Acknowledgment of restart

See `docs/architecture/adr-020-selfdev-mode.md` for detailed architecture and security model.

## Runtime Module Switching

Wireframe-AI supports **runtime module switching**, allowing agents to switch from one module to another at runtime (e.g., from `wireframe-adapter-rust` to a community module).

### Module Switching Capabilities

When module switching is enabled, agents can:
- **Switch to different modules** via the `switch_module` tool
- **Verify compatibility** before switching
- **Rollback on failure** to previous module
- **Coordinate via NATS** for distributed switching

### Module Registry

The module registry tracks installed modules with their metadata:
- Module ID and type (adapter, context, orchestrator, etc.)
- Binary path and source path
- Version and interface definition
- Enabled/disabled status

### Compatibility Checking

Before switching, the system verifies:
- **Interface compatibility** - New module subscribes/publishes required topics
- **Type compatibility** - New module implements required interface for its type
- **Breaking changes** - Detects incompatible topic changes

### Process Manager

The process manager handles module switching:
```bash
# Switch from one module to another
.\scripts\process-manager.ps1 -Switch -OldModule wireframe-adapter-rust -NewModule community-adapter-x

# Force switch (skip compatibility checks)
.\scripts\process-manager.ps1 -Switch -OldModule wireframe-adapter-rust -NewModule community-adapter-x -Force
```

### NATS Coordination

Module switching uses NATS for coordination:
- `module.switch.request` - Request to switch modules
- `module.switch.ack` - Acknowledgment of switch operation

### Module Types

Wireframe-AI supports different module types with specific requirements:

| Type | Required Subscriptions | Required Publications |
|------|----------------------|----------------------|
| **Adapter** | `agent.job` | `agent.result` |
| **Context** | `task.submitted` | `task.enriched` |
| **Orchestrator** | `task.enriched` | `agent.job`, `task.complete` |

### Usage Example

Agent can switch modules:
```json
{
  "tool": "switch_module",
  "parameters": {
    "new_module": "community-adapter-x",
    "force": false
  }
}
```

The system will:
1. Check compatibility with current module
2. Stop current module
3. Start new module
4. Verify new module is online
5. Acknowledge completion or rollback on failure

### Security Considerations

Module switching introduces additional security considerations:
- **Compatibility validation** - Prevents breaking changes
- **Rollback capability** - Automatic recovery on failure
- **Audit logging** - All switches logged for traceability
- **Force mode** - Requires explicit override for incompatible switches

See SDK documentation for `CompatibilityChecker`, `ModuleRegistry`, and `ModuleSwitchCoordinator` for implementation details.

## Design Principles

- **Message bus is the backbone** — every module is replaceable because they only speak NATS
- **State ownership** — persistent state is owned by exactly one module (Context); everything else is stateless
- **Self-contained jobs** — each `AgentJob` carries everything the adapter needs (no external queries during execution)
- **SDK-first** — the `agentic-sdk` provides everything a module author needs: envelope, types, Module trait, NATS helpers

## Building

```bash
# Full build (all modules + SDK)
cargo build --release

# Build with schema validation enabled (validates messages against JSON schemas)
cargo build --release --features schema-validation

# Run individual modules
cargo run -p wireframe-ai-interface -- "your task"
cargo run -p wireframe-ai-context-core
cargo run -p wireframe-ai-orchestrator-core
cargo run -p wireframe-ai-sandbox-core
```

## Schema Validation

Wireframe AI includes optional JSON Schema validation for message payloads. When enabled, the system validates all messages against embedded schemas before publishing them to NATS. This ensures contract compliance between modules.

**To enable schema validation:**

```bash
# Build with the schema-validation feature
cargo build --release --features schema-validation

# Or enable it for a specific module
cargo build -p wireframe-ai-interface --features schema-validation
```

**Schemas are embedded in the binary for:**
- `task.submitted` → `EMBEDDED_TASK_SUBMITTED_SCHEMA`
- `task.enriched` → `EMBEDDED_TASK_ENRICHED_SCHEMA`
- `task.complete` → `EMBEDDED_TASK_COMPLETE_SCHEMA`
- `agent.job` → `EMBEDDED_AGENT_JOB_SCHEMA`
- `agent.result` → `EMBEDDED_AGENT_RESULT_SCHEMA`

The validation is performed by the `agentic-sdk::validate_envelope_payload()` function, which automatically selects the appropriate embedded schema based on the topic and validates the payload. If validation fails, the module logs an error and refuses to publish the message.

**Why embedded schemas?** Schemas are embedded as string constants in the binary rather than loaded from files at runtime. This ensures that:
- Schema validation works regardless of where the binary is executed from
- No file system dependencies or path resolution issues
- Schemas are always available and version-locked with the binary
- No risk of schema files being modified or deleted in production

**Note:** Schema validation is optional and disabled by default to avoid runtime overhead. Enable it in production environments where contract compliance is critical.

## Environment Variables

Wireframe-AI uses environment variables for configuration. Set these before running any module.

### Core

| Variable | Default | Description |
|---|---|---|
| `WIREFRAME_AI_NATS_URL` | `nats://localhost:4222` | NATS server URL |
| `WIREFRAME_AI_EXECUTION_MODE` | `direct` | Tool execution mode: `sandbox`, `direct`, or `hybrid` |
| `WIREFRAME_AI_ALLOWED_BASE_DIR` | Current directory | Restrict file operations to this directory |
| `WIREFRAME_AI_SHELL` | Platform default | Override shell for command execution (e.g., `powershell`, `bash`) |

### Sandbox

| Variable | Default | Description |
|---|---|---|
| `WIREFRAME_AI_SANDBOX_ROOT` | Temp directory | Root directory for sandbox file operations |
| `WIREFRAME_AI_SANDBOX_ALLOWED_COMMANDS` | Built-in whitelist | JSON array of allowed shell commands in sandbox |

### Selfdev

| Variable | Default | Description |
|---|---|---|
| `WIREFRAME_AI_SELFDEV` | `false` | Enable self-development mode (`true` or `1`) |
| `WIREFRAME_AI_SOURCE_ROOT` | Current directory | Source root for selfdev read/write/compile operations |

### Provider API Keys

| Variable | Required For | Description |
|---|---|---|
| `OPENAI_API_KEY` | `openai` provider | OpenAI API key |
| `DEEPSEEK_API_KEY` | `deepseek` provider | DeepSeek API key |
| `OPENCODE_GO_API_KEY` | `opencode-go` provider | OpenCode Go API key |

### Example

```bash
# Linux/macOS
export WIREFRAME_AI_NATS_URL=nats://localhost:4222
export WIREFRAME_AI_EXECUTION_MODE=hybrid
export WIREFRAME_AI_SELFDEV=true
export WIREFRAME_AI_SOURCE_ROOT=/home/user/wireframe-ai
export OPENAI_API_KEY=sk-...

# Windows (PowerShell)
$env:WIREFRAME_AI_NATS_URL="nats://localhost:4222"
$env:WIREFRAME_AI_EXECUTION_MODE="hybrid"
$env:WIREFRAME_AI_SELFDEV="true"
$env:OPENAI_API_KEY="sk-..."
```

## Python Adapter

```bash
pip install -e sdk/agentic-sdk-py
pip install -e adapter/python
wireframe-ai-adapter-python
```

## Scripts

| Script | Purpose |
|---|---|
| `scripts/start-all.ps1` | **Start everything** — NATS + all modules in separate windows + interface |
| `scripts/download-nats.ps1` | Download NATS server binary |
| `scripts/build-release.ps1` | Build all modules in release mode |
| `scripts/run-demo.ps1` | Start NATS + context + orchestrator (background jobs) |
| `scripts/cross-build.ps1` | Build for Linux, macOS, Windows, ARM64 |
| `scripts/smoke-test.ps1` | End-to-end pipeline verification |
| `adapter/python/add_provider.py` | Interactive LLM provider setup |

## Cross-Compilation

```bash
# Prerequisites
rustup target add x86_64-unknown-linux-gnu aarch64-apple-darwin x86_64-pc-windows-gnu

# Build all targets
.\scripts\cross-build.ps1

# Or download NATS and do a full demo
.\scripts\download-nats.ps1
.\scripts\run-demo.ps1
```

### Cross-Compilation Requirements

The `.cargo/config.toml` file specifies cross-compilation linkers. To use these targets, you need to install the appropriate toolchains:

**Linux (Ubuntu/Debian):**
```bash
# For x86_64-linux-gnu (usually pre-installed)
sudo apt-get install gcc-x86-64-linux-gnu

# For aarch64-linux-gnu (ARM64)
sudo apt-get install gcc-aarch64-linux-gnu

# For x86_64-w64-mingw32 (Windows cross-compilation)
sudo apt-get install gcc-mingw-w64-x86-64
```

**macOS:**
```bash
# For aarch64-apple-darwin (Apple Silicon) - use Xcode toolchain
# No additional installation needed if you have Xcode Command Line Tools
xcode-select --install
```

**Windows:**
```bash
# For cross-compiling to Linux from Windows, consider using Docker
# or the `cross` tool which handles this automatically:
cargo install cross
```

If you don't have these linkers installed, you can still build for your native platform using `cargo build --release` without cross-compilation targets.

## Example Module

The `examples/ping-module/` shows how to write a module with the `#[module]` proc-macro:

```bash
cargo run -p ping-module
```

```rust
#[module(
    subscribes = ["ping.request"],
    publishes  = ["ping.response"],
    queue_group = "ping_handler"
)]
impl Module for PingModule {
    async fn handle(&mut self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        vec![env.reply("ping.response", json!({"reply": "pong"}))]
    }
}
```

## Interface UX

The interface (`wireframe-ai-interface`) provides an interactive terminal experience:

- **Banner** on startup showing system identity
- **Connection status** while connecting to NATS
- **Input prompt** with clear instructions for multiline task entry
- **Live spinner** while waiting for the pipeline to complete
- **Formatted output** with sectioned results, side effects (files written, commands run), and warnings
- **Elapsed time** displayed on completion
- Supports direct task via argument: `cargo run -p wireframe-ai-interface -- "your task"` (banner suppressed in quiet mode with `--quiet`)

## Documentation

Comprehensive documentation for Wireframe-AI:

- **[Plugin Development Guide](docs/guides/Plugin-Development-Guide.md)** - Complete guide for developing plugins for Wireframe-AI modules
- **[Configuration Examples](docs/guides/Configuration-Examples.md)** - Configuration examples for all modules and plugins
- **[API Reference](docs/reference/API-Reference.md)** - Detailed API reference for SDK traits and types
- **[SDK Quick Start](docs/guides/SDK-Quick-Start.md)** - Get started with the Wireframe-AI SDK
- **[Project Architecture](docs/getting-started/Project-Architecture.md)** - System architecture overview
- **[Project Core](docs/getting-started/Project-Core.md)** - Core concepts and design principles
- **[Provider System](docs/getting-started/Provider-System.md)** - LLM provider system documentation
- **[NATS Message Envelope](docs/reference/schemas/NATS-MESSAGE-ENVELOPE.md)** - Message envelope specification

### Configuration Examples

- **[Complete System Configuration](examples/configurations/complete-system.yaml)** - Production-ready configuration with all modules enabled
- **[Minimal System Configuration](examples/configurations/minimal-system.yaml)** - Minimal configuration for getting started

### Plugin Development

- **[Plugin Templates](templates/plugins/)** - Ready-to-use templates for creating custom plugins
  - [Context Storage Template](templates/plugins/context-storage-template.rs)
  - [Orchestrator Planner Template](templates/plugins/orchestrator-planner-template.rs)
  - [Sandbox Tool Template](templates/plugins/sandbox-tool-template.rs)
- **[Hello World Plugin Tutorial](docs/guides/Hello-World-Plugin-Tutorial.md)** - Step-by-step tutorial for creating your first plugin

### Performance

- **[Plugin Benchmarks](benchmarks/)** - Performance benchmarks for Wireframe-AI plugins
- **[Performance Optimization Guide](docs/project/Performance-Optimization-Guide.md)** - Comprehensive performance optimization strategies
- **[Load Testing Script](scripts/load-test.sh)** - Load testing script for performance validation

### Production

- **[Production Deployment Guide](docs/project/Production-Deployment-Guide.md)** - Production deployment strategies and best practices
- **[Production Configuration](configs/production.yaml)** - Production-ready configuration with optimized settings
- **[Monitoring Infrastructure](monitoring/)** - Metrics collection and distributed tracing setup

### Additional Documentation

- **[Universal Modularization Plan](docs/project/Universal-Modularization-Plan.md)** - Overview of the modularization effort
- **[Best Practices](docs/project/Best-Practices.md)** - Development best practices for Wireframe-AI
- **[Security](docs/project/SECURITY.md)** - Security considerations and guidelines

## Platform Features

Wireframe-AI includes several platform-level features for production deployments:

### Multi-Tenancy (Tenant Module)

The tenant module provides multi-tenant isolation with:
- **Tenant Creation**: Create isolated tenant environments with unique IDs
- **Resource Quotas**: Configure per-tenant limits for tokens, requests, and concurrent jobs
- **Provider Allowlists**: Restrict which LLM providers each tenant can use
- **Topic Isolation**: Control which NATS topics tenants can access
- **Usage Tracking**: Monitor resource consumption per tenant
- **Tenant Suspension**: Isolate problematic tenants

**Usage:**
```bash
# Start tenant module
cargo run -p wireframe-ai-tenant-core

# Create a tenant via NATS
# Publish to tenant.create with: { "id": "tenant-123", "name": "Acme Corp", ... }
```

### Observability (Observability Module)

The observability module provides comprehensive monitoring:
- **Metrics Collection**: Store and query performance metrics
- **Distributed Tracing**: Track request flows across modules with span trees
- **Log Aggregation**: Centralized logging with correlation ID tracking
- **Health Monitoring**: Real-time health status for all modules
- **Historical Analysis**: SQLite persistence for trend analysis

**Usage:**
```bash
# Start observability module
cargo run -p wireframe-ai-observability-core

# Modules publish metrics to metrics.>
# Modules publish traces to trace.>
# Modules publish logs to log.>
```

### Centralized Configuration

The config crate provides unified configuration management:
- **Environment Variables**: Override defaults via environment variables
- **TOML Files**: Load configuration from files
- **Hot Reload**: Automatically reload configuration when files change
- **Module-Specific Config**: Separate configuration sections for each module
- **Type-Safe**: Compile-time configuration validation

**Usage:**
```rust
use wireframe_config::{WireframeConfig, ConfigManager};

// Load from environment
let config = WireframeConfig::from_env()?;

// Load from file with hot reload
let mut manager = ConfigManager::new();
manager.load_from_file("config.toml").await?;
manager.enable_hot_reload().await?;

// Get current config
let config = manager.get().await;
```

### Message Inspector (Wireframe Debug)

The wireframe-debug tool provides real-time message inspection:
- **Topic Filtering**: Subscribe to specific NATS topics
- **Correlation Tracking**: Filter messages by correlation ID
- **Multiple Formats**: Pretty, JSON, or compact output
- **Message Capture**: Save messages to files for later replay
- **Headers-Only Mode**: View envelope metadata without payloads

**Usage:**
```bash
# Inspect all messages
wireframe-debug --topic ">"

# Filter by correlation ID
wireframe-debug --correlation abc-123 --format json

# Capture messages
wireframe-debug --topic "task.>" --capture messages.jsonl
```

### Enhanced CLI Tooling

The wireframe-cli tool provides module management commands:
- **Module List**: List all installed modules with details
- **Module Start**: Start specific modules with build mode control
- **Module Stop**: Stop running modules (graceful or force)
- **Module Status**: Check running status of modules
- **Module Logs**: View module logs

**Usage:**
```bash
# List all modules
wireframe module list --detailed

# Start a module
wireframe module start wireframe-ai-context-core --build-mode release

# Stop a module
wireframe module stop wireframe-ai-context-core --force

# Check status
wireframe module status wireframe-ai-context-core
```
