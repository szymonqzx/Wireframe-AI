---
status: active
created: 2026-05-06
---

# TEAM_019 - TUI NATS Integration

## Task
Implement actual agent interaction in the TUI by integrating with the Wireframe-AI NATS message bus. The TUI should work alongside the existing interface module as an alternative frontend.

## Requirements
- NATS connection to message bus
- Publish `task.submitted` messages
- Subscribe to `task.complete` for responses
- Configurable agent flow (orchestrator vs direct)
- Display agent responses in chat UI
- Handle errors and timeouts gracefully

## Progress
- [x] Add NATS client dependency to TUI
- [x] Create NATS connection manager in app.rs
- [x] Implement task submission (publish to task.submitted)
- [x] Implement response handling (subscribe to task.complete)
- [x] Add agent flow configuration (orchestrator vs direct)
- [x] Update chat message handling to display agent responses
- [x] Add error handling for NATS failures
- [x] Add timeout handling for agent responses
- [ ] Test NATS integration with mock responses
- [x] Update documentation

## Decisions
- **TUI as alternative frontend**: Works alongside kernel/interface, not replacing it
- **Configurable agent flow**: Add command/setting to switch between orchestrator (parallel) and direct (single-agent) modes
- **Focus on NATS first**: Implement NATS connection/publish/subscribe, reasoning adapter integration can come later
- **Timeout handling**: Check for timed-out tasks on every tick, remove from pending set, show timeout message

## Architecture
- Add `nats_client` field to AppState
- Add `agent_flow_mode` field (Orchestrator/Direct)
- Add `agent_timeout_secs` field for timeout configuration
- Add `pending_tasks` map to track submission timestamps
- Create async task for NATS message handling
- Use correlation IDs to match responses to requests
- Display agent responses as Assistant messages in chat
- Check for timeouts in `on_tick()` and remove expired tasks

## Implementation Details
- Added `agentic-sdk` and `futures` dependencies to Cargo.toml
- Added `AgentFlowMode` enum to model.rs (Orchestrator/Direct)
- Added `AgentResponse` struct for channel communication
- Added `PendingTask` struct for timeout tracking (submitted_at, user_input)
- Added NATS client, agent flow mode, timeout, and response channels to AppState
- Implemented `connect_nats()` to connect and announce online
- Implemented `submit_task()` to publish to task.submitted and track pending task
- Implemented `start_response_listener()` to subscribe to task.complete
- Implemented `check_agent_responses()` to process incoming responses and check timeouts
- Added `/agent-flow` command to configure flow mode
- Added `/timeout` command to configure timeout (1-3600 seconds)
- Added CLI args: `--nats-url`, `--agent-flow`, `--timeout`
- Modified message handling to submit to agent system when NATS is connected
- Falls back to placeholder message when NATS not connected
- Timeout checking runs on every tick (default 100ms)
- Timeout messages show the original user request

## Handoff Notes
- Reasoning adapter not yet implemented - focus on NATS infrastructure
- Can test with mock responses or echo adapter
- Future work: integrate with actual reasoning adapter when available
- TODO: Test with actual NATS server and orchestrator
- Documentation updated in QUICKSTART.md
