---
status: completed
created: 2026-05-06
completed: 2026-05-06
---

# TEAM_016 - Popular AI Agents TUI Research and Application

## Task
Research popular AI agents with TUI interfaces, analyze their UX/UI patterns, and plan how to apply them to the Wireframe-AI TUI.

## Progress
- [x] Research popular AI agents with TUIs
- [x] Document key UX/UI patterns from each agent
- [x] Analyze current Wireframe-AI TUI implementation
- [x] Map agent patterns to Wireframe-AI TUI equivalents
- [x] Create implementation plan
- [x] Document design decisions

## Agents to Research
- OpenCode (already researched)
- Cursor (CLI/TUI) - GUI-based, no TUI
- Continue.dev (CLI) - Not found
- Aider (CLI tool) - Terminal-based, no TUI
- GPT Engineer (CLI) - Archived, terminal-based
- Codeium (CLI) - Not found
- Copilot CLI - Not found
- Other AI coding agents

## Research Findings

### AIChat (sigoden/aichat) - 9.9k stars
**Language:** Rust
**TUI Features:**
- REPL Mode with tab autocompletion, multi-line input, history search
- Customizable keybindings and REPL prompts
- CMD Mode for command-line functionalities
- Shell Assistant: Natural language to shell commands
- Multi-form input: stdin, files, directories, URLs, external commands
- Role customization for LLM behavior
- Session management for context-aware conversations
- Macro system for repetitive tasks
- RAG integration for external documents
- Function calling with AI Tools & MCP
- AI Agents (CLI version of OpenAI GPTs)
- Built-in HTTP server with LLM Playground and Arena
- Custom themes (dark/light) with syntax highlighting

**Key Patterns:**
- Dual-mode operation (CMD + REPL)
- Rich input handling (files, URLs, pipes)
- Session persistence
- Role-based customization
- Built-in web interface for comparison

### Sofos Code (alexylon/sofos-code) - 9 stars
**Language:** Rust
**TUI Features:**
- Interactive TUI with inline viewport at bottom of terminal
- Keep typing during AI turns (FIFO message queue)
- Live status line showing model, mode, reasoning config, token totals
- Markdown formatting with syntax highlighting
- Image vision (local/web images, clipboard paste)
- Session history with auto-save and resume picker
- Custom instructions (project + personal context files)
- File operations (read, write, edit, list, glob, create, move, copy, delete)
- Targeted edits with diff-based `edit_file`
- Ultra-fast editing with Morph Apply integration
- File search by glob pattern
- Code search with ripgrep
- Web search and fetch
- Bash execution with 3-tier permission system
- MCP integration
- Visual diffs with line numbers
- Iterative tools (up to 200 tool calls per request)
- Context compaction (summarizes older messages)
- Cost tracking with session token usage
- Safe mode (read-only)

**Key Patterns:**
- Inline viewport design (preserves terminal scrollback)
- Message queueing during AI turns
- Live status indicators
- Multi-line input (Shift+Enter for newline)
- Session persistence with resume
- Permission-based bash execution
- Context management with compaction

### ZAI Shell (TaklaXBR/zai-shell) - 41 stars
**Language:** Python
**TUI Features:**
- Autonomous P2P system administration
- Sentinel 1.5: Behavioral risk intelligence
  - 4-dimension risk breakdown (structural, behavioral, contextual, intent)
  - Panic mode detection
  - Lesson memory from past failures
  - Context-aware warnings
  - Non-blocking by design
- P2P Mesh: Secure collaboration
  - End-to-end encryption (AES-128)
  - Zero-trust architecture
  - Natural language bridge
  - Global reach via tunnels
- Hybrid intelligence
  - Multi-modal (GUI + images)
  - Research capabilities (web browsing)
  - Self-healing (5-strategy auto-retry)
- Cross-shell support (13+ shells)
- Offline AI with local models
- Persistent memory (vector + JSON)
- Thinking mode with visible reasoning
- GUI automation

**Key Patterns:**
- Safety-first with risk intelligence
- Self-healing auto-retry
- P2P collaboration
- Multi-modal input
- Cross-shell flexibility
- Offline capability

