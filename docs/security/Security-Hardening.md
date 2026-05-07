# Security Hardening Guide

## Overview

Wireframe-AI is designed with defense-in-depth security. This guide covers hardening for production deployments.

## Authentication & Authorization

### NATS Security

- Enable NATS TLS: `--tls --tlscert server.crt --tlskey server.key`
- Use NATS credentials files instead of anonymous connections
- Enable NATS JWT authentication for multi-tenant deployments

```bash
nats-server --tls --tlscert server.crt --tlskey server.key --auth mytoken
```

### Module Authentication

Modules should validate messages using:
- Correlation ID signing (optional)
- Source module verification via `sys.module.online` registry
- Topic ACLs enforced by NATS

## Input Validation

All modules must:
1. Validate JSON schemas before processing (use `agentic-sdk::validate_envelope_payload`)
2. Sanitize user input to prevent injection
3. Limit payload sizes (max 10MB recommended)
4. Validate topic names match expected patterns

## Sandboxing

The `wireframe-ai-sandbox` module provides:
- Filesystem isolation (restricted to `WIREFRAME_AI_SANDBOX_ROOT`)
- Network policy enforcement (`NetworkPolicy::Sandboxed`)
- Command allowlisting (shell_exec only permits known-safe commands)
- Resource limits (CPU/memory caps via Docker/cgroups)

### Enabling Full Sandbox Mode

```bash
export WIREFRAME_AI_EXECUTION_MODE=sandbox
export WIREFRAME_AI_SANDBOX_ROOT=/tmp/wireframe-sandbox
export WIREFRAME_AI_SANDBOX_ALLOWED_COMMANDS='["ls","cat","grep","find","wc"]'
```

## Secret Management

Never store API keys in module source code. Use:
1. Environment variables (development only)
2. Kubernetes Secrets (production)
3. Vault integration (enterprise)

### Kubernetes Secret Example

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: provider-secrets
  namespace: wireframe-ai
type: Opaque
stringData:
  OPENAI_API_KEY: "sk-..."
  ANTHROPIC_API_KEY: "sk-ant-..."
```

Mount as environment variables in deployment:

```yaml
envFrom:
  - secretRef:
      name: provider-secrets
```

## Audit Logging

The `wireframe-ai-event-sourcing-core` module captures all messages:
- Immutable event log
- Queryable by correlation_id, topic, timestamp
- Export to SIEM via `audit.query.result`

Enable audit logging:
```bash
nats pub sys.config.set '{"audit_logging":true}'
```

## Rate Limiting

Use the `wireframe-ai-tenant-core` module for quota enforcement:
- Per-tenant token limits
- Per-tenant request rate limits
- Per-tenant concurrent job limits

## Network Security

- Deploy NATS behind a firewall, accessible only to module pods/services
- Use mTLS between modules and NATS
- Isolate external-facing modules (webhooks-core) in a DMZ

## Selfdev Security

Selfdev mode is **disabled by default** in production. If enabled:
1. Require explicit intent detection keywords
2. Log all selfdev operations to `~/.wireframe-ai/selfdev.log`
3. Maintain rollback binaries
4. Require human approval for destructive operations

## Vulnerability Management

- Run `cargo audit` before each release
- Pin dependency versions in `Cargo.lock`
- Use `cargo-deny` to ban vulnerable crates
- Subscribe to RustSec advisories

## Compliance

- GDPR: Tenant module supports data isolation and deletion
- SOC2: Event sourcing provides audit trails
- HIPAA: Deploy with end-to-end encryption and access controls
