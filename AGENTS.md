# AGENTS.md

**Purpose:** Wireframe-AI project constitution. Strict structural boundaries, exact command references, explicit anti-feature-creep rules.

## Core Principles

**Quality Over Speed:** Ship small, correct, reviewed changes. Never take shortcuts that compromise codebase health.

**Evidence-Based:** Prefer evidence over vibes. Every claim should be backed by data, tests, or citations.

**Structural Separation:** Structural changes ≠ behavioral changes (never mix in same commit). Commit structural first, then behavioral.

**Test-Driven Development:** TDD mandatory: Red-Green-Refactor cycle. Write failing test first, minimal code to pass, then refactor.

**Clean Code First:** Tidy before adding features. Remove dead code, extract magic numbers, split long functions, remove duplication.

**Simplicity First:** Choose the simplest viable solution. No speculative features, no over-abstraction.

**Readability Priority:** Code must be immediately understandable. Clear naming, appropriate abstraction level.

**Dependency Minimalism:** No new libraries without justification. Every dependency must have a clear purpose.

**Security First:** All external data validated, no secrets in code, proper authentication/authorization.

**Token Efficiency:** Optimize context window usage. Be concise, avoid unnecessary repetition.

## Default Behaviors

- Read existing code before editing
- Prefer minimal diffs over rewrites
- Never invent APIs, commands, benchmarks, or pricing
- Cite files and commands when reporting
- Run the narrowest relevant check before declaring done

## Project Vault

- All architectural decisions live in `vault/decisions/ADR-NNN.md`
- Subsystem maps live in `vault/mocs/<name>.md` — start here for any cross-cutting task
- When making a big decision, write an ADR *first*, then implement

## Context Engineering (The Four Moves)

**Offload** — Push rarely-needed info to disk/DB: plans, AGENTS.md, skills, memories.

**Retrieve** — Pull right context at right time: @mention, MCP tools, @web/@docs.

**Compress** — Summarize/evict stale turns: prompt cache timer, megaplan → fresh session.

**Isolate** — Prevent context bleed: worktrees, spaces, subagents.

**Diagnose slow sessions:** Carrying dead context or missing needed context?

## TDD & Tidy First (Kent Beck Principles)

### Red-Green-Refactor Cycle (Mandatory)

**Red:** Write a failing test that demonstrates the desired behavior
- Test must fail before implementation
- Test must be specific to the behavior being added
- Use descriptive test names that document intent

**Green:** Write the minimal code to make the test pass
- No more code than necessary
- No speculative features
- Test passes = implementation complete

**Refactor:** Improve code structure without changing behavior
- Run tests before and after
- No behavioral changes allowed in refactor commits
- Separate structural refactors from behavioral changes

### Structural vs Behavioral Changes (Strict Separation)

**Structural Changes** (rearranging code without changing behavior):
- Extract functions/modules
- Rename identifiers
- Move code between files
- Change internal data structures
- Commit message: "refactor: <description>"

**Behavioral Changes** (adding new features):
- Add new functions
- Change function signatures
- Modify algorithm logic
- Add new dependencies
- Commit message: "feat: <description>"

**Rule:** Never mix structural and behavioral changes in the same commit. If you need both, commit structural first, then behavioral.

### Tidy First (Clean Code Before Features)

Before adding any new feature:
1. Identify the code area the feature will touch
2. Run existing tests - they must pass
3. If code is messy, tidy it first (separate commit)
4. Only then add the new feature (separate commit)
5. Verify all tests pass

**Tidy checklist:**
- Remove dead code and commented sections
- Extract magic numbers to named constants
- Split long functions (>50 lines)
- Remove duplication
- Improve naming for clarity
- Add missing documentation

## Self-Improving Micro-Learning (Mandatory)

**Hit same error twice?** Stop and write a memory capturing: trigger, failure, fix. Future sessions must read and avoid documented failure modes.

## Error Resolution Protocol (Mandatory)

### Problem Identification Phase (Required Before Any Fix)

When encountering an error, you MUST complete this phase before writing code:

1. **Parse the error completely:**
   - Extract error type, message, stack trace
   - Identify the exact line and file
   - Note error code if present

2. **Map data flow paths:**
   - Trace execution path to error point
   - Identify all inputs and outputs
   - Map state transformations
   - Identify where data diverges from expected

