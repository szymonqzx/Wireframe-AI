---
status: completed
created: 2026-05-06
completed: 2026-05-06
---

# TEAM_014 - OpenCode TUI Research and Integration Planning

## Task
Research OpenCode's actual TUI implementation to understand its UX/UI patterns and plan how to apply them to the Wireframe-AI TUI.

## Progress
- [x] Research OpenCode TUI architecture and patterns
- [x] Document key UX/UI concepts from OpenCode
- [x] Analyze current Wireframe-AI TUI implementation
- [x] Map OpenCode patterns to Wireframe-AI TUI equivalents
- [x] Create implementation plan
- [x] Document design decisions

## Research Findings

### OpenCode TUI Overview

OpenCode is a TypeScript/JavaScript-based TUI (not Rust/ratatui like Wireframe-AI). It's built as part of a larger monorepo with multiple packages including a web console, desktop app, and CLI tools.

**Key Architecture:**
- Built with TypeScript/JavaScript (Bun runtime)
- Client/server architecture (TUI is just one possible client)
- Uses a command palette system (`ctrl+p`)
- Leader key system (`ctrl+x` as default)
- Slash command system (`/command`)
- File reference system with `@` for fuzzy search
- Bash command execution with `!`
- Session management with undo/redo via Git
- Theme system with JSON config
- Customizable keybindings

### OpenCode TUI Key Features

1. **Command Palette** (`ctrl+p`)
   - Central hub for all TUI customization
   - Settings persist across restarts
   - Quick access to all features

2. **Slash Commands**
   - `/connect` - Add provider
   - `/compact` - Compact session (alias: `/summarize`)
   - `/details` - Toggle tool execution details
   - `/editor` - Open external editor
   - `/exit` - Exit (aliases: `/quit`, `/q`)
   - `/export` - Export conversation to Markdown
   - `/help` - Show help dialog
   - `/init` - Guided setup for AGENTS.md
   - `/models` - List available models
   - `/new` - Start new session (alias: `/clear`)
   - `/redo` - Redo undone message
   - `/sessions` - List/switch sessions (aliases: `/resume`, `/continue`)
   - `/share` - Share session
   - `/themes` - List themes
   - `/thinking` - Toggle thinking visibility
   - `/undo` - Undo last message
   - `/unshare` - Unshare session

3. **Leader Key System** (`ctrl+x`)
   - `ctrl+x c` - Compact
   - `ctrl+x e` - Editor
   - `ctrl+x q` - Exit
   - `ctrl+x x` - Export
   - `ctrl+x m` - Models
   - `ctrl+x n` - New
   - `ctrl+x r` - Redo
   - `ctrl+x l` - Sessions
   - `ctrl+x t` - Themes
   - `ctrl+x u` - Undo

4. **File References** (`@`)
   - Fuzzy file search in current directory
   - Automatic content injection into conversation
   - Example: `@packages/functions/src/api/index.ts`

5. **Bash Commands** (`!`)
   - Execute shell commands
   - Output added as tool result
   - Example: `!ls -la`

6. **Session Management**
   - Undo/redo via Git integration
   - Session persistence
   - Session sharing
   - Multiple sessions support

7. **Configuration**
   - `tui.json` for TUI-specific settings
   - Separate from `opencode.json` (server config)
   - Schema validation
   - Environment variable for custom config path

8. **TUI Config Options**
   - `theme` - UI theme
   - `keybinds` - Custom keyboard shortcuts
   - `scroll_acceleration.enabled` - macOS-style scroll acceleration
   - `scroll_speed` - Scroll speed control
   - `diff_style` - Diff rendering style
   - `mouse` - Mouse capture toggle

### Current Wireframe-AI TUI Features

**Framework:** Rust + ratatui + Elm Architecture

**Existing Features:**
- Chat interface with role-based styling
- Slash command autocomplete (OpenCode-style)
- Leader key system (Ctrl+X)
- Multiple overlays (module status, logs, NATS flow, inspector, schema validator)
- Side panels (left: metrics, right: context)
- Info widgets (progress bars, spinners, status badges)
- Theme system (Default, Dark, Light)
- Adaptive layout for screen size
- F1-F4 for panel/overlay toggles

**Slash Commands (current):**
- `/help`, `/new`, `/sessions`, `/themes`, `/details`, `/compact`, `/export`, `/undo`, `/redo`, `/quit`

## OpenCode vs Wireframe-AI TUI Comparison

| Feature | OpenCode | Wireframe-AI TUI |
|---------|----------|------------------|
| Framework | TypeScript/Bun | Rust/ratatui |
| Command Palette | `ctrl+p` | Not implemented |
| Leader Key | `ctrl+x` | `ctrl+x` (implemented) |
| File References | `@` fuzzy search | Planned but not implemented |
| Bash Commands | `!` prefix | Planned but not implemented |
| Session Management | Git-based undo/redo | Basic session save/load |
| Theme System | JSON config | Enum-based in code |
| Keybinding Config | JSON config | Hardcoded in config |
| Side Panels | Not mentioned | Implemented (F1/F2) |
| Overlays | Not mentioned | Implemented (F3/F4) |
| Info Widgets | Not mentioned | Implemented (negative space) |
| Agent Switching | Tab key (build/plan) | Not implemented |