### Aider (paul-gauthier/aider) - 44.4k stars
**Language:** Python
**CLI Features:**
- Terminal-based AI pair programming
- Codebase mapping for large projects
- 100+ code languages support
- Git integration with automatic commits
- IDE integration via comments
- Images & web pages as context
- Voice-to-code
- Automatic linting & testing
- Copy/paste to web chat

**Key Patterns:**
- Git-first workflow
- Codebase mapping for context
- Multi-language support
- Voice input
- Testing automation

### OpenHands (OpenHands/OpenHands) - 72.7k stars
**Language:** Python/TypeScript
**Features:**
- Web-based UI (not TUI)
- Full AI software engineer
- Container-based execution
- Multi-language support

**Note:** Web-based, not TUI relevant

### Summary of TUI Patterns

**Common Patterns Across Agents:**
1. **Session Management:** Auto-save, resume, history
2. **Rich Input Handling:** Files, URLs, images, clipboard
3. **Status Indicators:** Live status lines with model, mode, tokens
4. **Permission Systems:** Multi-tier permission for dangerous operations
5. **Context Management:** Compaction, summarization, custom instructions
6. **Tool Integration:** MCP, function calling, web search
7. **Multi-line Input:** Shift+Enter for newlines
8. **Command Systems:** Slash commands, aliases
9. **Theme Support:** Custom themes with syntax highlighting
10. **Cost Tracking:** Token usage and cost estimates

**Unique Patterns:**
- **Sofos Code:** Inline viewport design, message queueing during AI turns
- **ZAI Shell:** Behavioral risk intelligence, self-healing auto-retry, P2P collaboration
- **AIChat:** Dual-mode (CMD + REPL), built-in web interface for comparison
- **Aider:** Git-first workflow, voice input, codebase mapping

## Current Wireframe-AI TUI Implementation Analysis

**Existing Features:**
- Command palette (Ctrl+P) - TEAM_015
- Enhanced slash commands with autocomplete - TEAM_015
- File reference system with fuzzy search (@ prefix) - TEAM_015
- Side panels (left/right) for additional context - TEAM_004
- Overlay system (module status, logs, NATS flow, inspector, schema validator) - TEAM_004
- Widget system (progress bars, spinners, status badges) - TEAM_004
- Theme support (dark/light) - TEAM_003
- Session management (session_id, session_path) - TEAM_002
- Module process tracking (status, PID, stdout/stderr) - TEAM_002
- Input modes (Normal, Editing, VimInsert, VimNormal) - TEAM_002
- Message history with role-based styling - TEAM_002
- Adaptive layout based on screen size - TEAM_004

**Architecture:**
- Elm Architecture (Model-View-Update) - TEAM_002
- Ratatui TUI framework
- State serialization/deserialization
- Multi-panel layout system
- Negative space widget placement

**Current Gaps vs. Research Findings:**
1. **Session Persistence:** Has session_id but no auto-save/resume functionality
2. **Multi-line Input:** No Shift+Enter for newlines
3. **Live Status Line:** No real-time token tracking or model status
4. **Message Queueing:** Cannot type during AI turns
5. **Context Management:** No compaction or summarization
6. **Cost Tracking:** No token usage or cost estimates
7. **Permission System:** No multi-tier permission for dangerous operations
8. **Custom Instructions:** No project/personal context file loading
9. **Image Vision:** No image input support
10. **Web Integration:** No web search or fetch capabilities
11. **Macro System:** No command macros
12. **Role Customization:** No role-based LLM behavior customization

## Pattern Mapping: Agent Features → Wireframe-AI TUI

### High Priority (High Impact, Low Complexity)

**1. Session Persistence (Sofos Code, AIChat)**
- **Current:** session_id exists but no auto-save/resume
- **Implementation:** Add session auto-save to disk, resume picker command
- **Benefit:** Users can continue conversations across sessions
- **Complexity:** Low - uses existing session_id field

