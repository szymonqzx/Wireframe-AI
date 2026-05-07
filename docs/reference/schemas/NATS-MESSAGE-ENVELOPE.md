# NATS Message Envelope

All inter-module communication in Wireframe AI uses a uniform JSON envelope.

## Structure

```json
{
  "message_id": "550e8400-e29b-41d4-a716-446655440000",
  "session_id": "session_e29b41d4-a716-446655440000",
  "correlation_id": "550e8400-e29b-41d4-a716-446655440000",
  "topic": "task.submitted",
  "schema_version": 1,
  "timestamp": 1714880000,
  "payload": { ... }
}
```

## Fields

| Field | Type | Description |
|---|---|---|
| `message_id` | UUID v4 | Globally unique for this exact message |
| `session_id` | `session_<uuid>` | Groups all messages in one conversation |
| `correlation_id` | UUID v4 | Links messages across the request lifecycle |
| `topic` | `namespace.noun.verb` | Routing key |
| `payload` | object | Topic-specific typed payload |
| `schema_version` | u32 | Envelope schema version (currently 1) |
| `timestamp` | i64 | Unix seconds when created |

## Topic Flow

```
┌──────────┐      task.submitted      ┌──────────┐
│          │ ───────────────────────>  │          │
│ Interface│                          │  Context │
│          │ <── task.complete ───────│          │
└──────────┘                          └────┬─────┘
                                           │ task.enriched
                                           ▼
                                     ┌──────────┐
                                     │Orchestrator│
                                     └────┬──────┘
                                          │ agent.job (× N)
                                          ▼
                                     ┌──────────┐
                                     │  Adapter  │
                                     └────┬──────┘
                                          │ agent.result (× N)
                                          ▼
                                     ┌──────────┐
                                     │Orchestrator│
                                     └────┬──────┘
                                          │ task.complete
                                          ▼
                                     ┌──────────┐
                                     │ Interface │
                                     └──────────┘
```

## Child Envelopes

When an orchestrator splits a task into sub-jobs, child envelopes inherit the parent's `session_id` and derive `correlation_id`:

```
Parent correlation:  "550e8400-e29b-41d4-a716-446655440000"
Child 1 correlation: "550e8400-e29b-41d4-a716-446655440000-1"
Child 2 correlation: "550e8400-e29b-41d4-a716-446655440000-2"
```

## Validation Rules

Receivers MUST:

1. Check `schema_version` — reject if > 1 (unknown future version)
2. Ensure non-empty `message_id`, `session_id`, `correlation_id`, `topic`
3. Reject messages with `timestamp` > now + 60s (future-dated, likely clock skew)
4. Verify `topic` matches expected subscription pattern

## Implementation

- **Rust**: `sdk/agentic-sdk/src/envelope.rs` — generic `Envelope<T>` struct with `new()`, `child()`, `validate()`
- **Python**: `sdk/agentic-sdk-py/src/agentic_sdk/envelope.py` — `Envelope[T]` dataclass with mirror API
