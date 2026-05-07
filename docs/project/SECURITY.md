# Security Considerations

This document outlines the security measures implemented in Wireframe AI to ensure safe deployment to production.

## Sandbox Module Security

### Command Execution

The sandbox module executes commands in an isolated environment with the following security measures:

1. **Command Whitelist**: Only whitelisted executables can be run. The default whitelist includes common local development tools (python, node, cargo, git, etc.). Network access tools (ssh, scp, rsync), container management tools (docker, docker-compose), and cloud infrastructure tools (kubectl, terraform, aws, az, gcloud) are **excluded** from the default whitelist for security. These can be added via the `WIREFRAME_AI_SANDBOX_ALLOWED_COMMANDS` environment variable if needed for your specific use case.

2. **No Shell Interpretation**: Commands are executed directly without shell interpretation (no `sh -c` or `cmd /C`). This prevents command injection attacks.

3. **Proper Argument Parsing**: Commands are parsed using the `shell-words` crate, which correctly handles quoted arguments (e.g., `python script.py "hello world"`). This ensures that arguments with spaces are passed correctly to the command without requiring shell interpretation.

4. **Argument Validation**: Shell metacharacters (`|`, `&`, `;`, `$`, `` ` ``, `(`, `)`, `<`, `>`, `\`, newlines) are rejected in arguments as a defense-in-depth measure.

5. **Command Length Limits**: Maximum command length of 1000 characters and maximum of 50 arguments to prevent DoS attacks.

6. **Resource Limits**: On Unix systems, sandbox processes are limited to 300 seconds of CPU time and 1GB of address space to prevent resource exhaustion attacks. Note: These resource limits are only enforced on Unix/Linux systems via libc. On Windows and other platforms, OS-level resource limits should be configured separately.

7. **Working Directory Validation**: Working directories are validated to ensure they remain within the sandbox root.

### File Operations

File operations (read, write, list) include:

1. **Path Validation**: Paths are validated to prevent directory traversal attacks:
   - No `..` components allowed
   - No absolute paths allowed
   - No drive letters (Windows)
   - Maximum path length of 4096 characters

2. **Symlink Protection**: After path validation, canonical paths are checked to ensure they remain within the sandbox root (defense-in-depth against symlink attacks).

3. **TOCTOU Mitigation**: Path validation happens before filesystem operations to minimize time-of-check-time-of-use race conditions.

## Context Module Security

### FTS5 Search

The context module uses SQLite FTS5 for full-text search with these protections:

1. **Parameterized Queries**: All SQL queries use parameterized statements (`?1`, `?2`) to prevent SQL injection.

2. **Input Sanitization**: Search queries are sanitized to only allow alphanumeric characters, spaces, hyphens, and apostrophes.

3. **Query Length Limits**: Maximum query length of 1000 characters to prevent DoS attacks.

4. **FTS5 Escaping**: Double quotes in search terms are properly escaped using FTS5's doubling mechanism (`"` → `""`).

## Schema Validation

Wireframe AI includes optional JSON Schema validation for message payloads. When enabled, the system validates all messages against embedded schemas before publishing them to NATS. This ensures contract compliance between modules.

**To enable schema validation:**

```bash
# Build with the schema-validation feature
cargo build --release --features schema-validation

# Or enable it for a specific module
cargo build -p wireframe-ai-interface --features schema-validation
```

**Schemas are embedded in the binary for:**
- `task.submitted` → `EMBEDDED_TASK_SUBMITTED_SCHEMA`
- `task.enriched` → `EMBEDDED_TASK_ENRICHED_SCHEMA`
- `task.complete` → `EMBEDDED_TASK_COMPLETE_SCHEMA`
- `agent.job` → `EMBEDDED_AGENT_JOB_SCHEMA`
- `agent.result` → `EMBEDDED_AGENT_RESULT_SCHEMA`

The validation is performed by the `agentic-sdk::validate_envelope_payload()` function, which automatically selects the appropriate embedded schema based on the topic and validates the payload. If validation fails, the module logs an error and refuses to publish the message.

**Why embedded schemas?** Schemas are embedded as string constants in the binary rather than loaded from files at runtime. This ensures that:
- Schema validation works regardless of where the binary is executed from
- No file system dependencies or path resolution issues
- Schemas are always available and version-locked with the binary
- No risk of schema files being modified or deleted in production

**Note:** Schema validation is optional and disabled by default to avoid runtime overhead. Enable it in production environments where contract compliance is critical.

## Environment Variable Filtering

The context module filters environment variables before including them in the context package:

- Variables containing `key`, `secret`, `password`, `token`, `api_key`, `api_secret`, `auth`, `credential`, or `private` (case-insensitive) are excluded
- This prevents accidental exposure of sensitive credentials to AI agents
- Note: We use specific patterns like `api_key` and `api_secret` instead of the broader `api` to avoid filtering legitimate non-secret variables like `API_ENDPOINT` or `API_VERSION`

## Network Security

- The orchestrator defaults to `NetworkPolicy::OutboundOnly`, allowing only outbound HTTP connections
- No inbound listeners are permitted by default
- Filesystem policy defaults to `SandboxWritable`, confining operations to the working directory

## Recommendations for Production

1. **Run as Non-Root User**: All modules should run as a non-privileged user.

2. **Resource Limits**: Configure OS-level resource limits (CPU, memory, file descriptors) for sandbox processes.

3. **Network Isolation**: Consider running the sandbox in a network namespace or container with restricted network access.

4. **Audit Whitelist**: Review and customize the command whitelist for your specific use case.

5. **Filesystem Permissions**: Ensure the sandbox root directory has appropriate permissions (read/write for the sandbox user only).

6. **Monitoring**: Enable logging and monitoring for security events (failed validations, path traversal attempts, etc.).

7. **Regular Updates**: Keep dependencies (SQLite, Rust crates, Python packages) updated for security patches.

## Security Testing

The project includes integration tests that verify:
- Envelope serialization roundtrips
- Message type validation
- Path validation logic
- Command validation logic

Run tests with:
```bash
cargo test
```

## Reporting Security Issues

If you discover a security vulnerability, please report it responsibly through the project's security contact or issue tracker with the "security" label.
