# Wireframe AI - Project Architecture

## 1. The decisions that actually matter

Before anything else, these are the choices that are genuinely painful to refactor once the system is running. Get these right, stay flexible on everything else.

### The message envelope

Every event on the bus carries the same wrapper fields. If you change this structure later, every module breaks simultaneously. Keep it minimal — a session ID, a correlation ID for tracking agent jobs back to their parent task, a topic name, and a payload. Nothing else.

### State ownership

Exactly one module owns persistent session state. If two modules write to memory independently, you'll have split-brain bugs that are nearly impossible to debug. The Context module owns all of it. Every other module borrows state per-call, never stores it.

### The agent job unit

When the Orchestrator fans work out to multiple agents, each job needs to be a completely self-contained unit — it must carry everything the reasoning adapter needs to run without querying anything else. This is what makes parallelism clean and makes adapter swaps safe. Define this format now.

### Module discovery via queue groups

NATS queue groups are how optional modules work without any configuration or registry. Multiple subscribers in the same queue group compete for messages; NATS routes each message to exactly one. This is how the Orchestrator can be present or absent with no code changes anywhere else.

### Tool interface standard

MCP (Model Context Protocol) for how reasoning adapters discover and call tools in the sandbox. Choosing this now means any future reasoning engine that speaks MCP works immediately.

---

## 2. Stack

| Layer | Choice | Why |
| --- | --- | --- |
| Core modules | Rust + Tokio | Compiled single binaries, cross-platform via `cargo build --target`, async concurrency for free |
| AI/ML adapters | Python (isolated) | The ML ecosystem lives here; Rust can't replace it and shouldn't try |
| Message bus | NATS | Single binary ~10MB, Rust-native via `async-nats`, queue groups handle optional modules natively |
| Serialization | `serde_json` | Zero-cost, battle-tested, language-agnostic on the wire |
| Tool interface | MCP | Standardizes tool discovery between reasoning engines and sandbox |
| Cross-platform builds | `cargo` + `cross` | One command, any target: Windows, macOS, Linux, ARM |

The Python adapter is the only place heavy dependencies live. Everything else — Tokio, async-nats, serde — adds a few hundred KB to compiled binaries.

---

## 3. Message envelope

This is the format every single event on the bus uses, regardless of topic. Define it once, never touch it again.

```json
{
  "message_id": "550e8400-e29b-41d4-a716-446655440000",
  "session_id": "session_e29b41d4-a716-446655440000",
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
  "topic": "agent.job",
  "schema_version": 1,
  "timestamp": 1714820000,
  "payload": {}
}
```

`correlation_id` is the critical field for the orchestrator — it links an `agent.result` back to the `agent.job` that produced it, and links both back to the original `task.enriched` that spawned them. Without this, concurrent agent results are unroutable.

---

## 4. Module catalogue

### Interface module — Rust

Publishes: `task.submitted`
Subscribes: `task.complete`

CLI, chat, or future GUI. Wraps user input in the envelope and waits for a result. Has no knowledge of what processes it. Swappable with a web server, a VS Code extension, anything.

---

### Context module — Rust

Publishes: `task.enriched`
Subscribes: `task.submitted`

The only module that reads and writes persistent memory. Pulls relevant embeddings or history for the session, attaches them to the task payload, and forwards the enriched version. If you later swap from SQLite embeddings to a graph database, only this module changes.

---

### Orchestrator module — Rust, optional

Publishes: `agent.job` (N times, concurrently)
Subscribes: `task.enriched` (queue group: `task_handler`)

Receives an enriched task, breaks it into a plan of N independent sub-jobs, and fires all of them onto `agent.job` simultaneously using `tokio::spawn`. It then listens on `agent.result` for completions, collecting results by `correlation_id` until all N are in, then synthesizes and publishes `task.complete`.

When the Orchestrator is absent, the Reasoning Adapter joins the same `task_handler` queue group and handles `task.enriched` directly as a single job.

---

### Reasoning adapter — Rust, N instances

Publishes: `agent.result`
Subscribes: `agent.job` (queue group: `agent_worker`)

The Rust reasoning adapter uses the Provider trait for LLM communication. It supports multiple providers (OpenAI, Anthropic, local models) through a unified interface. The queue group means you can run two or three instances in parallel and NATS load-balances jobs across them automatically.

**Provider System:** The adapter uses a provider-core crate that defines the Provider trait, with implementations for different LLM providers. Providers support capability negotiation, session management, and streaming responses. New providers can be added by implementing the Provider trait.

Stateless per invocation — all context comes in via the `agent.job` payload. Session management is handled internally by the SessionManager for multi-turn conversations.

---

### Sandbox module — Rust, MCP server

Runs generated code in an isolated environment. Exposes its capabilities (file_read, file_write, shell_exec, file_list) as MCP tools over **stdio**. The Reasoning Adapter spawns the sandbox as a subprocess and communicates via JSON-RPC.

Instead of publishing to NATS topics, the sandbox is discovered and called through MCP's tool discovery protocol:

```
Adapter → spawns sandbox process → MCP initialize handshake
       → tools/call (file_read, file_write, shell_exec, file_list)
       → MCP response with results
```

Any future reasoning engine that speaks MCP gets sandbox access for free.

---

## 5. How the Orchestrator dispatches concurrent agents

The Orchestrator's job is simple: receive one task, emit N agent jobs in parallel, collect N results.

In Rust with Tokio, the fan-out looks like this conceptually:

