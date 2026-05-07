# Wireframe AI Example Modules

A collection of 11 example modules demonstrating common patterns for building Wireframe AI modules.

## Running Examples

Each example is a standalone Rust binary. To run:

```bash
cargo run --bin <module-name>
```

Ensure NATS is running first:

```bash
nats-server
```

## Module Catalog

### ping-module
**Pattern**: Basic request/response  
**Topics**: `ping.request` -> `ping.response`  
**Demonstrates**: `#[module]` macro, `Envelope::reply()`, graceful shutdown

```bash
cargo run --bin ping-module
```

### echo-module
**Pattern**: Echo service  
**Topics**: `echo.request` -> `echo.response`  
**Demonstrates**: Simple payload inspection, request/response

```bash
cargo run --bin echo-module
```

### logger-module
**Pattern**: Wildcard subscription listener  
**Topics**: `task.>`, `agent.>`, `sys.>`  
**Demonstrates**: Wildcard NATS subscriptions, structured logging of all messages

```bash
cargo run --bin logger-module
```

### metrics-module
**Pattern**: Metrics collection and aggregation  
**Topics**: `task.>`, `agent.>`, `sys.>`, `metrics.query` -> `metrics.query.response`, `metrics.snapshot`  
**Demonstrates**: Stateful module, periodic background publishing, query/response

```bash
cargo run --bin metrics-module
```

### validator-module
**Pattern**: Schema validation service  
**Topics**: `validate.request` -> `validate.response`  
**Demonstrates**: Payload validation, structured error responses

```bash
cargo run --bin validator-module
```

### webhook-receiver
**Pattern**: External event ingestion  
**Topics**: `webhook.incoming` -> `event.normalized`, `webhook.received`  
**Demonstrates**: Normalizing external payloads, dual response pattern

```bash
cargo run --bin webhook-receiver
```

### file-watcher-module
**Pattern**: File system monitoring  
**Topics**: `file.watch.request`, `file.unwatch.request` -> `file.watch.confirmed`, `file.changed`  
**Demonstrates**: Background task with interval, stateful file tracking

```bash
cargo run --bin file-watcher-module
```

### cache-module
**Pattern**: Key-value cache service  
**Topics**: `cache.get`, `cache.set`, `cache.delete`, `cache.clear` -> `cache.response`  
**Demonstrates**: Multi-topic service, TTL expiry, CRUD operations

```bash
cargo run --bin cache-module
```

### router-module
**Pattern**: Dynamic message routing  
**Topics**: `route.request`, `route.register`, `route.unregister` -> `route.response`, `routed.message`  
**Demonstrates**: Runtime configuration, rule-based forwarding

```bash
cargo run --bin router-module
```

### rate-limiter-module
**Pattern**: Quota enforcement  
**Topics**: `rate.check`, `rate.configure` -> `rate.result`, `rate.configured`  
**Demonstrates**: Per-session rate tracking, sliding window, dynamic reconfiguration

```bash
cargo run --bin rate-limiter-module
```

### health-check-module
**Pattern**: System health monitoring  
**Topics**: `sys.module.heartbeat`, `sys.module.online`, `sys.module.offline`, `health.query` -> `health.status`, `health.alert`  
**Demonstrates**: Heartbeat tracking, stale detection, alert publishing

```bash
cargo run --bin health-check-module
```

## Template Quick Reference

| Template | Use Case | Key Topics |
|----------|----------|-----------|
| Basic | Simple handler | `example.topic` |
| Adapter | LLM adapter | `agent.job` -> `agent.result` |
| Context | Context enrichment | `task.submitted` -> `task.enriched` |
| Orchestrator | Parallel dispatch | `task.enriched` -> `agent.job` + `task.complete` |
| Listener | Observability | `*.>` |
| Service | Request/response | `service.request` -> `service.response` |

Generate from templates with the CLI:

```bash
cargo run --bin wireframe -- new my-adapter --template adapter
```
