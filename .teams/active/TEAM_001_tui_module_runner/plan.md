# Feature Implementation Plan - TEAM_001

## Feature Overview
Implement a Terminal User Interface (TUI) module runner that consolidates all Wireframe AI modules into a single window with split panes. Each module runs as a separate child process with output captured and displayed in its dedicated pane.

## Requirements

- Single TUI window with split panes for each module (context, orchestrator, sandbox, adapter, interface)
- Each module runs as a separate child process
- Real-time capture and display of stdout/stderr for each module
- Configurable via TOML file (which modules to show, layout, log levels)
- Keyboard shortcuts: Ctrl+C to stop all, Tab to switch panes, q to quit
- Status indicators for each module (running/stopped/error)
- Cross-platform (Windows, Linux, macOS)

## Design Decisions

- **Framework**: ratatui (Rust-native TUI library)
- **Async runtime**: tokio (already in use in project)
- **Process spawning**: tokio::process::Command
- **Output capture**: tokio::io::lines() for stdout/stderr
- **Configuration**: TOML format (.wireframe-runner.toml)
- **Layout**: Horizontal split panes (one row per module)
- **Buffering**: Circular buffer for log lines (max 1000 lines per pane)

## Implementation Tasks

1. Create new crate: `tools/tui-runner/Cargo.toml`
2. Add dependencies: ratatui, crossterm, tokio, serde, toml
3. Create configuration structure and TOML parser
4. Implement module process spawning logic
5. Implement output capture and buffering
6. Implement TUI layout with split panes
7. Implement keyboard event handling
8. Implement status tracking for each module
9. Add start/stop/restart controls
10. Create example configuration file
11. Update start script to optionally use TUI runner
12. Test with all modules

## Testing Strategy

- Unit tests for configuration parsing
- Integration tests for process spawning and output capture
- Manual testing with actual Wireframe AI modules
- Test keyboard shortcuts
- Test configuration loading
- Test error handling (module crash, etc.)

## Dependencies

- External: ratatui, crossterm, serde, toml
- Internal: None (standalone tool)

## Success Criteria

- Single TUI window displays all module outputs
- Modules can be started/stopped individually
- All modules can be stopped with Ctrl+C
- Configuration file controls which modules show
- Cross-platform compatibility verified
- Performance acceptable (no lag, reasonable memory usage)