## OpenCode Patterns to Apply to Wireframe-AI

### Pattern 1: Command Palette (High Priority)
**OpenCode:** `ctrl+p` opens command palette for all customization
**Wireframe-AI:** Not implemented
**Implementation:**
- Add command palette overlay
- Searchable command list
- Quick access to all slash commands
- Settings management through palette
- Persist settings across sessions

### Pattern 2: Enhanced Slash Commands
**OpenCode:** 15+ slash commands with aliases
**Wireframe-AI:** 10 basic commands
**Enhancement:**
- Add command aliases (e.g., `/quit` → `/q`)
- Add missing commands: `/connect`, `/models`, `/thinking`, `/share`
- Implement command descriptions in autocomplete
- Add command grouping in help

### Pattern 3: File Reference System
**OpenCode:** `@` for fuzzy file search with auto-injection
**Wireframe-AI:** Planned but not implemented
**Implementation:**
- Integrate fuzzy search (nucleo or similar)
- Auto-inject file content into conversation
- Show file preview in autocomplete
- Support directory references

### Pattern 4: Bash Command Integration
**OpenCode:** `!` prefix for shell commands
**Wireframe-AI:** Planned but not implemented
**Implementation:**
- Parse `!` prefix in input
- Execute shell commands safely
- Display output as tool result
- Add command history

### Pattern 5: Git-Based Session Management
**OpenCode:** Undo/redo via Git integration
**Wireframe-AI:** Basic session save/load
**Enhancement:**
- Integrate Git for change tracking
- Implement undo/redo for file changes
- Add session branching
- Show diff in overlays

### Pattern 6: Configurable Keybindings
**OpenCode:** JSON-based keybinding config
**Wireframe-AI:** Hardcoded in config file
**Enhancement:**
- Move keybindings to JSON schema
- Allow runtime keybinding changes
- Add keybinding validation
- Support custom leader keys

### Pattern 7: Advanced TUI Config
**OpenCode:** Separate `tui.json` with schema validation
**Wireframe-AI:** Single config file
**Enhancement:**
- Split TUI config from server config
- Add JSON schema validation
- Support custom config paths via env var
- Add scroll acceleration
- Add diff style options

### Pattern 8: Agent Switching
**OpenCode:** Tab key to switch between build/plan agents
**Wireframe-AI:** Not implemented
**Implementation:**
- Add agent state to app
- Implement Tab key handler
- Show current agent in UI
- Add agent-specific behaviors

## Implementation Plan

### Phase 1: Command Palette (Foundation)
1. Design command palette data structure
2. Implement command palette overlay
3. Add searchable command list
4. Integrate with existing slash commands
5. Add palette to leader key system

### Phase 2: Enhanced Slash Commands
1. Add command alias system
2. Implement missing commands (/connect, /models, /thinking, /share)
3. Add command descriptions
4. Group commands in help dialog
5. Add command discovery in palette

### Phase 3: File Reference System
1. Integrate fuzzy search library
2. Implement `@` prefix parsing
3. Add file content injection
4. Create file preview in autocomplete
5. Add directory reference support

### Phase 4: Bash Command Integration
1. Implement `!` prefix parsing
2. Add safe shell command execution
3. Display command output as tool result
4. Add command history
5. Integrate with chat flow

### Phase 5: Git-Based Session Management
1. Integrate Git library
2. Track file changes via Git
3. Implement undo/redo for changes
4. Add session branching
5. Show diff in overlays

### Phase 6: Configurable Keybindings
1. Design keybinding JSON schema
2. Implement keybinding loader
3. Add runtime keybinding changes
4. Add keybinding validation
5. Support custom leader keys

### Phase 7: Advanced TUI Config
1. Split TUI config from server config
2. Add JSON schema validation
3. Support custom config paths
4. Add scroll acceleration
5. Add diff style options

### Phase 8: Agent Switching
1. Add agent state to app
2. Implement Tab key handler
3. Show current agent in UI
4. Add agent-specific behaviors
5. Integrate with command palette

## Decisions
- OpenCode uses TypeScript/Bun, Wireframe-AI uses Rust/ratatui - adapt patterns to Rust ecosystem
- Prioritize command palette as foundation for other features
- Maintain Elm Architecture
- Keep existing side panels and overlays (they're not in OpenCode but are valuable)
- Incremental implementation to avoid breaking changes
- Use existing Rust crates where possible (nucleo for fuzzy search, git2 for Git integration)

## Handoff Notes
- OpenCode's TUI is more command-focused than panel-focused
- Wireframe-AI's existing panels/overlays are valuable additions
- Focus on command palette and enhanced slash commands first
- File references and bash commands are high-value features
- Git-based session management is complex but valuable
- Config system needs architectural split
