# TUI Enhancement Plan

## Goal
Enhance the Wireframe-AI TUI for reliability, architecture, UI/UX polish, and correct logic to support end-user chat, developer tools, and debugging workflows with cross-platform compatibility, minimal dependencies, high performance, and rich features.

## Analysis Summary

### Critical Issues Found
**Reliability:**
- No error recovery in async operations
- Placeholder data in panels (no real NATS integration)
- No graceful shutdown or reconnection logic
- Missing bounds checking for array access
- No input validation

**Architecture:**
- AppState is bloated (20+ fields in one struct)
- Theme stored as string instead of enum
- No dependency injection for NATS client
- Hardcoded constants should be configurable
- Plugin system defined but unused

**UI/UX:**
- No keyboard shortcut hints visible
- No visual feedback for leader key activation
- Autocomplete state exists but not rendered
- No scroll indicators in chat
- No message timestamps visible
- Spinners not animated (update_spinners never called)

**Logic:**
- Command palette deprecated but state remains
- Fuzzy/symbol search incomplete
- Module management functions are stubs
- No actual NATS integration
- Theme switching not implemented
- Session management incomplete

## Tasks

### Phase 1: Reliability Foundation
- [ ] Add error recovery with graceful degradation in async operations → Verify: Run TUI, trigger errors, see graceful fallback
- [ ] Implement NATS connection management with auto-reconnect → Verify: Start NATS, stop it, watch TUI reconnect
- [ ] Add bounds checking for all array/vector access → Verify: Run clippy, fix all warnings
- [ ] Implement graceful shutdown on SIGTERM/SIGINT → Verify: Send SIGTERM, verify clean exit
- [ ] Add input validation for user commands → Verify: Test edge cases, see validation errors

### Phase 2: Architecture Cleanup
- [ ] Split AppState into focused structs (ChatState, OverlayState, PanelState) → Verify: Build passes, tests pass
- [ ] Convert theme field from String to Theme enum in AppState → Verify: Theme switching works
- [ ] Extract hardcoded constants to config → Verify: Config file changes panel widths
- [ ] Implement dependency injection for NATS client → Verify: Can mock NATS in tests
- [ ] Remove deprecated command palette state → Verify: Build passes, no unused fields

### Phase 3: UI/UX Polish
- [ ] Add keyboard shortcut hints to overlay titles → Verify: See "[F1]" in panel titles
- [ ] Implement leader key visual feedback (status indicator) → Verify: Press Ctrl+X, see indicator
- [ ] Render autocomplete dropdown when typing slash commands → Verify: Type "/", see command list
- [ ] Add scroll indicators to chat area → Verify: Scroll past top/bottom, see indicators
- [ ] Show message timestamps in chat → Verify: Messages display timestamps
- [ ] Wire up spinner animation in tick loop → Verify: Spinners animate in real-time

### Phase 4: Logic Implementation
- [ ] Implement actual NATS subscription/publishing → Verify: Messages flow through NATS
- [ ] Implement theme switching command (/themes) → Verify: Type /themes, switch themes
- [ ] Complete session save/load functionality → Verify: Save session, reload it
- [ ] Implement module start/stop/restart with process management → Verify: Start module, see it running
- [ ] Implement fuzzy file search with nucleo → Verify: Type @file, see matches

### Phase 5: Testing & Verification
- [ ] Add unit tests for critical paths (error handling, bounds checking) → Verify: `cargo test` passes
- [ ] Add integration test for NATS reconnection → Verify: Test passes with NATS restart
- [ ] Run clippy and fix all warnings → Verify: `cargo clippy` clean
- [ ] Run cargo build --release → Verify: Release build succeeds
- [ ] Manual testing: Run full workflow (chat, overlays, themes) → Verify: All features work end-to-end

## Done When
- [ ] All phases completed with verification steps passing
- [ ] No clippy warnings
- [ ] Release build succeeds
- [ ] Manual end-to-end testing passes
- [ ] Code follows project clean code standards

## Notes
- Phase 1 and 2 are blocking (must complete before UI/UX work)
- Phases 3 and 4 can be parallelized per feature
- Maintain backward compatibility with existing config files
- All changes must be cross-platform (Windows/Linux/macOS)
- No new dependencies beyond current Cargo.toml