3. **Isolate the exact error source:**
   - Determine if error is in: logic, data, configuration, environment, dependency
   - Identify the specific component responsible
   - Check if error is reproducible
   - Check if error is intermittent

4. **Generate hypothesis:**
   - Formulate specific root cause hypothesis
   - Identify supporting evidence
   - Identify contradictory evidence
   - Rank hypotheses by likelihood

**Only after completing Problem Identification:** Propose and implement a fix.

### Error Recurrence Protocol

If the same error occurs more than once:
1. Stop guessing solutions
2. Perform deep analysis of the pattern
3. Check for systemic issues (configuration, environment, dependencies)
4. Add permanent guardrails (tests, validation, monitoring)
5. Document in MEMORY.md for future reference

## Execution Rules (Prevent Hangs)

- If a command takes more than 10 seconds with no output, kill it and try a different approach
- If you've been on the same step for more than 60 seconds, stop and explain what's blocking you
- Never retry the exact same command that just failed. Diagnose first

## Token Limit Warning

Cascade has a soft cap around 6k tokens on the combined rules context per session. Past it, the tail gets silently truncated. If AGENTS.md + Memories + active Rules push over this, pull rarely-used sections into vault/ or skills and @mention them on demand.

## Instruction File Hierarchy

- **AGENTS.md** - Directory-scoped rules, committed to repo (what not to do)
- **CLAUDE.md** - Anthropic/Claude Code convention (Cascade reads as fallback)
- **.windsurfrules** - Legacy Windsurf project rules (still supported)
- **SKILL.md** - Per-skill instructions + bundled resources (how to do it)
- **MEMORY.md** - Long-running learned facts (what we already decided)
- **CONTEXT.md** - Current-session snapshot (where we are now)

**Rule of thumb:** AGENTS.md is the what not to do (rules), SKILL.md is the how to do it (procedures), MEMORY.md is what we already decided (facts), CONTEXT.md is where we are now (state). Cascade's built-in Memories & Rules handles personal preferences that shouldn't be in the repo at all.

## Cross-Platform Compatibility

This AGENTS.md follows the universal AGENTS.md standard (backed by OpenAI, Google, Anthropic, Cursor, Sourcegraph) for cross-platform AI-assisted development compatibility.

Compatible with:
- Windsurf (native via AGENTS.md)
- Cursor (via symlink to .cursorrules)
- Claude Code (via symlink to CLAUDE.md)
- GitHub Copilot (reads AGENTS.md natively)
- Aider, Zed (native AGENTS.md support)

**One-shot cross-agent compatibility:** Symlink a single file so every agent reads the same rules:
```bash
ln -s AGENTS.md GEMINI.md  # Example
```

## Skills System

Windsurf auto-discovers skills from multiple locations:
- `.windsurf/skills/` - Project-specific skills
- `~/.codeium/windsurf/skills/` - Global skills
- Via `gh skill install` - GitHub CLI skill installation

### Skills Ecosystem — gh skill, agentskills.io, and Viral Skills

The agent-skills ecosystem is suddenly one of the most active parts of the AI-tools landscape. As of mid-April 2026:

**agentskills.io** is the spec for SKILL.md — 30+ platforms (Claude Code, Cursor, Codex, Gemini CLI, Copilot, OpenClaw, Hermes, Windsurf) follow it.

**gh skill CLI** — landed April 16. Install, update, publish, pin, verify provenance of any SKILL.md-formatted skill from GitHub directly into any supported agent, including Cascade.

**mvanhorn/last30days-skill** — #1 on GitHub Trending April 16, 22k stars, 1.8k forks. The canonical example of a "does one thing extremely well" skill.

**jezweb/skills** — a curated list of 300+ community skills that install with gh skill install.

#### Installing Skills with the New gh skill CLI

```bash
# One-time install of the CLI extension
gh extension install github/gh-skill

# Search for skills
gh skill search research multi-source

# Install into Cascade (target flag writes to .windsurf/skills/)
gh skill install mvanhorn/last30days-skill --target windsurf

# Pin a version (skills are content-addressed, so this is immutable)
gh skill install mvanhorn/last30days-skill@v3.0.5 --target windsurf

# Check for updates
gh skill outdated --target windsurf

# Publish your own
gh skill publish .windsurf/skills/deep-recall
```