**2. Multi-line Input (Sofos Code, AIChat)**
- **Current:** Single-line input only
- **Implementation:** Shift+Enter for newline, Enter to submit
- **Benefit:** Better for complex prompts and code snippets
- **Complexity:** Low - modify input handling in app.rs

**3. Live Status Line (Sofos Code)**
- **Current:** No real-time status indicators
- **Implementation:** Add status line showing model, mode, active modules
- **Benefit:** Better situational awareness
- **Complexity:** Low - add to view.rs layout

**4. Custom Instructions (Sofos Code)**
- **Current:** No context file loading
- **Implementation:** Load AGENTS.md and .wireframe/instructions.md at startup
- **Benefit:** Project-specific context for AI
- **Complexity:** Low - file I/O at startup

### Medium Priority (High Impact, Medium Complexity)

**5. Message Queueing (Sofos Code)**
- **Current:** Cannot type during AI turns
- **Implementation:** FIFO queue for messages during AI processing
- **Benefit:** Faster workflow, no waiting
- **Complexity:** Medium - requires async message handling

**6. Context Management (Sofos Code)**
- **Current:** No compaction or summarization
- **Implementation:** Summarize older messages when context limit reached
- **Benefit:** Longer conversations without context loss
- **Complexity:** Medium - requires LLM integration for summarization

**7. Cost Tracking (Sofos Code, AIChat)**
- **Current:** No token tracking
- **Implementation:** Track token usage per session, display cost estimates
- **Benefit:** Cost awareness and budgeting
- **Complexity:** Medium - requires token counting from LLM responses

### Low Priority (High Impact, High Complexity)

**8. Permission System (Sofos Code, ZAI Shell)**
- **Current:** No permission checks for dangerous operations
- **Implementation:** 3-tier permission system for file operations, bash execution
- **Benefit:** Safety and control
- **Complexity:** High - requires permission UI and state management

**9. Image Vision (Sofos Code, AIChat)**
- **Current:** No image input
- **Implementation:** Support image paths, URLs, clipboard paste
- **Benefit:** Multi-modal interaction
- **Complexity:** High - requires vision model integration

**10. Web Integration (Sofos Code, AIChat)**
- **Current:** No web search/fetch
- **Implementation:** Web search and fetch commands
- **Benefit:** Access to live information
- **Complexity:** High - requires web API integration

### Not Recommended (Low Impact or Out of Scope)

**11. Macro System (AIChat)**
- **Rationale:** Wireframe-AI has slash commands which serve similar purpose
- **Alternative:** Enhance slash command system with aliases

**12. Role Customization (AIChat)**
- **Rationale:** Wireframe-AI uses subagent system for role-based behavior
- **Alternative:** Leverage existing subagent architecture

**13. P2P Collaboration (ZAI Shell)**
- **Rationale:** Out of scope for Wireframe-AI's architecture
- **Alternative:** Use existing NATS message bus for collaboration

