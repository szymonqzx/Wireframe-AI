# Hybrid Architecture Design: Adopting jcode Provider Pattern

## Goal
Adopt jcode's provider interface pattern while keeping Wireframe-AI's NATS/message bus architecture.

## Current Wireframe-AI Architecture
```
┌─────────────┐    NATS    ┌──────────────────┐
│   TUI/CLI   │ ◄─────────► │  Kernel/Orchestrator │
└─────────────┘            └──────────────────┘
                                   │
                                   ▼
                            ┌──────────────────┐
                            │  ReasoningAdapter │
                            │  (Python)         │
                            │  - providers.json │
                            │  - HTTP calls     │
                            └──────────────────┘
```

## Target Hybrid Architecture
```
┌─────────────┐    NATS    ┌──────────────────┐
│   TUI/CLI   │ ◄─────────► │  Kernel/Orchestrator │
└─────────────┘            └──────────────────┘
                                   │
                                   ▼
                            ┌──────────────────┐
                            │  ReasoningAdapter │
                            │  (Rust)           │
                            │  - Provider trait │
                            │  - Discovery      │
                            │  - Sessions       │
                            │  - Transports     │
                            └──────────────────┘
                                   │
                    ┌──────────────┼──────────────┐
                    ▼              ▼              ▼
            ┌───────────┐  ┌───────────┐  ┌───────────┐
            │ OpenAI    │  │ Anthropic │  │ Local     │
            │ Provider  │  │ Provider  │  │ Provider  │
            │ (HTTP)    │  │ (HTTP)    │  │ (stdio)   │
            └───────────┘  └───────────┘  └───────────┘
```

## Key Changes

### 1. New Crate: `provider-core`
- **Location**: `provider-core/`
- **Purpose**: Define Provider trait and core infrastructure
- **Components**:
  - `Provider` trait (adapted from jcode)
  - `ProviderDiscovery` for capability negotiation
  - `SessionManager` for conversation sessions
  - `Transport` trait (HTTP, stdio)

### 2. Provider Implementations
- **Location**: `providers/`
- **Components**:
  - `providers/openai/` - OpenAI-compatible HTTP provider
  - `providers/anthropic/` - Anthropic HTTP provider
  - `providers/local/` - Local model stdio provider

### 3. Adapter Rewrite
- **Location**: `adapter/` (Rust, replacing Python)
- **Purpose**: NATS subscriber that uses Provider trait
- **Components**:
  - NATS message handling
  - Provider selection and routing
  - Session management
  - Tool execution via MCP

### 4. Provider Discovery Protocol
- **provider.describe**: Returns provider metadata and capabilities
- **provider.status**: Returns availability and setup state
- **Capability negotiation**: Models, features, limits

### 5. Session Management
- **Session lifecycle**: ensure, close, isolation
- **Conversation context**: Multi-turn conversations
- **Session persistence**: Resume capability

## Migration Strategy

### Phase 1: Core Infrastructure
1. Create `provider-core` crate with Provider trait
2. Implement Transport trait (HTTP, stdio)
3. Implement SessionManager
4. Implement ProviderDiscovery

### Phase 2: Provider Implementations
1. Implement OpenAI provider (HTTP)
2. Implement Anthropic provider (HTTP)
3. Implement Local provider (stdio)

### Phase 3: Adapter Rewrite
1. Create Rust adapter crate
2. Implement NATS message handling
3. Integrate Provider trait
4. Implement tool execution via MCP

### Phase 4: Migration
1. Test new adapter with existing providers
2. Update documentation
3. Deprecate Python adapter
4. Remove old JSON config

## Provider Trait (Simplified from jcode)

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    /// Send messages and get a streaming response
    async fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        system: &str,
        session_id: Option<&str>,
    ) -> Result<EventStream>;

    /// Get provider metadata
    fn describe(&self) -> ProviderMetadata;

    /// Get current status
    fn status(&self) -> ProviderStatus;

    /// List available models
    fn available_models(&self) -> Vec<String>;

    /// Set model
    fn set_model(&self, model: &str) -> Result<()>;

    /// Get current model
    fn model(&self) -> String;

    /// Create independent instance
    fn fork(&self) -> Arc<dyn Provider>;
}
```

## Provider Metadata

```rust
pub struct ProviderMetadata {
    pub provider_id: String,
    pub provider_label: String,
    pub provider_version: String,
    pub protocol_version: String,
    pub transport: String,
    pub capabilities: ProviderCapabilities,
}

pub struct ProviderCapabilities {
    pub core_methods: Vec<String>,
    pub optional_methods: Vec<String>,
    pub features: Vec<String>,
    pub custom_methods: Vec<CustomMethod>,
}
```

## Session Management

```rust
pub struct SessionManager {
    sessions: HashMap<String, Session>,
}

pub struct Session {
    pub session_id: String,
    pub provider: Arc<dyn Provider>,
    pub messages: Vec<Message>,
    pub created_at: DateTime<Utc>,
}
```

## Transport Layer

```rust
#[async_trait]
pub trait Transport: Send + Sync {
    async fn send(&self, request: Request) -> Result<Response>;
    fn transport_type(&self) -> TransportType;
}

pub enum TransportType {
    Http,
    Stdio,
    Socket,
}
```

## NATS Integration

The adapter will subscribe to `agent.job` and publish to `agent.result`, same as before, but using the new Provider trait internally.

## Backward Compatibility

During migration, the Python adapter will remain functional. The Rust adapter can be deployed alongside it, with gradual rollout.
