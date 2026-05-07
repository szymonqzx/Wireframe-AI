---
name: parallel-search
description: Fast context retrieval using parallel tool calls for codebase mapping and pattern detection
allowed-tools:
  - read
  - grep
  - glob
triggers:
  - model
---

# Parallel Search Skill

**Purpose:** Fast context retrieval using parallel tool calls, inspired by SWE-grep principles.
**Pattern:** 8 parallel tool calls per turn, maximum 4 serial turns.
**Use Cases:** Codebase mapping, pattern detection, dependency analysis, fast research.

---

## When to Use

Use this skill when you need to:
- Quickly map codebase structure
- Find patterns across multiple files
- Analyze dependencies between modules
- Perform fast research before implementation
- Understand architecture before making changes

**Do NOT use for:**
- Simple single-file searches (use grep directly)
- Sequential analysis tasks
- Tasks requiring deep semantic understanding (use Tree-sitter or CodeQL)

---

## Core Principles

### SWE-grep Inspired Pattern

Based on Cognition's SWE-grep research:
- **8 parallel tool calls** per turn (maximize parallelism)
- **4 serial turns** maximum (minimize latency)
- **Restricted tool set** (grep, glob, read for cross-platform compatibility)
- **Fast tool execution** (optimized for speed)

### Parallel Search Strategy

```powershell
# Turn 1: 8 parallel searches
$parallelSearches = @(
    "rg --json 'pattern1' --type rust",
    "rg --json 'pattern2' --type rust",
    "rg --json 'pattern3' --type python",
    "rg --json 'pattern4' --type toml",
    "find . -name '*.rs'",
    "find . -name '*.py'",
    "find . -name 'Cargo.toml'",
    "find . -name 'requirements.txt'"
)

# Execute all in parallel
$parallelSearches | ForEach-Object { Start-Process -NoNewWindow $_ }

# Turn 2: Analyze results, spawn 8 more targeted searches
# Turn 3: Deep dive into specific files
# Turn 4: Final synthesis
```

---

## Tool Set

### Primary Tools

1. **ripgrep (rg)** - Fast text search with regex
   - Use `--json` output for programmatic parsing
   - Use `--type` for language-specific searches
   - Use `--glob` for file pattern filtering

2. **find/glob** - File pattern matching
   - Use for finding specific file types
   - Use for directory structure analysis
   - Use for configuration file discovery

3. **read** - File content reading
   - Use for reading specific files identified by search
   - Batch read multiple files in parallel
   - Use line ranges for targeted reading

### Tool Usage Patterns

```rust
// Pattern 1: Language-specific search
rg --json "async_nats" --type rust

// Pattern 2: File pattern search
find . -name "*.rs" -o -name "*.py"

// Pattern 3: Glob pattern search
glob "**/src/main.rs"

// Pattern 4: Parallel file reads
read("kernel/interface/src/main.rs")
read("modules/orchestrator/src/main.rs")
read("modules/context/src/main.rs")
read("adapter/rust/src/main.rs")
```

---

## Search Patterns

### Wireframe-AI Specific Patterns

#### NATS Usage Pattern
```bash
# Search for NATS usage across codebase
rg --json "nats::" --type rust &
rg --json "async_nats" --type rust &
rg --json "publish\|subscribe" --type rust &
rg --json "JetStream" --type rust &
wait
```

#### Module Pattern
```bash
# Search for module definitions
rg --json "mod\s+\w+" --type rust &
rg --json "#\[module\]" --type rust &
rg --json "announce_online" --type rust &
rg --json "announce_offline" --type rust &
wait
```

#### Provider Pattern
```bash
# Search for Provider trait usage
rg --json "Provider" --type rust &
rg --json "provider_core" --type rust &
rg --json "complete\|describe\|status" --type rust &
rg --json "EventStream" --type rust &
wait
```

#### Envelope Pattern
```bash
# Search for Envelope usage
rg --json "Envelope<" --type rust &
rg --json "envelope::" --type rust &
rg --json "message_id\|session_id\|correlation_id" --type rust &
rg --json "schema_version" --type rust &
wait
```

#### Schema Pattern
```bash
# Search for schema references
rg --json "schema" --type rust &
rg --json "validate" --type rust &
rg --json "jsonschema" --type rust &
rg --json "JSONSchema" --type rust &
wait
```

---

## Workflow

### Turn 1: Broad Discovery (8 parallel searches)

**Goal:** Map codebase structure and identify key patterns

**Searches:**
1. `rg --json "mod\s+\w+" --type rust` - Module structure
2. `rg --json "nats::" --type rust` - NATS usage
3. `rg --json "Provider" --type rust` - Provider system
4. `rg --json "Envelope<" --type rust` - Envelope usage
5. `find . -name "*.rs"` - Rust files
6. `find . -name "*.py"` - Python files
7. `find . -name "Cargo.toml"` - Rust projects
8. `find . -name "requirements.txt"` - Python dependencies

**Output:** File lists, pattern counts, file locations

### Turn 2: Targeted Analysis (8 parallel searches)

**Goal:** Deep dive into specific areas identified in Turn 1

**Searches:**
1. `rg --json "async_nats::connect" --type rust` - NATS connections
2. `rg --json "publish\(" --type rust` - Message publishing
3. `rg --json "subscribe\(" --type rust` - Message subscriptions
4. `rg --json "impl Provider" --type rust` - Provider implementations
5. `rg --json "fn complete" --type rust` - Provider methods
6. `rg --json "Envelope::new" --type rust` - Envelope creation
7. `rg --json "child\|reply" --type rust` - Envelope relationships
8. `rg --json "schema_version" --type rust` - Schema usage

**Output:** Specific usage patterns, function signatures, call sites

### Turn 3: File Reading (8 parallel reads)

**Goal:** Read key files identified in Turn 2

**Reads:**
1. `kernel/interface/src/main.rs` - Interface module
2. `modules/orchestrator/src/main.rs` - Orchestrator module
3. `modules/context/src/main.rs` - Context module
4. `provider-core/src/lib.rs` - Provider trait
5. `sdk/agentic-sdk/src/envelope.rs` - Envelope system
6. `adapter/rust/src/main.rs` - Rust adapter
7. `adapter/python/src/adapter.py` - Python adapter
8. `config/src/lib.rs` - Configuration

**Output:** File contents, implementation details

### Turn 4: Synthesis (Final analysis)

**Goal:** Synthesize findings into coherent understanding

**Analysis:**
- Map module dependencies
- Identify message flow patterns
- Document Provider implementations
- List Envelope usage patterns
- Identify schema validation points

**Output:** Comprehensive codebase understanding report

## Additional Resources

For performance optimization and integration with other skills, see `@[skills/parallel-search/SKILL-advanced.md]`.

For detailed examples, metrics, troubleshooting, and production considerations, see `@[skills/parallel-search/SKILL-examples.md]`.