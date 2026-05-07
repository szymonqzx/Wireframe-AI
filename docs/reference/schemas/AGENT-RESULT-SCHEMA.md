# Agent Result Schema

## Purpose

The `AgentResult` message is what a Reasoning Adapter sends back after processing an `AgentJob`. It carries the output, tool invocations, errors, and usage metrics back to the Orchestrator.

## Schema

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AgentResult {
    /// The job this completes (echoed from AgentJob.job_id)
    pub job_id: String,
    
    /// Root correlation ID (echoed from AgentJob.correlation_parent)
    pub correlation_parent: String,

    /// What the adapter produced
    pub output: AgentOutput,

    /// Tool calls made during execution
    pub tool_invocations: Vec<ToolInvocation>,

    /// Errors that occurred (non-fatal)
    pub errors: Vec<AdapterError>,

    /// Token usage / cost metrics
    pub usage: Option<UsageMetrics>,

    /// Unix timestamp when the adapter completed
    pub completed_at: i64,
}
```

## Field Details

### `job_id`

Echoed from the incoming `AgentJob.job_id`. The Orchestrator uses this for deduplication.

### `correlation_parent`

Echoed from the incoming `AgentJob.correlation_parent`. This is the critical field — the Orchestrator groups all `AgentResult` messages with the same `correlation_parent` together to reconstruct the complete task result.

### `output: AgentOutput`

```rust
pub struct AgentOutput {
    /// Final text result
    pub text: Option<String>,
    /// Structured output (JSON, if requested)
    pub structured: Option<serde_json::Value>,
    /// Files written (relative to working_dir)
    pub files_written: Vec<PathBuf>,
    /// Commands executed
    pub commands_run: Vec<String>,
}
```

### `tool_invocations: Vec<ToolInvocation>`

Records every tool call the adapter made, including parameters, result, and duration:

```rust
pub struct ToolInvocation {
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub result: serde_json::Value,
    pub duration_ms: u64,
}
```

### `errors: Vec<AdapterError>`

Non-fatal errors encountered during execution:

```rust
pub struct AdapterError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}
```

### `usage: Option<UsageMetrics>`

```rust
pub struct UsageMetrics {
    pub prompt_tokens: usize,
    pub completion_tokens: usize, 
    pub total_tokens: usize,
    pub cost_cents: Option<f64>,
}
```

## Validation Rules

Receivers MUST:
1. Verify `job_id` matches an outstanding `AgentJob`
2. Check `correlation_parent` is well-formed
3. Reject results with empty `output.text` AND empty `output.structured` (one must be present)
4. Deduplicate by `job_id` — if two results arrive with the same `job_id`, keep the first

## Serialization Example

```json
{
  "job_id": "550e8400-e29b-41d4-a716-446655440000",
  "correlation_parent": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "output": {
    "text": "Here is the Python script you requested...",
    "files_written": ["scripts/sort_csv.py"],
    "commands_run": ["python3 scripts/sort_csv.py test.csv"]
  },
  "tool_invocations": [
    {
      "tool_name": "file_write",
      "parameters": {"path": "scripts/sort_csv.py", "content": "..."},
      "result": {"output": "Wrote 1024 bytes to scripts/sort_csv.py"},
      "duration_ms": 5
    }
  ],
  "errors": [],
  "usage": {
    "prompt_tokens": 450,
    "completion_tokens": 120,
    "total_tokens": 570,
    "cost_cents": 0.85
  },
  "completed_at": 1714880000
}
```

## Versioning

Same policy as AgentJob — add fields freely, remove fields with deprecation, change types only with `schema_version` bump.
