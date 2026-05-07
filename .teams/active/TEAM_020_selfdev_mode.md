---
status: active
created: 2026-05-06
---

# TEAM_020 - Selfdev Mode Implementation

## Task
Implement JCODE-style selfdev mode for Wireframe-AI agents, allowing runtime code modification, recompilation, and self-replacement. Make sandbox optional and enable direct PC execution.

## Requirements
1. **Selfdev Capability**: Agents can read their own source code, modify it, compile it, and replace their running binary
2. **Optional Sandbox**: Sandbox should be optional, not required for basic operation
3. **Direct PC Execution**: Agents can run directly on the host PC without isolation
4. **Security**: Maintain security boundaries while enabling self-modification
5. **Coordination**: NATS-based coordination for hot-swapping modules

## Progress
- [x] Create team file for selfdev implementation
- [x] Design selfdev architecture and security model
- [x] Update NATS schemas for selfdev coordination
- [x] Add optional sandbox mode to adapter
- [x] Implement source code access mechanism
- [x] Add binary replacement and process restart capability
- [x] Update documentation
- [x] Implement runtime module switching logic
- [ ] Test selfdev mode with code modification
- [ ] Test module switching functionality

## Decisions

### Selfdev Activation
- **Opt-in with intent detection**: Selfdev activates when agent prompts suggest editing own code (keywords: "edit my code", "modify myself", "improve my implementation")
- **Environment override**: `WIREFRAME_AI_SELFDEV=true` can force-enable for testing
- **All modules support selfdev**: Consistency across the system

### Execution Modes
- **Configurable per deployment**: Three modes (sandbox, direct, hybrid)
- **Sandbox mode**: Default for production, isolated execution
- **Direct mode**: Default for development, full host access
- **Hybrid mode**: Sandbox for normal ops, direct for selfdev

### Security Model
- **Intent detection**: Prevents accidental selfdev
- **Audit logging**: All selfdev operations logged to `~/.wireframe-ai/selfdev.log`
- **Rollback capability**: Keep previous binary for recovery
- **Optional approval**: Human approval for production selfdev
- **Rate limiting**: Prevent abuse

### Coordination
- **NATS-based**: Leverage existing message bus
- **New topics**: module.rebuild.request, module.rebuild.status, module.restart.request, module.restart.ack
- **Graceful shutdown**: Module finishes current job before restart

### Use Case
- **Dev + Production**: Selfdev works in both environments with appropriate security controls

## Handoff Notes

### Implementation Summary

**Completed:**
1. ✅ Architecture design documented in `docs/architecture/adr-020-selfdev-mode.md`
2. ✅ NATS schemas created for selfdev coordination:
   - `module_rebuild_request.json`
   - `module_rebuild_status.json`
   - `module_restart_request.json`
   - `module_restart_ack.json`
   - `module_switch_request.json`
   - `module_switch_ack.json`
   - `sys_module_online.json` (updated with selfdev fields)
3. ✅ TOPICS.md updated with new `module` namespace
4. ✅ SDK updated with:
   - `announce_online_with_selfdev()` function
   - `CompatibilityChecker` for module compatibility validation
   - `ModuleRegistry` for tracking installed modules
   - `ModuleSwitchCoordinator` for runtime switching logic
5. ✅ ExecutionConstraints updated with `execution_mode` field
6. ✅ Rust adapter enhanced with:
   - Three execution modes (Sandbox, Direct, Hybrid)
   - Selfdev intent detection
   - Source code access tools (read_source, write_source)
   - Self-compilation tool (compile_self)
   - Self-restart tool (restart_self)
   - Module switch tool (switch_module)
   - Platform-aware shell execution (Windows/Unix)
7. ✅ Process manager script enhanced (`scripts/process-manager.ps1`) with module switching support
8. ✅ README.md updated with selfdev and module switching documentation

**Key Files Modified:**
- `sdk/agentic-sdk/src/module.rs` - Added selfdev announcement function
- `sdk/agentic-sdk/src/message_types.rs` - Added execution_mode field
- `sdk/agentic-sdk/src/compatibility.rs` - NEW: Compatibility checking logic
- `sdk/agentic-sdk/src/registry.rs` - NEW: Module registry implementation
- `sdk/agentic-sdk/src/switch.rs` - NEW: Module switch coordination logic
- `sdk/agentic-sdk/src/lib.rs` - Re-exports for new modules
- `adapter/rust/src/main.rs` - Implemented selfdev tools, execution modes, and module switch tool
- `schemas/v1/TOPICS.md` - Added module namespace with switch topics
- `schemas/v1/agent_job.json` - Added execution_mode field
- `Cargo.toml` - Added tokio process feature
- `README.md` - Added selfdev and module switching documentation

### Remaining Work

**Testing Required:**
1. Test selfdev mode with actual code modification
2. Test process manager restart flow
3. Test NATS coordination for rebuild/restart
4. Test intent detection with various prompts
5. Test all three execution modes
6. Test rollback capability
7. Test audit logging
8. Test module switching with compatible modules
9. Test module switching with incompatible modules
10. Test module switching rollback on failure
11. Test compatibility checker with various module types

**Future Enhancements:**
1. Implement NATS client in process manager for automatic coordination
2. Add audit logging implementation
3. Add rate limiting for selfdev operations
4. Add human approval workflow for production
5. Implement graceful shutdown handler in adapter
6. Add binary signing verification
7. Create selfdev test suite
8. Implement module discovery (download from git/registry)
9. Add state migration between modules
10. Implement NATS-based online detection for module switching

### Environment Variables

To enable selfdev mode:
```bash
export WIREFRAME_AI_SELFDEV=true
export WIREFRAME_AI_SOURCE_ROOT=/path/to/wireframe-ai
export WIREFRAME_AI_EXECUTION_MODE=direct  # or sandbox, hybrid
```

### Usage Example

**Selfdev Mode:**
Agent can now:
1. Read its source: `read_source(path="adapter/rust/src/main.rs")`
2. Modify code: `write_source(path="adapter/rust/src/main.rs", content="...")`
3. Compile: `compile_self()`
4. Restart: `restart_self(auto_restart=true)`

The adapter will detect selfdev intent from prompts containing keywords like "edit my code", "modify myself", etc., and automatically switch to direct execution mode.

**Module Switching:**
Agent can switch to a different module:
```json
{
  "tool": "switch_module",
  "parameters": {
    "new_module": "community-adapter-x",
    "force": false
  }
}
```

Or via process manager:
```bash
.\scripts\process-manager.ps1 -Switch -OldModule wireframe-adapter-rust -NewModule community-adapter-x
```

The system will:
1. Check compatibility between modules
2. Stop current module
3. Start new module
4. Verify new module is online
5. Rollback on failure
