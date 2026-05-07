# TEAM_002 - TUI Chat with Extensible Overlay Architecture

**Status:** active
**Created:** 2026-05-06
**Completed:** 

## Task

Build a chat-first TUI for Wireframe-AI with toggleable operational overlays (module status, logs, NATS flow). Architecture designed with extensibility in mind to support future plugin system. Replaces TEAM_001 TUI module runner.

## Progress

- [ ] Phase 0: Foundation (crate setup, traits, Elm skeleton)
- [ ] Phase 1: Chat Interface (messages, streaming, search)
- [ ] Phase 2: Module Management (process spawning, status tracking)
- [ ] Phase 3: Overlays (overlay framework, status/logs/NATS overlays)
- [ ] Phase 4: NATS Integration (real-time updates)
- [ ] Phase 5: Polish & Configuration (TOML, CLI, theming)
- [ ] Phase 6: Migration & Cleanup (replace TEAM_001, testing)
- [ ] Phase 7: Extensibility Verification (plugin proof-of-concepts)

## Decisions

- Chat-first design with overlay system (Option D from brainstorm)
- Extensibility via ViewPlugin and OverlayPlugin traits (foundation for Option E)
- Direct NATS subscription for real-time module status updates
- Replace TEAM_001 entirely (not build on top)
- Configuration via TOML + CLI args (modules, keybindings, UI layout)
- Elm Architecture (Model-Update-View) for state management

## Handoff Notes

- Plan file: `docs/PLAN-tui-chat-extensible.md`
- New crate location: `tools/tui-chat/`
- Key extensibility traits defined in Phase 0
- NATS integration optional via config
- Cross-platform testing critical
