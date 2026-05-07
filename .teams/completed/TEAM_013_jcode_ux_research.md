---
status: completed
created: 2026-05-06
completed: 2026-05-06
---

# TEAM_013 - JCODE UI Research and TUI Integration Planning

## Task
Research JCODE UI to understand its UX/UI patterns and plan how to apply them to the Wireframe-AI TUI.

## Progress
- [x] Research JCODE UI architecture and patterns
- [x] Document key UX/UI concepts from JCODE
- [x] Analyze current TUI implementation
- [x] Map JCODE patterns to TUI equivalents
- [x] Create implementation plan
- [x] Document design decisions

## Research Findings

### What is "JCODE"?

Based on codebase analysis, "JCODE" appears to be a design philosophy/inspiration rather than a specific tool or library. It's referenced alongside "Pi" as design inspiration for the TUI.

### Key JCODE-Inspired Patterns Already Implemented

1. **Negative Space Widgets** (widgets.rs)
   - Progress bars, spinners, status badges occupy only negative space
   - Located in bottom-right corner of screen
   - Non-intrusive, clutter-free design

2. **Side Panels** (panels.rs)
   - Left panel: System metrics (NATS topics, module health)
   - Right panel: Context (session info, message count)
   - Toggleable via keyboard shortcuts (F1, F2)
   - Adaptive layout - hides on small screens

3. **Minimal, Modern Design**
   - Clean borders and typography
   - Color-coded status indicators
   - Keyboard shortcut hints in panel titles

### Current TUI Architecture

**Framework:** ratatui with Elm Architecture
**Structure:**
- `main.rs` - Entry point, event loop
- `app.rs` - Application state and logic (Update function)
- `view.rs` - Rendering logic (View function)
- `model.rs` - Data structures
- `widgets.rs` - Info widgets (jcode-inspired)
- `panels.rs` - Side panels
- `theme.rs` - Color palettes and styling
- `command.rs` - Command parsing
- `config.rs` - Configuration

**Key Features:**
- Chat interface with role-based styling
- Slash command autocomplete (OpenCode-style)
- Multiple overlays (module status, logs, NATS flow, inspector, schema validator)
- Leader key system (Ctrl+X) for keybindings
- Theme system (Default, Dark, Light)
- Adaptive layout for screen size

## JCODE Design Principles (Interpreted)

Based on existing implementation patterns:

1. **Negative Space Utilization**
   - Widgets occupy unused screen space
   - Non-intrusive status indicators
   - Progressive disclosure - show only what's needed

2. **Toggleable Panels**
   - Side panels for supplementary information
   - Keyboard shortcuts for quick access
   - Adaptive visibility based on screen size

3. **Minimal Visual Noise**
   - Clean borders and typography
   - Color-coded status without excessive decoration
   - Keyboard hints integrated into UI

4. **Context-Aware Overlays**
   - Deep inspection tools (NATS inspector, schema validator)
   - Modal overlays for focused tasks
   - Clear exit mechanisms

## Mapping JCODE Patterns to TUI Enhancements

### Pattern 1: Enhanced Negative Space Usage
**Current:** Widgets in bottom-right corner
**Enhancement:**
- Add widget priority system (critical info first)
- Smart widget sizing based on available space
- Widget grouping by category (status, progress, info)
- Fade-out animation for inactive widgets

### Pattern 2: Improved Panel System
**Current:** Left/right panels with basic info
**Enhancement:**
- Draggable panel resizing
- Panel pinning/unpinning
- Panel content filtering
- Panel history/quick access
- Multiple panel layouts (presets)

### Pattern 3: Keyboard-First Navigation
**Current:** F-keys for toggles, Ctrl+X leader key
**Enhancement:**
- Comprehensive keybinding system
- Keybinding hints in context menus
- Customizable keybindings
- Vi-style navigation modes
- Command palette for keybinding discovery

### Pattern 4: Context-Aware UI
**Current:** Static overlays and panels
**Enhancement:**
- Dynamic content based on current task
- Smart panel suggestions
- Context-sensitive autocomplete
- Workflow-specific layouts

### Pattern 5: Visual Hierarchy
**Current:** Basic color coding
**Enhancement:**
- Consistent visual language
- Importance-based sizing
- Subtle animations for state changes
- Clear focus indicators
- Status-based color themes

## Implementation Plan

### Phase 1: Widget System Enhancement
1. Implement widget priority queue
2. Add smart widget sizing algorithm
3. Create widget grouping system
4. Add widget lifecycle management

### Phase 2: Panel System Upgrade
1. Implement panel resizing
2. Add panel presets/layouts
3. Create panel content filtering
4. Add panel history system

### Phase 3: Navigation Improvements
1. Expand keybinding system
2. Add keybinding discovery UI
3. Implement customizable keybindings
4. Add Vi-style navigation modes

### Phase 4: Context Awareness
1. Implement dynamic content loading
2. Add smart panel suggestions
3. Create workflow-specific layouts
4. Add context-sensitive autocomplete

### Phase 5: Visual Polish
1. Implement consistent visual language
2. Add subtle animations
3. Create focus indicator system
4. Add status-based theming

## Decisions
- JCODE is a design philosophy, not a specific tool
- Focus on patterns already present in codebase
- Incremental enhancement of existing features
- Maintain Elm Architecture
- Preserve ratatui framework

## Handoff Notes
- Current TUI already implements core JCODE patterns
- Focus on enhancing existing patterns rather than adding new paradigms
- Maintain backward compatibility with existing keybindings
- Keep performance in mind for widget/panel updates
