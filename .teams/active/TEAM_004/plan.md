# Feature Implementation Plan - TEAM_004

## Feature Overview
Implement modern, clutter-free UI improvements for Wireframe-AI TUI:
- Info widgets using negative space (progress bars, spinners, status badges)
- Side panels (left for system metrics, right for context) with toggle commands
- Overlays (NATS inspector, schema validator) for deep inspection
- Adaptive layout for screen-size awareness

## Requirements
- Preserve existing chat, overlays, module management, NATS integration
- Maintain Elm Architecture
- Use ratatui for TUI components
- Minimal, modern design inspired by jcode and Pi
- No git commits per user request

## Design Decisions
- Hybrid adaptive layout approach (Option D from brainstorm)
- Incremental implementation: widgets → panels → overlays
- Info widgets occupy negative space only (jcode pattern)
- Side panels toggleable via slash commands (`/panel left`, `/panel right`)
- Overlays context-aware (NATS inspector, schema validator)
- Screen-size adaptation for accessibility

## Implementation Tasks

### Phase 1: Info Widgets (Low effort, high value)
1. Add widget state to AppState (progress, spinners, status badges)
2. Create widget rendering module (`widgets.rs`)
3. Implement progress bar widget for long operations
4. Implement spinner widget for async tasks
5. Implement status badge widget for module health
6. Add negative space calculation logic
7. Integrate widgets into main render loop

### Phase 2: Side Panels (Medium effort)
1. Add panel state to AppState (left_panel_visible, right_panel_visible)
2. Create panel rendering module (`panels.rs`)
3. Implement left panel for system metrics (NATS topics, module health)
4. Implement right panel for context (file previews, schema validation)
5. Add toggle commands (`/panel left`, `/panel right`, `/panel off`)
6. Add keyboard shortcuts for panel toggles
7. Integrate panels into main layout calculation

### Phase 3: Overlays (Medium effort)
1. Enhance existing overlay system
2. Implement NATS inspector overlay (message tracing, topic monitoring)
3. Implement schema validator overlay (envelope validation, schema checking)
4. Add overlay-specific keyboard shortcuts
5. Integrate with existing overlay toggle system

### Phase 4: Adaptive Layout (Low-Medium effort)
1. Add screen size detection to AppState
2. Implement layout adaptation logic (small vs medium vs large screens)
3. Adjust widget/panel visibility based on screen size
4. Add responsive constraints to layout calculations

## Testing Strategy
- Visual inspection of UI components
- Test panel toggle commands
- Test widget visibility in different screen sizes
- Verify overlay rendering
- Ensure existing functionality preserved

## Dependencies
- ratatui (already in use)
- crossterm (already in use)
- No new external dependencies

## Success Criteria
- Info widgets display in negative space without clutter
- Side panels toggle cleanly and show relevant information
- Overlays provide deep inspection capabilities
- Layout adapts to screen size appropriately
- All existing features remain functional
- Code compiles without errors
