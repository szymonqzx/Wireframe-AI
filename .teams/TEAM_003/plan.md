# Implementation Plan - TEAM_003: OpenCode UX Integration

## Feature Overview

Implement OpenCode TUI UX patterns and visual appearance in the existing ratatui-based TUI (`tools/tui-chat/`) while preserving all current functionality. This hybrid approach adopts OpenCode's design system systematically without copying code.

## Requirements

### Functional Requirements
- Slash command system (`/help`, `/new`, `/sessions`, `/themes`, `/details`, `/compact`, `/export`, `/undo`, `/redo`)
- File reference syntax using `@` for fuzzy search
- Bash command execution using `!` prefix
- ctrl+x leader key system for keybinds
- Command palette UI overlay
- OpenCode color palette and visual styling
- OpenCode layout (chat panel, status bar, command palette)
- Undo/redo with Git integration
- Session management (list, switch, persist)
- Theme switching
- Export to Markdown
- Preserve existing NATS integration
- Preserve existing module management overlays
- Preserve existing Elm Architecture

### Non-Functional Requirements
- Pure Rust implementation (no Zig/FFI)
- Maintain existing ratatui framework
- Cross-platform (Windows, macOS, Linux)
- Low resource footprint
- Extensible design system for future evolution

## Design Decisions

### Command System Architecture
- **Command Parser**: New module `src/command.rs` to parse slash commands, `@` file refs, `!` bash commands
- **Command Registry**: Trait-based system for registering new commands
- **Leader Key**: ctrl+x as leader, followed by single character (e.g., `ctrl+x n` for `/new`)
- **Command Palette**: Overlay UI for fuzzy searching commands (similar to Ctrl+P but for commands)

### Visual Design System
- **Theme Module**: New module `src/theme.rs` with OpenCode color palette
- **Layout System**: Refactor `src/view.rs` to match OpenCode's panel structure
- **Widget Styles**: Consistent styling across all components using the theme system

### Feature Integration
- **Undo/Redo**: Use git2 crate for Git operations
- **Sessions**: Extend existing session management with list/switch UI
- **Export**: Add Markdown export functionality
- **Themes**: Predefined theme presets matching OpenCode

## Implementation Tasks

### Phase 1: Command System

#### Task 1.1: Create Command Parser Module
- [ ] Create `src/command.rs`
- [ ] Define `Command` enum (SlashCommand, FileReference, BashCommand)
- [ ] Implement parser for `/` prefix (slash commands)
- [ ] Implement parser for `@` prefix (file references)
- [ ] Implement parser for `!` prefix (bash commands)
- [ ] Add unit tests for parser

#### Task 1.2: Implement Slash Commands
- [ ] Define command registry trait
- [ ] Implement `/help` command (show available commands)
- [ ] Implement `/new` command (clear session, start new)
- [ ] Implement `/sessions` command (list and switch sessions)
- [ ] Implement `/themes` command (list and switch themes)
- [ ] Implement `/details` command (toggle tool execution details)
- [ ] Implement `/compact` command (summarize session)
- [ ] Implement `/export` command (export to Markdown)
- [ ] Implement `/undo` command (undo last message + Git revert)
- [ ] Implement `/redo` command (redo last undo + Git restore)

#### Task 1.3: Implement Leader Key System
- [ ] Modify event handler in `src/app.rs` to detect ctrl+x
- [ ] Implement leader key state machine (wait for next key after ctrl+x)
- [ ] Map leader key combinations to commands (ctrl+x n → /new, etc.)
- [ ] Add visual indicator when leader key is active

#### Task 1.4: Implement File Reference (@) and Bash Command (!)
- [ ] Integrate `@` parser with existing fuzzy search (nucleo)
- [ ] Display file content in chat when referenced
- [ ] Implement `!` command execution via tokio::process::Command
- [ ] Display bash command output in chat
- [ ] Handle errors gracefully

#### Task 1.5: Create Command Palette Overlay
- [ ] Design command palette UI overlay
- [ ] Implement fuzzy search for commands
- [ ] Add keybinding to open command palette (default: Ctrl+K or Ctrl+P variant)
- [ ] Integrate with existing overlay system

### Phase 2: Visual Design System

#### Task 2.1: Extract OpenCode Color Palette
- [ ] Research OpenCode's color scheme from screenshots/docs
- [ ] Define color constants in `src/theme.rs`
- [ ] Create theme struct with foreground, background, accent colors
- [ ] Add predefined themes (default, dark, light)

#### Task 2.2: Implement Theme System
- [ ] Create theme trait
- [ ] Apply theme to all ratatui widgets
- [ ] Add theme switching logic
- [ ] Persist theme selection in config

