# ADR-020: Selfdev Mode Implementation

## Status
Proposed

## Context
Wireframe-AI currently uses static compiled binaries that communicate via NATS. The system was designed for modularity and replaceability, but not for runtime self-modification. Users want JCODE-style selfdev capability where agents can read their own source code, modify it, compile it, and replace their running binary.

**Requirements:**
- Agents can read their own source code
- Agents can modify their source code
- Agents can compile themselves
- Agents can replace their running binary
- Agents can restart with the new version
- Sandbox should be optional
- Agents can run directly on the host PC
- Selfdev should be opt-in, triggered by agent prompts
- All modules should support selfdev
- Execution mode should be configurable
- Selfdev should work in both development and production

**Constraints:**
- Maintain security boundaries while enabling self-modification
- Balance flexibility with safety
- Support both sandboxed and direct execution modes
- Coordinate hot-swaps via NATS

## Decision

### Core Architecture

**Selfdev Mode Activation:**
- Selfdev is opt-in, activated when an agent's prompt suggests editing its own code
- Detection via heuristic analysis of agent prompts (keywords: "edit my code", "modify myself", "improve my implementation")
- Environment variable `WIREFRAME_AI_SELFDEV=true` can force-enable for testing
- Selfdev capability is advertised via NATS `sys.module.online` message

**Execution Modes:**
Three execution modes, configurable per deployment:

1. **Sandbox Mode** (default for production)
   - Agent runs in isolated sandbox environment
   - File operations confined to sandbox directory
   - Resource limits enforced (CPU, memory, timeout)
   - Selfdev disabled by default (can be enabled with explicit flag)

2. **Direct Mode** (default for development)
   - Agent runs directly on host PC
   - Full filesystem access (within OS permissions)
   - No resource limits
   - Selfdev enabled by default

3. **Hybrid Mode**
   - Agent runs in sandbox for normal operations
   - Selfdev operations temporarily elevate to direct mode
   - Requires explicit approval for each selfdev operation

**Source Code Access:**
- Source directory mounted at known path: `WIREFRAME_AI_SOURCE_ROOT` (default: current working directory)
- Agent can read source files via file operations
- Source path is included in module metadata on startup
- Selfdev mode exposes source root to agent; non-selfdev mode hides it

**Binary Replacement Mechanism:**
- New binary compiled to `target/release/<module-name>`
- Process manager service handles hot-swap coordination
- NATS topics for coordination:
  - `module.rebuild.request` - Request to rebuild a module
  - `module.rebuild.status` - Build status updates
  - `module.restart.request` - Request to restart with new binary
  - `module.restart.ack` - Acknowledgment of restart
- Graceful shutdown: module finishes current job, publishes `sys.module.offline`, exits
- Process manager (or orchestration layer) starts new binary
- Rollback: keep previous binary, can revert on failure

**Security Model:**
- Selfdev operations require explicit agent intent detection
- Audit logging: all selfdev operations logged to `~/.wireframe-ai/selfdev.log`
- Binary signing: optional, verify binary integrity before hot-swap
- Permission checks: OS-level permissions still apply
- Rate limiting: limit selfdev operations to prevent abuse
- Approval workflow: optional human approval for production selfdev

### Module-Level Changes

**All modules gain:**
1. Selfdev capability flag in module metadata
2. Source root path configuration
3. Binary path configuration
4. Process coordination via NATS
5. Graceful shutdown handler
6. Selfdev operation logging

**Adapter-specific:**
- Tool execution supports both sandbox (MCP) and direct (local) modes
- Runtime switch between modes based on configuration
- Source code access tools (read_source, write_source, compile_self)

**Sandbox-specific:**
- Remains optional - can be disabled entirely
- When disabled, adapter uses direct mode
- When enabled, can be bypassed for selfdev operations (hybrid mode)

### NATS Schema Updates

New schemas:
- `module_rebuild_request.json` - Request to rebuild a module
- `module_rebuild_status.json` - Build status updates
- `module_restart_request.json` - Request to restart with new binary
- `module_restart_ack.json` - Acknowledgment of restart

Updated schemas:
- `sys_module_online.json` - Add selfdev_capability flag, source_root, binary_path
- `agent_job.json` - Add execution_mode field (sandbox/direct/hybrid)

## Rationale

1. **Opt-in activation**: Selfdev is powerful but risky. Agent intent detection ensures it's only used when genuinely needed, not accidentally.

2. **All modules support selfdev**: Consistency across the system. Any module should be able to improve itself.

3. **Configurable execution mode**: Different environments have different needs. Development prefers direct access; production prefers isolation.

4. **Dev + production support**: Selfdev is valuable for continuous improvement in production (A/B testing, bug fixes, optimization), not just development.

5. **Hybrid mode**: Best of both worlds - isolation for normal operations, flexibility for selfdev.

6. **NATS coordination**: Leverages existing message bus infrastructure. No new coordination layer needed.

7. **Security boundaries**: Audit logging, optional signing, and approval workflows mitigate risks while enabling flexibility.

## Trade-offs

**Accepted:**
- Increased complexity in module initialization (selfdev detection, mode configuration)
- Additional NATS topics and schemas
- Security surface area increased (agents can modify their own code)
- Potential for infinite self-modification loops (mitigated by rate limiting)
- Process manager dependency for hot-swaps (can be simple script initially)

**Why acceptable:**
- Complexity is isolated to selfdev mode; normal operation unchanged
- NATS infrastructure already exists; adding topics is low cost
- Security risks are mitigated by intent detection, audit logging, and optional approval
- Rate limiting prevents abuse
- Process manager can start as simple shell script, evolve if needed

## Consequences

**Positive:**
- Agents can continuously improve themselves
- Faster iteration cycles (no manual rebuilds)
- Enables autonomous optimization and bug fixing
- Maintains existing modular architecture
- Flexible execution modes for different use cases

**Negative:**
- Increased attack surface (agents can modify their own code)
- More complex deployment (process manager, coordination)
- Potential for unstable self-modifications (mitigated by rollback)
- Additional testing burden (selfdev paths)

**Mitigation:**
- Intent detection prevents accidental selfdev
- Audit logging provides traceability
- Rollback capability recovers from bad modifications
- Optional approval workflow for production
- Comprehensive testing of selfdev paths
- Rate limiting prevents abuse

## Revisit Trigger
- Security incident related to selfdev mode
- Selfdev usage patterns indicate need for different activation mechanism
- Process coordination becomes bottleneck
- Performance issues with hybrid mode switching
- User feedback indicates preference for different default modes

## Implementation Plan

1. Add selfdev detection heuristics to adapter
2. Update module metadata with selfdev capability flags
3. Add execution mode configuration to all modules
4. Implement source code access tools
5. Add binary compilation and replacement logic
6. Create NATS schemas for coordination
7. Implement process manager (simple script initially)
8. Add audit logging
9. Test selfdev end-to-end
10. Update documentation
