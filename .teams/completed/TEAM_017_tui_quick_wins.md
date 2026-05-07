---
status: completed
created: 2026-05-06
completed: 2026-05-06
---

# TEAM_017 - TUI Quick Wins Implementation

## Task
Implement Phase 1 Quick Wins from AI Agents TUI research: session persistence, multi-line input, live status line, and custom instructions.

## Progress
- [x] Session persistence (auto-save, resume picker)
- [x] Multi-line input (Shift+Enter for newline)
- [x] Live status line (model, mode, active modules)
- [x] Custom instructions (AGENTS.md, .wireframe/instructions.md)

## Implementation Details

### 1. Session Persistence
- Add session auto-save to `.wireframe/sessions/`
- Implement `/resume` command with picker
- Add session list command
- Files: model.rs (add session fields), app.rs (save/load logic), command.rs (new commands)

### 2. Multi-line Input
- Modify input handling to support Shift+Enter
- Update input mode to track multi-line state
- Add visual indicator for multi-line mode
- Files: app.rs (input handling), view.rs (input rendering)

### 3. Live Status Line
- Add status line area to layout
- Display model, mode, active modules count
- Update in real-time
- Files: view.rs (layout), model.rs (status fields)

### 4. Custom Instructions
- Load AGENTS.md at startup if exists
- Load `.wireframe/instructions.md` if exists
- Append to system prompt
- Files: app.rs (startup logic), model.rs (instructions field)

## Dependencies
- TEAM_015: OpenCode TUI implementation (command palette, slash commands)
- TEAM_002: Session management (session_id, session_path)
- TEAM_004: Overlay system and layout

## Handoff Notes
- Building on TEAM_016 research findings
- Focus on high-impact, low-complexity features
- Test each feature before moving to next
