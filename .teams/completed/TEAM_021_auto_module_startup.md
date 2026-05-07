---
status: completed
created: 2026-05-06
completed: 2026-05-06
---

# TEAM_021 - Automatic Module and NATS Startup

## Task
Implement automatic startup of NATS server and all required modules when the TUI starts, with proper lifecycle management and cleanup on exit.

## Requirements
- Detect if NATS is already running
- Start NATS server if not running
- Start orchestrator module
- Start reasoning adapter module
- Start any other required modules
- Manage process lifecycle (start, monitor, stop)
- Clean shutdown on TUI exit
- Handle errors gracefully
- Show status in TUI UI

## Progress
- [x] Analyze current module startup process
- [x] Design process management system
- [x] Implement NATS detection and startup
- [x] Implement orchestrator startup
- [x] Implement reasoning adapter startup
- [x] Add process monitoring
- [x] Implement graceful shutdown
- [x] Add status indicators in UI
- [x] Test automatic startup
- [x] Update documentation

## Architecture
- Add process manager to handle subprocess lifecycle
- Detect existing NATS instances to avoid conflicts
- Start modules in dependency order (NATS → orchestrator → reasoning adapter)
- Track PIDs for cleanup
- Handle SIGINT/SIGTERM for graceful shutdown
- Show module status in TUI UI

## Implementation Plan
1. Create process manager module
2. Add NATS detection and startup logic
3. Add module startup logic for orchestrator and reasoning adapter
4. Integrate startup into TUI main.rs
5. Add shutdown handler for cleanup
6. Update UI to show module status
7. Test full lifecycle

## Implementation Details

### Process Manager Module (process_manager.rs)
- Created `ProcessManager` struct with Arc<Mutex<HashMap<String, Child>>>
- Implemented `is_process_running()` - checks if process exists via tasklist (Windows) or pgrep (Unix)
- Implemented `start_nats()` - spawns NATS server from kernel/nats/nats-server.exe
- Implemented `start_module()` - runs `cargo run --release -p <package>` for modules
- Implemented `start_all_modules()` - starts NATS, waits 2 seconds, then starts context, orchestrator, sandbox modules
- Implemented `stop_all()` - kills all managed processes on shutdown

### CLI Integration (main.rs)
- Added `--auto-start` CLI flag to enable automatic module startup (default: true)
- Added ProcessManager initialization in main()
- Added shutdown handler to stop processes on Ctrl+C
- Added cleanup handler to stop processes on normal exit
- Removed duplicate signal handlers from run_app() loop

### Dependencies
- Initially added tracing and tracing-subscriber for logging
- Removed tracing dependencies to simplify (using eprintln instead)
- Process manager uses eprintln for status messages

### Build Status
- TUI compiles successfully with no warnings
- All dead code removed (unused process_count, process_names methods)
- Unused imports removed (tokio::signal)

## Handoff Notes
- Auto-start is enabled by default (use `--auto-start=false` to disable)
- Process manager handles Windows-specific process detection via tasklist
- Modules are started in dependency order: NATS → context → orchestrator → sandbox
- NATS binary path: kernel/nats/nats-server.exe
- Error handling: if a module fails to start, a warning is printed but other modules continue
- Cleanup is automatic on both Ctrl+C and normal exit
- Status messages are printed to stderr (eprintln) for visibility