```rust
// On receiving task.enriched:
let jobs = planner.decompose(&task);      // break task into N sub-jobs
let handles: Vec<_> = jobs
    .into_iter()
    .map(|job| {
        let nc = nats_client.clone();
        tokio::spawn(async move {
            nc.publish("agent.job", job.to_envelope()).await
        })
    })
    .collect();

// Wait for all dispatches to complete
for h in handles { h.await.unwrap(); }

// Then collect results via correlation_id tracking
// until all N agent.result messages arrive
```

Each `agent.job` carries the full context it needs. The Reasoning Adapter instances pick up jobs competitively from the queue group — you can run one adapter or five and the system balances automatically.

> **Note:** The current orchestrator creates N identical copies of the input task (configurable via `CONCURRENCY_N`). A `planner.decompose()` method is stubbed — future work will break tasks into varied sub-jobs.

**Without the Orchestrator:** the Reasoning Adapter joins the `task_handler` queue group and treats each `task.enriched` as a single-job task. No config changes anywhere.

---

## 6. Adding a new module

This is the plug-and-play story. Say a new memory technique drops — graph-based RAG, for example — and you want to try it:

1. Create a new Rust (or Python) process.
2. Subscribe to `task.submitted`, publish to `task.enriched`, join queue group `task_handler`.
3. Run it alongside or instead of the old Context module.
4. Done.

No other module changes. No config file updates (NATS handles routing). The old module can be killed mid-run and the new one takes over.

The same pattern applies to a new LLM: write a new Python adapter, point it at `agent.job`, join the `agent_worker` queue group. You can even run old and new adapters simultaneously for A/B testing.

---

## 7. Repo structure

Flat, buildable from root, no ceremony.

```
Wireframe-AI/
│
├── kernel/                    ← what the end user always downloads
│   ├── nats/                  (the bus binary — downloaded separately)
│   └── interface/             (the only truly required Rust module)
│
├── modules/                   ← official optional modules, mix and match
│   ├── context/
│   ├── orchestrator/
│   └── sandbox/
│
├── sdk/                       ← what third-party module authors import
│   ├── agentic-sdk/           (Rust crate: envelope, NATS connect, heartbeat)
│   ├── agentic-sdk-macros/    (Rust proc-macro: #[module] attribute)
│   └── agentic-sdk-py/        (Python package: same for adapter authors)
│
├── adapter/                   ← reference reasoning adapters (any language)
│   ├── rust/                  (Rust adapter with Provider trait)
│   └── python/                (Python reference implementation - legacy)
├── provider-core/             ← Provider trait and core infrastructure
├── providers/                 ← LLM provider implementations
│   ├── openai/                (OpenAI-compatible HTTP provider)
│   ├── anthropic/             (Anthropic HTTP provider - future)
│   └── local/                 (Local model stdio provider - future)
│
├── schemas/                   ← the public API, versioned and documented
│   └── v1/
│       ├── TOPICS.md           (every topic, what it carries, who uses it)
│       ├── envelope.json
│       ├── task_submitted.json
│       ├── task_enriched.json
│       ├── task_complete.json
│       ├── agent_job.json
│       └── agent_result.json
│
├── examples/                  ← example modules for developers
│   └── ping-module/
│
├── tests/                     ← integration and benchmark tests
│   ├── integration_test.rs
│   ├── benchmark_test.rs
│   └── test_python_sdk.py
│
└── scripts/                   ← developer tooling
    ├── build-release.ps1
    ├── cross-build.ps1
    ├── download-nats.ps1
    ├── run-demo.ps1
    ├── smoke-test.ps1
    ├── start-all.ps1
    └── start-all.sh
```

`cargo build --release` in the root directory produces standalone binaries for each module. Each one is the entire module — no runtime, no dependencies, drag it to any platform and run it.

---

## 8. Cross-platform

Rust's `cargo` compiles to any target from any machine. For binaries you want to ship:

```bash
# macOS and Linux from the same machine
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target aarch64-apple-darwin

# Or use 'cross' for Docker-based cross-compilation
cross build --release --target x86_64-pc-windows-gnu
```

NATS is a single Go binary with pre-built releases for every platform — download it from nats.io and run it (not bundled in this repo). Use `scripts/download-nats.ps1` to fetch it automatically. The Python adapter is the only piece that needs a Python environment, which is expected.

Now the two diagrams. First, the full system structure:

![system_overview.svg](Wireframe%20AI%20-%20Project%20Architecture/system_overview.svg)

The second diagram shows what actually happens inside the Orchestrator when it receives a multi-step task — how it dispatches simultaneous agents and collects results:

![orchestrator_fanout.svg](Wireframe%20AI%20-%20Project%20Architecture/orchestrator_fanout.svg)

The note at the bottom of that diagram is the key design win: the path from `agent.result` to `task.complete` is identical whether you have one agent or five. The Orchestrator is an optional amplifier in the middle, not a load-bearing pillar.

**A few things to decide before you write a single line of code:**

The `agent_job` schema is the one you'll most regret getting wrong. It needs to carry everything a reasoning adapter needs in isolation — the enriched context, the specific sub-task, tool availability, and the correlation ID. Sketch that struct in Rust first and validate it against a few real agentic tasks you imagine yourself running. The envelope and the agent job together are the hardest refactors once you have working adapters built around them.

[Project Implementation Plan](Wireframe%20AI%20-%20Project%20Architecture/Project%20Implementation%20Plan%2057376e14bad24f1c8552c10d9234a362.csv)
