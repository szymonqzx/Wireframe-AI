# AGENTS.md

**Purpose:** Devin CLI orchestration for Wireframe-AI. Shared operating contract for Devin CLI and Windsurf.

## Mission

Ship small, correct, reviewed changes. Prefer evidence over vibes.

## Defaults

- Read existing code before editing
- Prefer minimal diffs over rewrites
- Never invent APIs, commands, benchmarks, or pricing
- Cite files and commands when reporting
- Run the narrowest relevant check before declaring done

## Karpathy Guidelines (Behavioral Standards)

Derived from Andrej Karpathy's observations on LLM coding pitfalls. Use `/karpathy-guidelines` skill for detailed guidance.

**Four Core Principles:**

1. **Think Before Coding** - State assumptions explicitly, present multiple interpretations, push back when simpler approaches exist, stop when confused
2. **Simplicity First** - Minimum code that solves the problem, no speculative features, no over-abstraction
3. **Surgical Changes** - Touch only what you must, clean up only your own mess, match existing style
4. **Goal-Driven Execution** - Define success criteria, transform tasks into verifiable goals, loop until verified

**When to Apply:**
- Non-trivial feature implementation
- Schema modifications
- Complex debugging
- Multi-step refactoring

**When to Relax:**
- Simple typo fixes
- Obvious one-liners
- Adding single imports
- Clear syntax errors

**Integration Point:** Always invoke `/karpathy-guidelines` before starting implementation work, alongside `/wireframe-workflow` and `/project-routing`.

## Defensive Prompting Examples

### When adding a new module:
"Add a new module for X. Start by examining kernel/interface/src/main.rs for module registration patterns, then check schemas/v1/ for envelope contracts. Be careful to follow the topic naming convention (namespace.noun.verb)."

### When debugging NATS issues:
"Debug the NATS communication issue. Check kernel/interface/src/main.rs for NATS setup, then identify which module publishes the problematic message. You'll need to restart nats-server after any configuration changes."

### When modifying schemas:
"Update the schema for X. First check schemas/v1/ for the current schema structure, then identify all modules that use this schema. Be careful to maintain backward compatibility or provide a migration path."

### When adding tests:
"Add unit tests for the context module. Focus on the state management functions in modules/context/src/lib.rs. Mock the NATS connection and SQLite database. Test edge cases around concurrent state updates."

### When performance optimization:
"Optimize the message flow in the orchestrator. Analyze modules/orchestrator/src/lib.rs for async patterns and serialization bottlenecks. Run cargo bench before and after changes to validate improvements."

## Project Context

Wireframe-AI: Modular, event-driven agentic system (Rust core with Provider system) with NATS message bus and SDK-based architecture.

**Key patterns:**
- State ownership: Context module owns all persistent state
- Message envelope: Never change root fields, only payload
- Topic naming: `namespace.noun.verb` or `namespace.noun`, lowercase, dot-separated
- Module identity: Publish to `sys.module.online` on startup, `sys.module.offline` on shutdown
- Provider system: Unified Provider trait for LLM providers with capability negotiation

**Essential commands:**
```bash
cargo build --release  # Build
cargo test              # Test
cargo clippy            # Lint
cargo fmt               # Format
nats-server && cargo run --bin kernel  # Run
cargo run --release --bin tui-minimal  # Run TUI
```

**TUI Provider Management:**
The minimal TUI includes provider configuration via TOML file:
- Configuration file: `tui-config.toml` in working directory
- Provider settings: name, API key environment variable, model
- Set current provider via `current_provider` field
- See `tools/tui-minimal/README.md` for configuration details

## Plugin Architecture Development Patterns

**Overview:** Wireframe-AI uses a plugin architecture where core modules (context-core, orchestrator-core, sandbox-core, interface-core) load and manage plugins through trait implementations.

**Key Patterns:**

### Plugin Development
- **Plugin Trait**: All plugins implement the base `Plugin` trait with lifecycle methods (plugin_id, version, description, initialize, health_check, shutdown)
- **Module-Specific Traits**: Each module has specific plugin traits (StorageBackend, MemoryBackend, TaskPlanner, ExecutionStrategy, Tool, SecurityPolicy, etc.)
- **Configuration**: Plugins are configured via YAML files in `configs/` directory
- **Plugin Registry**: Universal plugin registration and retrieval with type-safe downcasting

