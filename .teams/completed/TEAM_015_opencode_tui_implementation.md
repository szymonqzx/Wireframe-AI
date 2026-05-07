---
status: completed
created: 2026-05-06
completed: 2026-05-06
---

# TEAM_015 - OpenCode TUI Pattern Implementation

## Task
Implement OpenCode TUI patterns in Wireframe-AI TUI, starting with Command Palette as the foundation.

## Progress
- [x] Phase 1: Command Palette
  - [x] Design command palette data structure
  - [x] Implement command palette overlay
  - [x] Add searchable command list
  - [x] Integrate with existing slash commands
  - [x] Add palette to leader key system
- [x] Phase 2: Enhanced Slash Commands
  - [x] Add command aliases
  - [x] Implement missing commands (/connect, /models, /thinking, /share)
  - [x] Add command descriptions
  - [x] Group commands in help dialog
  - [x] Add command discovery in palette
- [x] Phase 3: File Reference System
  - [x] Implement file search with @ prefix
  - [x] Auto-inject file content for single matches
  - [x] Show list for multiple matches
  - [x] Handle file read errors
- [x] Phase 4: Bash Command Integration
  - [x] Implement ! prefix for shell commands
  - [x] Safe command execution with tokio::process
  - [x] Display command output as tool result
  - [x] Handle command errors
- [ ] Phase 5: Git-Based Session Management
- [ ] Phase 6: Configurable Keybindings
- [ ] Phase 7: Advanced TUI Config
- [ ] Phase 8: Agent Switching

## Decisions
- Start with Command Palette as foundation
- Maintain Elm Architecture
- Preserve existing side panels and overlays
- Use Rust ecosystem crates (nucleo for fuzzy search, git2 for Git)
- Incremental implementation to avoid breaking changes

## Implementation Details

### Phases 1-4 Completed
- Command palette with Ctrl+P toggle
- Searchable command list with descriptions
- 5 new OpenCode commands: /connect, /models, /thinking, /share, /unshare
- Command aliases already exist (e.g., /quit → /q)
- Enhanced autocomplete with all commands
- Command palette shows descriptions
- File reference system with @ prefix
- Auto-inject file content for single matches
- Bash command integration with ! prefix
- Safe command execution with tokio::process

### Remaining Phases
- Phase 5: Git-Based Session Management (complex, requires git2 integration)
- Phase 6: Configurable Keybindings (requires config system split)
- Phase 7: Advanced TUI Config (requires tui.json split)
- Phase 8: Agent Switching (requires agent state management)
