# Wireframe AI — Topic Namespace API

All topics follow `namespace.noun.verb` lowercase dot-separated.
Topics are organized into namespaces. Adding a new namespace never breaks existing modules.
Existing topic names within a namespace are immutable once shipped.

---

## `task` namespace — User task lifecycle

| Topic | Payload | Publisher | Subscribers | Purpose |
|---|---|---|---|---|
| `task.submitted` | `TaskSubmitted` | Interface | Context | User submits a new request |
| `task.enriched` | `TaskEnriched` | Context | Orchestrator (or Reasoning Adapter) | Task enriched with memory/context |
| `task.complete` | `TaskComplete` | Orchestrator (or Reasoning Adapter) | Interface | Final result sent back to user |

## `agent` namespace — Agent job dispatch

| Topic | Payload | Publisher | Subscribers | Purpose |
|---|---|---|---|---|
| `agent.job` | `AgentJob` | Orchestrator | Reasoning Adapter (`agent_worker` queue group) | Self-contained unit of work |
| `agent.result` | `AgentResult` | Reasoning Adapter | Orchestrator (`orchestrator_collector` queue group) | Result from a completed agent job |

## `provider` namespace — Provider discovery and management

| Topic | Payload | Publisher | Subscribers | Purpose |
|---|---|---|---|---|
| `provider.describe` | `ProviderDescribe` | Any module | Provider Registry | Request provider metadata and capabilities |
| `provider.describe.response` | `ProviderMetadata` | Provider Registry | Requester | Provider metadata response |
| `provider.status` | `ProviderStatusRequest` | Any module | Provider Registry | Request provider availability status |
| `provider.status.response` | `ProviderStatus` | Provider Registry | Requester | Provider status response |
| `provider.list` | `ProviderListRequest` | Any module | Provider Registry | List all available providers |
| `provider.list.response` | `ProviderListResponse` | Provider Registry | Requester | List of available providers |

## `exec` namespace — Sandbox execution

> **Note:** The sandbox does NOT use NATS topics. It communicates via MCP (Model Context Protocol) over stdio.
> The Reasoning Adapter spawns the sandbox subprocess and calls tools through JSON-RPC on stdin/stdout.
> Topics are reserved here for future use if the sandbox gains NATS support.

| Topic | Payload | Publisher | Subscribers | Purpose |
|---|---|---|---|---|
| `exec.request` | `ExecRequest` | (reserved) | (reserved) | Reserved for future NATS-based sandbox |
| `exec.result` | `ExecResult` | (reserved) | (reserved) | Reserved for future NATS-based sandbox |

## `sys` namespace — System lifecycle

| Topic | Payload | Publisher | Subscribers | Purpose |
|---|---|---|---|---|
| `sys.module.online` | ModuleIdentity | Any module | Any observer | Module announces itself on startup |
| `sys.module.offline` | ModuleIdentity | Any module | Any observer | Module announces graceful shutdown |
| `sys.module.heartbeat` | Heartbeat | Any module | Any observer | Periodic health check signal (optional) |
| `sys.module.error` | ModuleError | Any module | Any observer | Module reports errors for debugging |

## `module` namespace — Module lifecycle and selfdev

| Topic | Payload | Publisher | Subscribers | Purpose |
|---|---|---|---|---|
| `module.rebuild.request` | `ModuleRebuildRequest` | Any module (selfdev) | Process Manager | Request to rebuild a module with new source code |
| `module.rebuild.status` | `ModuleRebuildStatus` | Process Manager | Requester | Build status updates |
| `module.restart.request` | `ModuleRestartRequest` | Process Manager | Target Module | Request to restart with new binary |
| `module.restart.ack` | `ModuleRestartAck` | Target Module | Process Manager | Acknowledgment of restart |
| `module.switch.request` | `ModuleSwitchRequest` | Any module | Process Manager | Request to switch from one module to another |
| `module.switch.ack` | `ModuleSwitchAck` | Process Manager | Requester | Acknowledgment of module switch |

---

## Module registry (subscribers per topic)

| Topic | Queue Group | Competing Consumers |
|---|---|---|
| `task.submitted` | `task_handler` | Context (optional: multiple instances) |
| `task.enriched` | `task_handler` | Orchestrator XOR Reasoning Adapter |
| `agent.job` | `agent_worker` | Reasoning Adapter (N instances) |
| `agent.result` | `orchestrator_collector` | Orchestrator (only one) |
| `provider.describe` | `provider_registry` | Provider Registry (only one) |
| `provider.status` | `provider_registry` | Provider Registry (only one) |
| `provider.list` | `provider_registry` | Provider Registry (only one) |
| `exec.request` | `sandbox_worker` | (reserved — see Sandbox communication below) |
| `module.rebuild.request` | `process_manager` | Process Manager (only one) |
| `module.restart.request` | (no queue group) | Target module (by name) |
| `module.switch.request` | `process_manager` | Process Manager (only one) |

## Sandbox communication

The Sandbox module communicates via the **MCP protocol over stdio**, not NATS. When the Reasoning Adapter needs to execute a tool:

1. It spawns the sandbox binary (`wireframe-ai-sandbox`)
2. Performs MCP `initialize` handshake
3. Calls `/tools/call` with the tool name and arguments
4. Reads the JSON-RPC response
5. Terminates the sandbox process

This design keeps code execution isolated from the network — no open ports, no NATS subscription needed.

## Envelope structure

Every topic carries the standard Envelope wrapper:

```json
{
  "message_id": "uuid4",
  "session_id": "session_uuid",
  "correlation_id": "uuid4",
  "topic": "task.submitted",
  "schema_version": 1,
  "timestamp": 1714880000,
  "payload": { ... }
}
```

## Versioning

- Adding fields to payload structs: backward-compatible (ignore unknown)
- Removing fields: deprecate first, then remove in next major version
- Changing field types: breaking — increment `schema_version` in envelope
- New topic namespaces can be added freely
- Existing topic names are immutable once any module ships against them