Target flags the CLI currently supports: --target claude-code, --target cursor, --target codex, --target copilot, --target gemini-cli, --target windsurf, --target openclaw, --target hermes. One skill repo, N targets.

Under the hood, --target windsurf just copies the SKILL.md (and any bundled resources/, scripts/, etc.) into .windsurf/skills/<name>/ and sets the right permissions — exactly what the basic skills system describes, but with provenance + version pinning.

#### Installing last30days in Cascade (without the CLI)

If you don't want the GitHub CLI dependency:

```bash
mkdir -p .windsurf/skills
git clone --depth 1 https://github.com/mvanhorn/last30days-skill /tmp/l30d
cp -r /tmp/l30d/skills/last30days .windsurf/skills/
```

Then in Cascade: `/last30days windsurf 2.0` — synthesizes a brief across Reddit, X, YouTube, HN, Polymarket, GitHub, Bluesky, Perplexity, and five others.

Why this matters: last30days does exactly what multi-platform research does — but on demand, for any topic. Install it once and you have an always-current research tool without standing up your own MCP server.

#### Writing Skills Worth Publishing

The viral skills all follow the same shape:

- **Single-purpose.** last30days researches across platforms. pr-review reviews PRs. Not "research and review and deploy."
- **Description is a trigger.** The description: YAML field is the only thing Cascade sees when deciding whether to invoke. Write it as a condition: "Use when the user asks about..."
- **Progressive disclosure.** The skill body is short; details live in resources/ files the agent reads on demand.
- **Bundled scripts.** If there's deterministic logic (rate limit, dedup, format), write it in a script — don't make the LLM redo it.
- **Zero-config if possible.** Work immediately with sensible defaults; require secrets only if invoked for a gated source.
- **Cross-target.** Test on at least Claude Code + Cursor + Windsurf before publishing. A SKILL.md that assumes a specific slash-command runner won't travel.

#### Skills Worth Installing Today

| Skill | What it does | Why |
|-------|--------------|-----|
| last30days | Parallel multi-platform research + AI synthesis | Trending #1 for a reason |
| commit-surgeon | Rewrites git history, splits messy commits, interactive rebase driver | Saves you from git rebase -i hell |
| pr-surgeon | Turn a local branch into a clean PR (squash, rename, describe) | Use after any non-trivial Cascade session |
| test-backfill | Adds missing tests for uncovered lines, aware of your test framework | Pairs with the tester subagent |
| docs-writer | README / AGENTS.md / CHANGELOG keeper | Pairs with post_cascade_response hook |
| changelog-bot | Keeps CHANGELOG.md in conventional-commits format | Release hygiene |
| secret-scrubber | Scans diffs for secrets before commit | Complements the hook in §8 |

## Claude Code Subagents Pattern

Cascade emulates Claude Code's subagent pattern via `.windsurf/agents/` + worktrees. Each subagent has:
- Isolated context windows
- Domain-specific intelligence
- Tool permission controls
- Model routing (opus/sonnet/haiku)

## Hooks Configuration

Verify `.windsurf/hooks.json` matches the official format:
- Nested under "hooks", each event is an array
- Use "command" key on Unix, "powershell" on Windows for cross-compatibility
- Test hooks manually: `echo '{"file_path":"test.py"}' | python .windsurf/hooks/secret_scan.py`
- Exit codes: 0 = allow, 2 = block
- Hook merge order: workspace overrides user overrides system

## MCP Servers

Configured MCP servers for this project:
- github - Full GitHub API: PRs, issues, code, actions
- filesystem - Scoped file access
- fetch - Read arbitrary URLs
- sqlite - Local SQLite for vault/ and data exploration
- memory - Persistent key-value store
- chrome-devtools - Screenshots, console, DOM, perf traces
- playwright - Browser automation for E2E
- deepwiki - AI-powered documentation for GitHub repos
- jules - Google Jules agent integration

## Starter Templates & Configs

This project uses:
- windsurf-unlocked/starter - 8 subagents, hooks, skills, vault, Spec Kit workflows, MCP config
- karpathy's LLM Wiki gist - The vault/-style agentic-wiki pattern
- agentskills.io - SKILL.md specification + examples

## Karpathy Guidelines (Behavioral Standards)

Derived from Andrej Karpathy's observations on LLM coding pitfalls. Use `/karpathy-guidelines` skill for detailed guidance.