#### Task 2.3: Refactor Layout to Match OpenCode
- [ ] Analyze OpenCode's panel structure (chat left, status right, etc.)
- [ ] Modify `src/view.rs` layout to match
- [ ] Add status bar at bottom
- [ ] Ensure responsive layout on resize

#### Task 2.4: Apply Consistent Styling
- [ ] Define border styles (single, double, rounded)
- [ ] Define spacing and padding constants
- [ ] Apply consistent typography
- [ ] Ensure all overlays follow same styling

### Phase 3: Feature Parity

#### Task 3.1: Implement Undo/Redo with Git
- [ ] Add git2 dependency to Cargo.toml
- [ ] Implement Git repository detection
- [ ] Implement undo logic (revert last message + git checkout)
- [ ] Implement redo logic (restore message + git reset)
- [ ] Handle non-Git repositories gracefully
- [ ] Add error handling for Git operations

#### Task 3.2: Enhance Session Management
- [ ] Extend existing session persistence
- [ ] Implement session list UI overlay
- [ ] Add session switching functionality
- [ ] Implement session deletion
- [ ] Add session metadata (created, last modified)

#### Task 3.3: Implement Theme Switching
- [ ] Create theme selection UI
- [ ] Add theme preview
- [ ] Implement hot-reload of themes
- [ ] Add custom theme support (via config)

#### Task 3.4: Implement Export to Markdown
- [ ] Add export logic to convert chat history to Markdown
- [ ] Preserve code blocks with syntax highlighting
- [ ] Include metadata (date, session ID)
- [ ] Open in default editor after export

### Phase 4: Polish and Cross-Platform Testing

#### Task 4.1: Refine Animations and Transitions
- [ ] Add smooth transitions for overlays
- [ ] Implement command palette animations
- [ ] Add loading indicators for long operations

#### Task 4.2: Add Keyboard Shortcuts Documentation
- [ ] Implement `/help` command with all shortcuts
- [ ] Add in-app keybinding hints
- [ ] Create documentation file

#### Task 4.3: Cross-Platform Testing
- [ ] Test on Windows (current)
- [ ] Test on macOS (if available)
- [ ] Test on Linux (if available)
- [ ] Fix platform-specific issues

#### Task 4.4: Performance Optimization
- [ ] Profile rendering performance
- [ ] Optimize command palette fuzzy search
- [ ] Reduce memory footprint
- [ ] Ensure <100ms render time

## Testing Strategy

### Unit Tests
- Command parser tests (all syntax variants)
- Theme system tests
- Git integration tests

### Integration Tests
- Command execution end-to-end
- Session management workflows
- Undo/redo with Git
- Export functionality

### Manual Testing
- Visual inspection against OpenCode screenshots
- Keyboard shortcut testing
- Cross-platform verification

## Dependencies

### New Dependencies
- `git2` - Git operations for undo/redo
- `syntect` - Already present, ensure proper configuration

### Existing Dependencies (Reuse)
- `ratatui` - TUI framework
- `crossterm` - Terminal handling
- `tokio` - Async runtime
- `nucleo` - Fuzzy matching
- `serde` / `toml` - Configuration

## Success Criteria

- All slash commands work correctly
- File references (`@`) and bash commands (`!`) function as expected
- Leader key system (ctrl+x) works smoothly
- Visual appearance matches OpenCode (colors, layout, styling)
- Undo/redo with Git integration works
- Session management enhanced
- Theme switching functional
- Export to Markdown works
- All existing functionality preserved (NATS, module management, overlays)
- Cross-platform compatibility maintained
- Performance remains snappy (<100ms render time)

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Git dependency complexity | Medium | Make Git optional, fallback to message-only undo/redo |
| Visual matching difficulty | Medium | Use screenshots as reference, iterate based on feedback |
| Leader key conflicts | Low | Make leader key configurable |
| Performance regression | Medium | Profile early, optimize critical paths |
| Breaking existing features | High | Test existing overlays and NATS integration thoroughly |

## Implementation Order

1. **Phase 1.1**: Command parser (foundation)
2. **Phase 1.2**: Slash commands (core functionality)
3. **Phase 1.3**: Leader key system (UX pattern)
4. **Phase 1.4**: File refs and bash commands (integration)
5. **Phase 1.5**: Command palette (UI)
6. **Phase 2.1-2.2**: Theme system (visual foundation)
7. **Phase 2.3-2.4**: Layout and styling (visual parity)
8. **Phase 3.1**: Undo/redo (feature parity)
9. **Phase 3.2**: Session management (enhancement)
10. **Phase 3.3-3.4**: Themes and export (completeness)
11. **Phase 4**: Polish and testing (quality)

## Notes

- All code changes must include `// TEAM_003:` comments
- Preserve existing Elm Architecture (Model-Update-View)
- Maintain backward compatibility with existing config
- Document breaking changes if any
- Update existing PLAN-tui-chat-extensible.md if this supersedes it