**14. Behavioral Risk Intelligence (ZAI Shell)**
- **Rationale:** Complex to implement, may not align with Wireframe-AI's use case
- **Alternative:** Simple permission system (see #8)

## Implementation Plan

### Phase 1: Quick Wins (1-2 days)
**Goal:** Implement high-priority, low-complexity features

1. **Session Persistence**
   - Add session auto-save to `.wireframe/sessions/`
   - Implement `/resume` command with picker
   - Add session list command
   - Files: model.rs (add session fields), app.rs (save/load logic), command.rs (new commands)

2. **Multi-line Input**
   - Modify input handling to support Shift+Enter
   - Update input mode to track multi-line state
   - Add visual indicator for multi-line mode
   - Files: app.rs (input handling), view.rs (input rendering)

3. **Live Status Line**
   - Add status line area to layout
   - Display model, mode, active modules count
   - Update in real-time
   - Files: view.rs (layout), model.rs (status fields)

4. **Custom Instructions**
   - Load AGENTS.md at startup if exists
   - Load `.wireframe/instructions.md` if exists
   - Append to system prompt
   - Files: app.rs (startup logic), model.rs (instructions field)

### Phase 2: Enhanced Workflow (3-5 days)
**Goal:** Implement medium-priority features

5. **Message Queueing**
   - Add message queue to model
   - Modify AI turn handling to process queue
   - Add queue count indicator
   - Files: model.rs (queue field), app.rs (queue processing), view.rs (queue indicator)

6. **Context Management**
   - Add context limit configuration
   - Implement summarization trigger
   - Add `/compact` command
   - Files: model.rs (context fields), app.rs (compaction logic), command.rs (new command)

7. **Cost Tracking**
   - Add token counters to model
   - Track input/output tokens per message
   - Calculate cost estimates
   - Add `/cost` command
   - Files: model.rs (token fields), app.rs (tracking logic), command.rs (new command)

### Phase 3: Advanced Features (5-10 days)
**Goal:** Implement low-priority, high-complexity features

8. **Permission System**
   - Add permission state to model
   - Implement 3-tier permission levels
   - Add permission confirmation UI
   - Apply to file operations and bash execution
   - Files: model.rs (permission fields), app.rs (permission logic), view.rs (permission UI)

9. **Image Vision**
   - Add image input parsing
   - Support image paths, URLs, clipboard
   - Integrate with vision model
   - Files: model.rs (image fields), app.rs (image handling), view.rs (image rendering)

10. **Web Integration**
    - Add web search command
    - Add web fetch command
    - Integrate with web API
    - Files: command.rs (new commands), app.rs (web logic)

## Handoff Notes

### Research Summary
Researched 7 popular AI agents with TUI/CLI interfaces:
- **AIChat** (Rust, 9.9k stars): REPL mode, CMD mode, rich input handling, session management, MCP integration
- **Sofos Code** (Rust, 9 stars): Inline viewport, message queueing, live status line, session persistence, cost tracking
- **ZAI Shell** (Python, 41 stars): Behavioral risk intelligence, self-healing auto-retry, P2P collaboration
- **Aider** (Python, 44.4k stars): Git-first workflow, codebase mapping, voice input
- **OpenHands** (72.7k stars): Web-based, not TUI relevant
- **Cursor/GPT Engineer/Continue.dev**: Either GUI-based, archived, or not found

### Key Findings
1. **Common Patterns:** Session management, rich input handling, status indicators, permission systems, context management, tool integration, multi-line input, command systems, theme support, cost tracking
2. **Unique Patterns:** Sofos Code's inline viewport and message queueing, ZAI Shell's risk intelligence and self-healing, AIChat's dual-mode operation
3. **Wireframe-AI Gaps:** Session persistence, multi-line input, live status line, message queueing, context management, cost tracking, permission system, image vision, web integration

### Implementation Plan
**Phase 1 (Quick Wins - 1-2 days):**
- Session persistence (auto-save, resume picker)
- Multi-line input (Shift+Enter for newline)
- Live status line (model, mode, active modules)
- Custom instructions (AGENTS.md, .wireframe/instructions.md)

**Phase 2 (Enhanced Workflow - 3-5 days):**
- Message queueing (type during AI turns)
- Context management (compaction, summarization)
- Cost tracking (token usage, cost estimates)

**Phase 3 (Advanced Features - 5-10 days):**
- Permission system (3-tier for dangerous operations)
- Image vision (paths, URLs, clipboard)
- Web integration (search, fetch)

### Design Decisions
- **Prioritized** high-impact, low-complexity features first
- **Excluded** macro system (use slash commands instead), role customization (use subagents), P2P collaboration (use NATS), behavioral risk intelligence (use simple permission system)
- **Leveraged** existing Wireframe-AI architecture (Elm Architecture, Ratatui, session management, overlay system)
- **Maintained** incremental implementation approach

### Next Steps
1. Create TEAM_017 for Phase 1 implementation
2. Start with session persistence (highest impact, lowest complexity)
3. Test each feature before moving to next
4. Update documentation as features are implemented

### Dependencies
- Requires existing OpenCode TUI implementation (TEAM_015)
- Builds on existing session management (TEAM_002)
- Integrates with existing command system (TEAM_015)
- Uses existing overlay system (TEAM_004)