### Module Development
- **Core Modules**: Use plugin architecture to delegate functionality to plugins
- **NATS Communication**: Core modules handle NATS communication and plugin orchestration
- **Configuration Loading**: Load plugins from configuration files using PluginRegistry
- **Feature Flags**: Always enable `macros` feature in agentic-sdk dependency for module macro support

### Configuration Patterns
- **Environment Variables**: Use `${VAR}` syntax for environment variable expansion in configs
- **Validation**: Use jsonschema feature for configuration validation
- **Hot-Reload**: ConfigWatcher provides hot-reload support for configuration changes
- **Default Configs**: Each module has a default configuration in `configs/<module>-default.yaml`

### Testing Patterns
- **Plugin Tests**: Test plugin lifecycle, health checks, and error handling
- **Integration Tests**: Test plugin loading, initialization, and end-to-end functionality
- **Configuration Tests**: Test YAML/JSON parsing, environment variable expansion, and validation
- **Builder Tests**: Test type-safe message builders for compile-time validation

### Essential Commands for Plugin Development
```bash
cargo test -p agentic-sdk                    # Test SDK and plugin system
cargo test -p <module-core>                   # Test specific core module
cargo test -p <plugin-name>                  # Test specific plugin
cargo run --bin wireframe-cli -- new <name>   # Scaffold new module
```

### Module Feature Requirements
When creating or modifying core modules:
- **Always enable macros feature**: `agentic-sdk = { workspace = true, features = ["macros"] }`
- **Use module macro**: `#[agentic_sdk::module(subscribes = [...], publishes = [...])]`
- **Clone envelope fields**: Avoid borrow checker issues by cloning instead of moving envelope fields
- **Test plugin loading**: Ensure plugins can be loaded from configuration files
- **Validate backward compatibility**: Ensure existing functionality is preserved

## Quick Start

**New to Wireframe-AI?** Start here:
1. Read `docs/Project-Core.md` for system overview
2. Read `docs/Project-Architecture.md` for architecture
3. Read `docs/Provider-System.md` for provider system details
4. Use `/project-routing` to find the right agent/skill for your task
5. Use `/wireframe-workflow` before making any changes
6. See `.devin/SKILLS.md` for complete skills index

## Swarm Orchestration

Wireframe-AI leverages **swarm orchestration** for complex tasks - using multiple parallel subagents to achieve faster research and comprehensive analysis.

**IMPORTANT RATE LIMIT:** Never spawn more than 1 subagent at once. Use sequential workers for comprehensive research.

**When to use swarms:**
- Adding new modules or features (1 worker at a time, sequential for comprehensive research)
- Debugging complex issues (1 worker for investigation)
- Performance optimization (1 worker for bottleneck analysis)
- Security audits (1 worker at a time for comprehensive security review)
- Schema migrations (1 worker for impact analysis)

**Key swarm skill:** `/orchestration-patterns` - Auto-invokes for complex tasks, includes Wireframe-AI specific swarm examples with 1-worker pattern

**Performance benefit:** Sequential workers provide systematic coverage while avoiding rate limits

## Common Task → Skill Mapping

|| Task | Skill |
||------|-------|
|| Implement a new feature | `/implementation` |
|| Fix a bug | `/systematic-debugging` |
|| Add or update feature | `/enhancement` |
|| Review code changes | `/code-review-checklist` |
|| Run Rust tests | `/run-rust-tests` |
|| Build in release mode | `/build-release` |
|| Check Rust code quality | `/check-rust-quality` |
|| Architectural decision | `/architecture` |
|| Schema validation | `/wireframe-workflow` |
|| Quality checks | `/quality-checklist` |
|| Final verification | `/final-checks` |

## Subagent Profiles

Use these specialized subagents for targeted work:

- **backend-specialist** (sonnet): Rust modules, Python adapters, NATS messaging, API design
- **database-architect**: Database design, schema work, indexing
- **fast-researcher** (swe): Fast read-only codebase mapping
- **rust-researcher** (swe): Wireframe-AI Rust codebase research
- **security-auditor** (opus): Security review and vulnerability assessment
- **performance-optimizer**: Performance profiling and optimization
- **schema-validator**: Schema contract validation
- **test-runner**: Test execution and reporting

Invoke with agent selection or let `/project-routing` choose automatically.

## Skills

Use skills for detailed procedures. Invoke with `/skill-name` or let the agent choose.

**Core workflows:** `/karpathy-guidelines`, `/wireframe-workflow`, `/quality-checklist`, `/orchestration-patterns`, `/project-routing`, `/final-checks`, `/memory-discipline`, `/project-commands`