**Four Core Principles:**

1. **Think Before Coding** - State assumptions explicitly, present multiple interpretations, push back when simpler approaches exist, stop when confused
2. **Surgical Changes** - Touch only what you must, clean up only your own mess, match existing style
3. **Goal-Driven Execution** - Define success criteria, transform tasks into verifiable goals, loop until verified

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

**Account Creation:**
- Use throwaway emails for safe testing
- Create custom IAM roles for cloud resource access
- Never use personal credentials for agent work

**Environment Isolation:**
- Use development/staging environments only
- Avoid production access entirely
- Run fully isolated test environments for remote agents

**Access Control:**
- Prefer read-only API keys
- Humans manually run scripts that interact with outside services
- Never give agents write access to critical production systems

---

## Vault Protocol

This repo maintains a compounding `vault/` of project knowledge. The vault is the first place you look and the last place you write.

#### At Session Start

1. Read `vault/INDEX.md` to see what's documented.
2. If the task touches a topic with an existing vault page, read it BEFORE reading source code.
3. If the task touches a topic WITHOUT a vault page, note it — you'll write one at the end.

#### During the Session

- When you encounter something non-obvious (a gotcha, a decision, a pattern):
  - Check if `vault/` has a page for it. If yes, do not re-discover it.
  - If no, note it for end-of-session capture.
- Never cite internal knowledge the user can't verify. If it's not in `vault/`, don't claim it as fact.

#### At Session End

When the session produced a decision, solved a non-obvious problem, or established a pattern:

1. Write a vault page. Location rules (match the directories that exist in this repo's `vault/`):
   - Decisions (why we chose X over Y) → `vault/decisions/ADR-NNN-<topic>.md` (next sequential number)
   - Postmortems → `vault/incidents/<YYYY-MM-DD>-<topic>.md`
   - Service/component notes, runbooks, gotchas → `vault/services/<name>.md`
   - Ownership → `vault/people/<name>.md`
   - Cross-cutting topic hubs → `vault/moc/<topic>.md` (a "Map of Content" that links related pages)
   - Glossary entries → append to `vault/glossary.md`
2. Update `vault/INDEX.md` to link the new page.
3. One page per topic. If a page grows > 300 lines, split it (same threshold as `vault/INDEX.md`).

#### Writing Style

- Plain markdown. No front-matter unless required by another tool.
- Lead with the **bottom line**: the answer in the first 2 lines, details below.
- Use the present tense for current state ("we use Redis") and past tense for history ("we tried DynamoDB, it didn't fit").
- Include dates on decisions — these age, and agents need to know how fresh a claim is.
- No prose walls. Headers, lists, tables, code blocks.

#### Never

- Put secrets in `vault/`. It's git-tracked — assume public.
- Put PII in `vault/`. Same reason.
- Write vault pages that duplicate code comments. The vault is for things code can't express.
- Leave `vault/INDEX.md` out of date. The index is the API.

#### Skills Involved

- `wiki-query` — called first, reads relevant vault pages for the current intent.
- `wiki-update` — called last, writes new or updated pages before session ends.

## Anti-Patterns to Kill

**Re-explaining context** → AGENTS.md + Rules, load once.

**@mention-ing giant files** → @mention directory, let it search.

**Pasting terminal output** → Use `cascade > log 2>&1`.

**Long sessions** → Plan → close → reopen with plan.

**One model for everything** → Subagents with pinned models.

## Skills Ecosystem

**agentskills.io** — SKILL.md spec for 30+ platforms (Claude Code, Cursor, Codex, Gemini CLI, Copilot, OpenClaw, Hermes, Windsurf).

**gh skill CLI** — Install/update/publish/pin skills from GitHub: `gh extension install github/gh-skill`, `gh skill install <repo> --target windsurf`, `gh skill outdated --target windsurf`.

**Without CLI:** `mkdir -p .windsurf/skills && git clone <repo> && cp -r skills/<name> .windsurf/skills/`.

**Viral skills:** Single-purpose, description as trigger, progressive disclosure, bundled scripts, zero-config, cross-target.

**Recommended:** last30days (research), commit-surgeon (git history), pr-surgeon (PR cleanup), test-backfill (tests), docs-writer (docs), changelog-bot (changelog), secret-scrubber (secrets).
