---
status: active
created: 2026-05-06
---

# TEAM_018 - Hybrid: Adopt jcode Provider Pattern

## Task
Adopt jcode's provider interface pattern while keeping Wireframe-AI's NATS/message bus architecture.

## Scope (Hybrid Approach)
- **Keep Wireframe-AI core**: Rust + NATS + SQLite architecture
- **Keep modular structure**: Existing crate layout
- **Adopt jcode Provider trait**: Use jcode's provider interface pattern for LLM providers
- **Provider discovery**: Implement capability negotiation and provider.describe
- **Session management**: Add session lifecycle for LLM conversations
- **Transport flexibility**: Support HTTP + stdio (for local models)
- **Gradual migration**: Support both old JSON config and new protocol during transition

## Progress
- [x] Clone jcode repository locally
- [x] Analyze jcode codebase structure
- [x] Analyze jcode provider trait and protocol
- [x] Design hybrid architecture for Wireframe-AI
- [x] Create provider-core crate with Provider trait
- [x] Implement OpenAI provider (HTTP transport)
- [x] Adapt Rust adapter to use new provider system
- [x] Fix compilation errors in adapter/rust
- [x] Update NATS message schemas for provider protocol
- [x] Update project documentation
- [ ] Implement Anthropic provider (HTTP transport)
- [ ] Implement Local provider (stdio transport)
- [ ] Implement provider registry for NATS-based discovery
- [ ] Migrate existing Python adapter to new provider trait (or deprecate)
- [ ] Test all providers with new system

## Analysis Results

### jcode Architecture Scale
- **40+ crates** in jcode workspace
- **~1000+ source files** across all crates
- **Provider trait** with 30+ methods (complete, model switching, transport, compaction, etc.)
- **Protocol** with 50+ request types and 100+ event types
- **Full agent-to-agent communication** system
- **Swarm orchestration** built-in
- **Memory system** with embedding and graph
- **TUI system** with markdown, mermaid, workspace rendering

### Wireframe-AI Current Architecture
- **Rust core** with NATS message bus
- **Python adapter** for LLM providers (simple JSON config)
- **SQLite** for state
- **~10-15 modules** total
- **No agent-to-agent** communication
- **No swarm** orchestration
- **No memory** system

### Scope Assessment
This is not a "rewrite" - this is a **complete replacement** of Wireframe-AI with jcode's architecture. The scope is equivalent to:
- Rewriting 100% of Wireframe-AI code
- Adding 40+ new crates
- Implementing agent-to-agent communication
- Implementing swarm orchestration
- Implementing memory system
- Implementing TUI system
- Implementing protocol layer
- Implementing provider discovery
- Implementing session management
- Implementing transport layer

**Estimated effort: 4-8 weeks of full-time work**

## Decisions
- Using MIT-licensed jcode code as reference/implementation
- Complete architectural rewrite, no backward compatibility
- Full jcode protocol adoption including all transports

## Handoff Notes
- This is a complete rewrite, expect significant changes
- All existing provider configs will need migration
- Testing will be critical given the scope
- **Current state**: Provider core and OpenAI provider implemented, Rust adapter adapted and compiles successfully, NATS schemas updated, documentation updated
- **Next steps**: Implement additional providers (Anthropic, Local), implement provider registry for NATS-based discovery, test integration
- **Key changes made in this session**:
  - Created `provider-core` crate with Provider trait, SessionManager, Message, StreamEvent types
  - Created `providers/openai` crate implementing Provider trait for OpenAI-compatible APIs
  - Adapted `adapter/rust/src/main.rs` to use new Provider trait with RwLock for concurrent session management
  - Fixed compilation errors related to Arc/RwLock patterns and type mismatches
  - Added new NATS schemas for provider protocol (provider_describe, provider_metadata, provider_status, provider_list)
  - Updated `schemas/v1/TOPICS.md` with new `provider` namespace topics
  - Enhanced `agent_job.json` schema with additional model config fields (max_tokens, descriptions)
  - Created comprehensive `docs/Provider-System.md` documentation
  - Updated `docs/Project-Architecture.md` with provider system information
  - Updated `AGENTS.md` with provider system context
  - Updated `README.md` with provider system architecture and migration notes
