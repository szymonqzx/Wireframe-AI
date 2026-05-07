# Wireframe AI - Project Core

## Part 1: The Immutable Foundation

These are the only decisions that cannot be changed once any module is built against them. Get these right now. Everything else — frameworks, AI models, languages, strategies — is replaceable.

### 1.1 The message envelope

Every single event on the bus, from every module, uses this wrapper. It never changes. New fields may never be added to the root — only inside `payload`.

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

`schema_version` is the envelope version. `message_id` is this event's unique ID. `session_id` ties a conversation together across modules. `correlation_id` is how the Orchestrator matches `agent.result` events back to the `agent.job` that spawned them — critical for concurrent dispatch. `topic` is redundant with the NATS subject but makes payloads self-describing for logging and replay.

### 1.2 Topic naming convention

All topics follow `namespace.noun.verb` or `namespace.noun`, lowercase, dot-separated. No exceptions. New namespaces may be added freely. Existing topic names are immutable once any module ships against them.

### 1.3 Module identity protocol

On startup, every module publishes once to `sys.module.online`:

```json
{ "module_id": "my-memory-v2", "version": "1.0.0", "subscribes": ["task.submitted"], "publishes": ["task.enriched"] }
```

On shutdown (graceful), it publishes once to `sys.module.offline`. NATS heartbeat handles crash detection. No central registry. Any module that cares listens to these topics.

### 1.4 The agent job unit

This is what the Orchestrator fans out and what Reasoning Adapters consume. It must be fully self-contained — no adapter should need to query anything else to execute a job.

> **Note:** The schema below shows the core structure. The full schema in `schemas/v1/agent_job.json` includes additional fields like `output_format`, `top_p`, and detailed type definitions for all sub-structures.

```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "correlation_parent": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "task": {
    "user_input": "Write a Python script to sort CSV by column 2",
    "sub_task": null,
    "user_constraints": ["use Python 3"]
  },
  "context": {
    "memory_chunks": [],
    "session_history": [],
    "readonly_files": [],
    "safe_env": {},
    "working_dir": "/tmp",
    "max_context_tokens": 32768
  },
  "available_tool_capabilities": [
    { "name": "shell_exec", "input_schema": {}, "required_credentials": [], "rate_limit": null }
  ],
  "constraints": {
    "timeout_seconds": 300,
    "max_completion_tokens": 32768,
    "network_access": "outbound_only",
    "filesystem_policy": "sandbox_writable",
    "allow_subprocess": true
  },
  "model_config": { "provider": "openai", "model_name": "gpt-4o", "temperature": 0.7, "extra": {} },
  "metadata": { "submitter": "user", "priority": 1, "tags": {} },
  "schema_version": 1
}
```

### 1.5 SDK interface contract

Any module — Rust, Python, or anything else — must be wrappable in this three-method interface. The SDK enforces it. This is what makes third-party modules interchangeable.

```rust
trait Module {
    fn subscribes() -> &'static [&'static str];
    fn publishes() -> &'static [&'static str];
    async fn handle(&mut self, env: Envelope) -> Vec<Envelope>;
}
```

---

## Part 2: System Layers

Three layers. Layer 0 never changes. Layers 1 and 2 are infinitely replaceable.

**Layer 0 — Kernel.** What every user downloads. Three things: the NATS binary (downloaded separately from nats.io), the Interface module (the minimum I/O), and the SDK crate (the bridge that lets anyone write a module). No AI logic lives here. No opinions about memory, planning, or reasoning. Just the wire, a mouth, and the rules for talking on the wire.

**Layer 1 — Official modules.** Reference implementations you ship. Context, Orchestrator, Reasoning Adapter, Sandbox. Users download whichever ones they want. Any of them can be replaced by a community module that speaks the same topics.

**Layer 2 — Community modules.** Anything built by anyone using the SDK. Fully plug-and-play if it follows the topic schema. No approval process. No central registry. If it speaks the envelope format and subscribes to the right topic, it works.

![system_layers.svg](Wireframe%20AI%20-%20Project%20Core/system_layers.svg)

---

## Part 3: The SDK

The SDK is what makes Layer 2 real. Without it, writing a module means reimplementing NATS connection handling, envelope parsing, schema validation, heartbeat emission, and graceful shutdown every time. With it, a new module is a single struct and three methods.

### Rust crate — `agentic-sdk`

```toml
[dependencies]
agentic-sdk = { git = "https://github.com/you/agentic-core" }
```

What the SDK handles so module authors don't have to: NATS connection and reconnection, envelope serialization/deserialization, schema validation against the contract files, `sys.module.online` announcement on startup, `sys.module.offline` on shutdown, periodic heartbeat emission, error event publishing on malformed payloads, and queue group registration.

What a module author writes:

```rust
use agentic_sdk::{Envelope, Module};
use serde_json::Value;

struct MyMemoryModule;

#[agentic_sdk::module(
    subscribes = ["task.submitted"],
    publishes  = ["task.enriched"],
    queue_group = "task_handler"
)]
impl Module for MyMemoryModule {
    async fn handle(&mut self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let context = env.payload;
        vec![env.reply("task.enriched", serde_json::json!({ "context": context }))]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    MyMemoryModule.run("nats://localhost:4222").await
}
```

That is the entire module. The `#[module]` macro generates all the plumbing.

### Python package — `agentic-sdk-py`

```bash
pip install agentic-sdk
```

```python
from agentic_sdk import Module, Envelope

class MyReasoningAdapter(Module):
    subscribes   = ["agent.job"]
    publishes    = ["agent.result"]
    queue_group  = "agent_worker"

    async def handle(self, env: Envelope) -> list[Envelope]:
        result = await call_my_llm(env.payload)
        return [env.reply("agent.result", {"output": result})]

MyReasoningAdapter().run("nats://localhost:4222")
```

Any language with a NATS client library can implement the same pattern without the SDK — the SDK is convenience, not requirement. The contract is the envelope format and the topics, not the SDK itself.

---

## Part 4: The Topic Namespace API

This is the public contract. Topics are organized into namespaces. Adding a new namespace never breaks existing modules. Existing topic names within a namespace are immutable once shipped.

The second diagram shows the full namespace taxonomy — every domain that modules can plug into:

![topic_namespace_taxonomy.svg](Wireframe%20AI%20-%20Project%20Core/topic_namespace_taxonomy.svg)