**Code quality:** `/code-review-checklist`, `/check-rust-quality`, `/code-fix`

**Testing & building:** `/run-rust-tests`, `/build-release`

**Development:** `/implementation`, `/architecture`, `/systematic-debugging`, `/enhancement`

**Language-specific:** `/rust-pro`, `/python-patterns`, `/async-tokio-patterns`

**Workflow:** `/git-commit` (auto-invoked by agents)

## Workflow Guidance

**Starting a new feature:**
1. Use `/karpathy-guidelines` to establish behavioral standards
2. Use `/project-routing` to identify the right approach
3. Use `/orchestration-patterns` for swarm research (1 worker at a time for comprehensive analysis)
4. Use `/architecture` for design decisions
5. Use `/implementation` for systematic development
6. Use `/quality-checklist` before committing
7. Use `/final-checks` before completion

**Debugging an issue:**
1. Use `/karpathy-guidelines` to establish behavioral standards (Think Before Coding, Goal-Driven Execution)
2. Use `/orchestration-patterns` for swarm investigation (1 worker for systematic investigation)
3. Use `/systematic-debugging` for 4-phase methodology
4. Use `/rust-pro` or `/python-patterns` for language-specific guidance
5. Use `/run-rust-tests` to verify fixes

**Code review:**
1. Use `/karpathy-guidelines` to review for surgical changes and simplicity
2. Use `/code-review-checklist` for staged changes
3. Use `/check-rust-quality` for linting
4. Use `/quality-checklist` for comprehensive review

**Performance optimization:**
1. Use `/karpathy-guidelines` to ensure goal-driven execution with verifiable criteria
2. Use `/orchestration-patterns` for swarm analysis (1 worker for bottleneck identification)
3. Use `/performance-profiling` for detailed profiling
4. Use `/rust-pro` for optimization guidance
5. Use `/run-rust-tests` to validate improvements

## Current Limitations

### Limited Debugging Skills
AI agents are not yet capable of deep debugging. For complex bugs:
- Ask for probable root causes rather than end-to-end fixes
- Use `/systematic-debugging` skill for 4-phase methodology
- Human oversight required for final root cause determination
- Once root cause is known, agents can implement the fix effectively

### Visual Reasoning
Agents have limited visual reasoning capabilities:
- Use design systems with reusable components
- Provide code from Figma rather than screenshots
- Avoid pixel-perfect matching requirements
- Describe visual requirements at the code level, not pixel level

### Knowledge Cutoffs
When working with new libraries or frameworks:
- Always point agents to latest documentation
- Don't assume agents know recent API changes
- Provide explicit links to current docs
- Verify patterns are current, not outdated

### Complex Multi-System Issues
For issues spanning multiple systems:
- Use swarm orchestration (4-6 workers) for parallel investigation
- Set explicit checkpoints between phases
- Expect multiple feedback cycles (80% time savings, not 100%)
- Human verification remains essential for final quality

## Time Management and Loss Minimization

Not all agent interactions will result in success. Learn to maximize successful outcomes while minimizing wasted time and tokens.

### Be Willing to Cut Losses Early

A common mistake is committing to making an interaction successful, even when the agent's work is veering off track. If you find yourself thinking:
- "It's ignoring my instructions"
- "This thing is going in circles"

**Stop and discontinue the conversation or manually take over.** Sending more messages is more likely a sign that the task complexity exceeds the agent's capabilities rather than a simple mistake that can be corrected.

### Diversify Your Experiments

If you're new to working with agents:
- Try a range of different prompts and ideas
- Double down on tasks agents naturally perform well on
- Cut losses on tasks they don't handle well
- Don't force agents to find success every time

### Start Fresh When Not Making Progress

Starting over is often the right answer with agents (more than with humans). If an agent is struggling to address feedback or correct course:
- Start fresh with a new agent
- Provide all instructions up front
- This often gets to success faster than trying to fix a messed-up environment

**Key insight:** An agent's ability to correct a messed-up environment is much worse than its ability to generate fresh code from scratch.

## Security and Permissioning

### Create Accounts for Your Agent

- Use throwaway emails for safe testing of sites
- Create custom IAM roles for cloud resource access
- Never use personal credentials for agent work

### Use Development/Staging Environments

- Agent should use the same testing setup as team engineers
- Avoid giving access to production services entirely
- For remote agents, run fully isolated test environments on the remote machine

### Use Read-Only API Keys

- Give readonly access where possible
- Humans should manually run scripts that interact with outside services
- Never give agents write access to critical production systems
