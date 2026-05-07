# Agent Job Schema

## Purpose

The `AgentJob` message is the unit of work dispatched to Reasoning Adapters. It must be **completely self-contained** — the adapter should be able to execute it without querying any external state or making assumptions about the system topology.

This is the contract that makes parallelism and adapter swaps possible.

## Schema

```rust
/// A self-contained unit of work sent to a reasoning adapter.
/// Contains everything needed to execute without external queries.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AgentJob {
    /// Unique identifier for this job instance
    pub job_id: String,

    /// Correlation ID inherited from the parent task (for result routing)
    /// The adapter MUST echo this back in AgentResult.correlation_parent
    pub correlation_parent: String,

    /// Human-readable task description
    pub task: TaskDescription,

    /// Context package — all memory, embeddings, history, file snippets needed
    pub context: ContextPackage,

    /// Tool availability — which MCP tools this adapter may call
    pub available_tool_capabilities: Vec<ToolCapability>,

    /// Constraints and limits
    pub constraints: ExecutionConstraints,

    /// Model selection and parameters
    pub model_config: ModelConfig,

    /// Metadata for observability and tracing
    pub metadata: JobMetadata,

    /// Schema version for forward compatibility
    #[serde(default = "current_schema_version")]
    pub schema_version: u32,
}
```

## Field Details

### `job_id`

Unique identifier for this specific `agent.job` message. Used for deduplication and tracing within the adapter's own logs. UUID v4.

### `correlation_parent`

The **root correlation ID** from the original user request. This string is copied from the `Envelope.correlation_id` field of the incoming `task.enriched` message.

**Why it matters:** The result will be posted to `agent.result` with its own `correlation_id` (job_id + suffix), but MUST carry `correlation_parent` so the Orchestrator can group N results back to one task.

### `task: TaskDescription`

```rust
pub struct TaskDescription {
    /// What the user asked for, plain text
    pub user_input: String,

    /// Optional structured sub-task definition (for multi-step plans)
    pub sub_task: Option<SubTask>,

    /// Expected output format (if any)
    pub output_format: Option<OutputFormat>,

    /// Any explicit constraints the user stated ("use Python", "avoid external APIs", etc.)
    pub user_constraints: Vec<String>,
}
```

### `context: ContextPackage`

All memory/context needed to complete the task. **The adapter must not reach outside this package.**

```rust
pub struct ContextPackage {
    /// Retrieved embeddings or memory chunks relevant to this task
    pub memory_chunks: Vec<MemoryChunk>,

    /// Full conversation history for this session (trimmed to token budget)
    pub session_history: Vec<ChatMessage>,

    /// Relevant file contents the adapter may read (no write access here)
    pub readonly_files: Vec<FileSnapshot>,

    /// Environment variables the adapter may use (sanitized, no secrets)
    pub safe_env: HashMap<String, String>,

    /// Working directory path (read-only view)
    pub working_dir: PathBuf,

    /// Token budget for this invocation
    pub max_context_tokens: usize,
}
```

**Rule:** The adapter receives a `ContextPackage` and returns a `ContextPackageDelta` outlining what it wrote to persistent storage (if anything). The Context module owns all persistent writes.

### `available_tool_capabilities: Vec<ToolCapability>`

List of MCP tools the adapter is allowed to call. Derived from the system's tool registry and the specific agent's allowed skills.

```rust
pub struct ToolCapability {
    /// MCP tool name (e.g., "shell_exec", "file_read")
    pub name: String,

    /// Parameter schema (JSON Schema)
    pub input_schema: serde_json::Value,

    /// Any required secrets or credential references (opaque to adapter)
    pub required_credentials: Vec<CredentialRef>,

    /// Rate limit / quota info (if applicable)
    pub rate_limit: Option<RateLimit>,
}
```

**Discovery:** The Sandbox module exposes its tools via MCP `initialize` handshake. The agent builder serializes that list into `AgentJob.available_tool_capabilities` so the adapter knows what it can call without runtime discovery.

### `constraints: ExecutionConstraints`

```rust
pub struct ExecutionConstraints {
    /// Maximum wall-clock time allowed (seconds)
    pub timeout_seconds: Option<u32>,

    /// Maximum tokens the adapter may emit (including tool calls)
    pub max_completion_tokens: Option<usize>,

    /// Network policy: "none" | "outbound_only" | "full"
    pub network_access: NetworkPolicy,

    /// Filesystem policy: "readonly" | "sandbox_writable" | "isolated_vm"
    pub filesystem_policy: FilesystemPolicy,

    /// Whether the adapter may spawn subprocesses (via MCP tools)
    pub allow_subprocess: bool,
}
```

Defaults:
- `timeout_seconds`: 300 (5 min)
- `max_completion_tokens`: 32_768
- `network_access`: `outbound_only` (outbound HTTP, no inbound listeners)
- `filesystem_policy`: `sandbox_writable` (confined to working_dir)
- `allow_subprocess`: true (via MCP shell tool, sandboxed)

### `model_config: ModelConfig`

```rust
pub struct ModelConfig {
    /// Which model backend to use: "openai", "anthropic", "local"
    pub provider: String,

    /// Model identifier (e.g., "gpt-4o", "claude-3-5-sonnet", "llama3:70b")
    pub model_name: String,

    /// Temperature / sampling parameters
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,

    /// Any provider-specific parameters (passed through)
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}
```

### `metadata: JobMetadata`

```rust
pub struct JobMetadata {
    /// Who submitted this (user ID, bot, system)
    pub submitter: String,

    /// Priority hint for queueing (0 = low, 1 = normal, 2 = high)
    #[serde(default = "default_priority")]
    pub priority: u8,

    /// Arbitrary key-value tags for observability
    #[serde(default)]
    pub tags: HashMap<String, String>,
}
```

### `schema_version`

Current version: **1**. Increment on breaking envelope changes (not payload changes).

## Variants by Adapter Type

The base `AgentJob` is universal. Specific reasoning adapters may extend it with an `adapter_hints: serde_json::Value` field that carries optional, adapter-specific configuration (e.g., LangChain chain config, LlamaIndex index IDs). The adapter ignores unknown fields.

## Serialization Format

All messages use **JSON** (for now;MessagePack possible later). Envelope + payload is one atomic JSON document.

```json
{
  "message_id": "uuid4",
  "session_id": "session_uuid",
  "correlation_id": "uuid4",
  "topic": "agent.job",
  "schema_version": 1,
  "timestamp": 1714880000,
  "payload": {
    "job_id": "uuid4",
    "correlation_parent": "uuid4",
    "task": { "user_input": "...", ... },
    "context": { ... },
    "available_tool_capabilities": [ ... ],
    "constraints": { ... },
    "model_config": { ... },
    "metadata": { ... },
    "schema_version": 1
  }
}
```

## Validation Rules

Receivers MUST:
1. Check `schema_version` — reject if > current version (unknown future)
2. Verify `topic` matches expected subscription pattern
3. Validate `correlation_parent` is well-formed UUID
4. Ensure `available_tool_capabilities` is non-empty (adapter cannot function without tools)
5. Reject messages with `timestamp` > now + 60s (future-dated, likely error)

## Versioning Policy

- Adding fields to payload structs: backward-compatible (ignore unknown)
- Removing fields: deprecate first, then remove in next major version bump
- Changing field types: breaking — increment `schema_version`
- Envelope changes: increment `schema_version`

## Next

Define the `AgentResult` schema (what the adapter sends back).
