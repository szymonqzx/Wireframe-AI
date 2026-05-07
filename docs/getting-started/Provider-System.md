# Wireframe AI - Provider System

## Overview

The Provider System is a unified interface for LLM providers in Wireframe-AI. It enables the reasoning adapter to communicate with different LLM providers (OpenAI, Anthropic, Google, Cohere, Ollama, and custom providers) through a common trait, supporting capability negotiation, session management, streaming responses, provider discovery, and intelligent routing with fallback logic.

## Architecture

```
┌──────────────────┐
│ Reasoning Adapter │
│   (Rust)          │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Provider Core    │
│ - Provider trait │
│ - SessionManager │
│ - Discovery      │
│ - Router         │
│ - Config         │
└────────┬─────────┘
         │
    ┌────┴────┬──────────┬──────────┬──────────┐
    ▼         ▼          ▼          ▼          ▼
┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
│ OpenAI  │ │Anthropic│ │ Google │ │ Cohere │ │ Ollama │
│Provider│ │Provider │ │Provider│ │Provider│ │Provider│
└────────┘ └────────┘ └────────┘ └────────┘ └────────┘
```

## Core Components

### 1. Provider Trait

The `Provider` trait defines the interface that all LLM providers must implement:

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
    ) -> Result<BoxStream<'static, StreamEvent>>;

    /// Get provider metadata
    fn describe(&self) -> ProviderMetadata;

    /// Get current status
    fn status(&self) -> ProviderStatus;

    /// Provider name
    fn name(&self) -> &str;

    /// Current model
    fn model(&self) -> String;

    /// Set model
    fn set_model(&self, model: &str) -> Result<()>;

    /// List available models
    fn available_models(&self) -> Vec<String>;

    /// Whether provider supports streaming
    fn supports_streaming(&self) -> bool;

    /// Estimated cost per 1K tokens (prompt, completion) in USD cents
    fn cost_per_1k_tokens(&self) -> Option<(u64, u64)>;

    /// Create a new provider instance with independent mutable state
    fn fork(&self) -> Arc<dyn Provider>;
}
```

### 2. Provider Discovery Registry

The `ProviderDiscoveryRegistry` manages provider registration and discovery:

```rust
pub struct ProviderDiscoveryRegistry {
    providers: HashMap<String, Arc<dyn Provider>>,
    metadata: HashMap<String, ProviderMetadata>,
    available_from_env: HashMap<String, ProviderMetadata>,
}

impl ProviderDiscoveryRegistry {
    /// Register a provider
    pub fn register(&mut self, name: impl Into<String>, provider: Arc<dyn Provider>);

    /// Get a provider by name
    pub fn get(&self, name: &str) -> Option<Arc<dyn Provider>>;

    /// Get metadata for a provider
    pub fn metadata(&self, name: &str) -> Option<&ProviderMetadata>;

    /// List all registered provider names
    pub fn list(&self) -> Vec<String>;

    /// Discover providers from environment variables
    pub fn discover_from_env(&mut self) -> Vec<(String, ProviderMetadata)>;

    /// Find providers that support a specific capability
    pub fn by_capability(&self, capability: &str) -> Vec<(String, ProviderMetadata)>;
}
```

### 3. Provider Router

The `ProviderRouter` implements intelligent provider selection with fallback logic:

```rust
pub struct ProviderRouter {
    registry: ProviderDiscoveryRegistry,
    fallback_chain: Vec<String>,
    strategy: RoutingStrategy,
}

impl ProviderRouter {
    /// Select a provider based on routing strategy and requirements
    pub fn select_provider(&self, required_features: &[String]) -> Result<String>;

    /// Set routing strategy
    pub fn with_strategy(mut self, strategy: RoutingStrategy) -> Self;
}

pub enum RoutingStrategy {
    /// Always use the default provider, fallback on error
    DefaultWithFallback,
    /// Round-robin across available providers
    RoundRobin,
    /// Select provider with lowest cost per token
    LowestCost,
    /// Select provider with highest availability score
    HighestAvailability,
}
```

### 4. Configuration Schema

Unified configuration for all providers supporting TOML and JSON formats:

```rust
pub struct ProviderConfig {
    pub name: String,
    pub provider_type: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub enabled: Option<bool>,
    pub priority: Option<u32>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub stream: Option<bool>,
}

pub struct ProviderRegistryConfig {
    pub default_provider: String,
    pub fallback_chain: Vec<String>,
    pub providers: Vec<ProviderConfig>,
    pub routing_strategy: Option<RoutingStrategy>,
}
```

### 5. Session Manager

Manages conversation sessions for multi-turn LLM interactions:

```rust
pub struct SessionManager {
    sessions: HashMap<String, Session>,
}

impl SessionManager {
    /// Ensure a session exists (create or reuse)
    pub fn ensure_session(&mut self, session_id: Option<&str>, provider: &str, model: &str) -> String;

    /// Get a session by ID
    pub fn get_session(&self, session_id: &str) -> Option<&Session>;

    /// Get mutable session
    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut Session>;

    /// Close a session
    pub fn close_session(&mut self, session_id: &str) -> bool;

    /// List all sessions
    pub fn list_sessions(&self) -> Vec<SessionInfo>;
}
```

### 6. Message Types

#### Message
```rust
pub struct Message {
    pub role: String,
    pub content: String,
    pub tool_call_id: Option<String>,
}
```

#### StreamEvent
```rust
pub enum StreamEvent {
    TextDelta { text: String },
    ToolCall { id: String, name: String, arguments: String },
    Done,
}
```

#### ProviderMetadata
```rust
pub struct ProviderMetadata {
    pub provider_id: String,
    pub provider_label: String,
    pub provider_version: String,
    pub protocol_version: String,
    pub transport: String,
    pub capabilities: ProviderCapabilities,
}
```

#### ProviderStatus
```rust
pub struct ProviderStatus {
    pub availability: Availability,
    pub setup_state: SetupState,
    pub requires_manual_setup: bool,
    pub diagnostics: Vec<Diagnostic>,
}
```

## Provider Implementations

### OpenAI Provider

Located in `providers/openai/`, implements the Provider trait for OpenAI-compatible APIs (including Azure OpenAI, DeepSeek, OpenCode).

**Features:**
- HTTP transport
- Streaming responses (SSE)
- Tool/function calling
- Multiple model support
- API key authentication

**Configuration:**
```rust
let provider = OpenAIProvider::new(OpenAIConfig {
    api_key: Some("sk-...".to_string()),
    base_url: Some("https://api.openai.com/v1".to_string()),
    model: "gpt-4o".to_string(),
    stream: Some(true),
});
```

### Anthropic Provider

Located in `providers/anthropic/`, implements the Provider trait for Anthropic Claude API.

**Features:**
- HTTP transport
- Streaming responses (SSE)
- Tool/function calling
- System prompt support
- Multiple model support
- API key authentication

**Configuration:**
```rust
let provider = AnthropicProvider::new(AnthropicConfig {
    api_key: Some("sk-ant-...".to_string()),
    base_url: Some("https://api.anthropic.com".to_string()),
    model: "claude-3-5-sonnet-20241022".to_string(),
    stream: Some(true),
});
```

### Google Gemini Provider

Located in `providers/google/`, implements the Provider trait for Google Gemini API.

**Features:**
- HTTP transport
- Streaming responses (SSE)
- Tool/function calling
- System instruction support
- Multiple model support
- API key authentication

**Configuration:**
```rust
let provider = GeminiProvider::new(GeminiConfig {
    api_key: Some("...".to_string()),
    base_url: Some("https://generativelanguage.googleapis.com/v1beta".to_string()),
    model: "gemini-1.5-pro".to_string(),
    stream: Some(true),
});
```

### Cohere Provider

Located in `providers/cohere/`, implements the Provider trait for Cohere Command API.

**Features:**
- HTTP transport
- Streaming responses (SSE)
- Tool/function calling
- Preamble (system prompt) support
- Multiple model support
- API key authentication

**Configuration:**
```rust
let provider = CohereProvider::new(CohereConfig {
    api_key: Some("...".to_string()),
    base_url: Some("https://api.cohere.com".to_string()),
    model: "command-r".to_string(),
    stream: Some(true),
});
```

### Ollama Provider

Located in `providers/ollama/`, implements the Provider trait for local Ollama models using OpenAI-compatible API.

**Features:**
- HTTP transport (local)
- Streaming responses (SSE, OpenAI-compatible)
- Tool/function calling
- No API key required
- Multiple model support
- Health check endpoint

**Configuration:**
```rust
let provider = OllamaProvider::new(OllamaConfig {
    model: "llama3.2".to_string(),
    base_url: Some("http://localhost:11434/v1".to_string()),
    stream: Some(true),
});
```

## Configuration

### Configuration Files

Provider configuration can be specified via TOML or JSON files:

**TOML Example** (`provider-config.toml`):
```toml
default_provider = "openai"
fallback_chain = ["openai", "anthropic", "google", "cohere", "ollama"]
routing_strategy = "default_with_fallback"

[[providers]]
name = "openai"
provider_type = "openai"
model = "gpt-4o"
api_key = "${OPENAI_API_KEY}"
enabled = true
priority = 100
stream = true

[[providers]]
name = "anthropic"
provider_type = "anthropic"
model = "claude-3-5-sonnet-20241022"
api_key = "${ANTHROPIC_API_KEY}"
enabled = true
priority = 90
stream = true
```

**JSON Example** (`provider-config.json`):
```json
{
  "default_provider": "openai",
  "fallback_chain": ["openai", "anthropic", "google", "cohere", "ollama"],
  "routing_strategy": "default_with_fallback",
  "providers": [
    {
      "name": "openai",
      "provider_type": "openai",
      "model": "gpt-4o",
      "api_key": "${OPENAI_API_KEY}",
      "enabled": true,
      "priority": 100,
      "stream": true
    }
  ]
}
```

### Configuration Locations

Configuration is loaded in the following order (first found wins):
1. `WIREFRAME_AI_PROVIDER_CONFIG` environment variable (path to config file)
2. `./provider-config.toml` or `./provider-config.json` (current directory)
3. `~/.wireframe-ai/provider-config.toml` or `~/.wireframe-ai/provider-config.json` (user home)
4. Environment variables (fallback)

### Environment Variables

If no configuration file is found, providers are configured from environment variables:

- `OPENAI_API_KEY`, `OPENAI_MODEL`, `OPENAI_BASE_URL`
- `ANTHROPIC_API_KEY`, `ANTHROPIC_MODEL`, `ANTHROPIC_BASE_URL`
- `GOOGLE_API_KEY` or `GEMINI_API_KEY`, `GOOGLE_MODEL`, `GOOGLE_BASE_URL`
- `COHERE_API_KEY`, `COHERE_MODEL`, `COHERE_BASE_URL`
- `OLLAMA_MODEL`, `OLLAMA_BASE_URL`
- `DEEPSEEK_API_KEY` (OpenAI-compatible)
- `OPENCODE_GO_API_KEY` (OpenAI-compatible)

### Routing Strategies

The provider router supports multiple strategies for provider selection:

- **DefaultWithFallback**: Use the default provider, fall back to the next in the chain on error
- **RoundRobin**: Distribute requests across available providers
- **LowestCost**: Select the provider with the lowest cost per token
- **HighestAvailability**: Select the provider with the highest availability score

## NATS Integration

The provider system integrates with NATS through new topics in the `provider` namespace:

### Provider Discovery Topics

- `provider.describe` - Request provider metadata and capabilities
- `provider.describe.response` - Provider metadata response
- `provider.status` - Request provider availability status
- `provider.status.response` - Provider status response
- `provider.list` - List all available providers
- `provider.list.response` - List of available providers

See `schemas/v1/` for JSON schemas of these messages.

## Usage in Reasoning Adapter

The reasoning adapter uses the provider system as follows:

```rust
// Load provider configuration
let config = provider_config::load_provider_config(Some(&config_path))?;

// Build provider registry from configuration
let registry = provider_config::build_registry_from_config(&config)?;

// Create router with fallback chain
let router = ProviderRouter::new(registry.clone(), config.fallback_chain);

// Select provider based on required features
let provider_name = router.select_provider(&["streaming".to_string(), "tools".to_string()])?;
let provider = registry.get(&provider_name).unwrap();

// Complete request with streaming
let mut stream = provider.complete(&messages, &tools, &system_prompt, Some(&session_id)).await?;

// Process stream events
while let Some(event) = stream.next().await {
    match event {
        StreamEvent::TextDelta { text } => { /* handle text */ }
        StreamEvent::ToolCall { id, name, arguments } => { /* execute tool */ }
        StreamEvent::Done => break,
    }
}
```

## Adding a New Provider

To add a new LLM provider:

1. Create a new crate in `providers/<provider-name>/`
2. Implement the `Provider` trait with streaming support
3. Add SSE parsing method for the provider's format
4. Add the provider to the workspace in `Cargo.toml`
5. Add provider configuration support in `adapter/rust/src/provider_config.rs`
6. Register the provider in the configuration schema

Example:

```rust
use provider_core::{Provider, ProviderMetadata, ProviderStatus, Message, StreamEvent};
use async_trait::async_trait;
use futures::stream::{BoxStream, StreamExt};

pub struct MyProvider {
    config: MyProviderConfig,
    client: Client,
}

#[derive(Debug, Clone)]
pub struct MyProviderConfig {
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
    pub stream: Option<bool>,
}

#[async_trait]
impl Provider for MyProvider {
    async fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        system: &str,
        session_id: Option<&str>,
    ) -> Result<BoxStream<'static, StreamEvent>> {
        let use_streaming = self.config.stream.unwrap_or(true);
        // Implement provider-specific completion logic with streaming support
    }

    fn describe(&self) -> ProviderMetadata {
        // Return provider metadata
    }

    fn status(&self) -> ProviderStatus {
        // Return provider status
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    // Implement other required methods...
}

impl MyProvider {
    /// Parse SSE lines from provider's streaming response
    pub fn parse_sse_line(&self, line: &str) -> Option<StreamEvent> {
        // Parse provider-specific SSE format
    }
}
```

## Session Management

Sessions are managed by the `SessionManager` in the reasoning adapter:

- Each session tracks conversation history with a specific provider and model
- Sessions are identified by a session ID (typically the correlation_parent from AgentJob)
- Sessions persist across multiple agent jobs within the same conversation
- Sessions can be closed explicitly or automatically after a timeout

## Transport Layer

Providers can use different transport mechanisms:

- **HTTP** - For cloud-based APIs (OpenAI, Anthropic, Google, Cohere)
- **HTTP (local)** - For local models (Ollama)
- **stdio** - For local models (future)
- **Socket** - For custom transport mechanisms (future)

The transport type is specified in the provider metadata and can be used by the reasoning adapter to determine how to communicate with the provider.

## Capability Negotiation

Providers advertise their capabilities through the `describe()` method:

- **Core methods** - Required Provider trait methods
- **Optional methods** - Optional provider-specific methods
- **Features** - Supported features (streaming, tools, images, etc.)
- **Custom methods** - Provider-specific custom methods

This allows the reasoning adapter to:
- Check if a provider supports required features before using it
- Select appropriate providers based on task requirements
- Use the router to find the best provider for a given task
- Provide meaningful error messages when capabilities are missing

## Streaming Support

All providers support streaming responses via Server-Sent Events (SSE):

- **OpenAI**: Standard SSE format with `data:` prefix
- **Anthropic**: SSE format with event types (`content_block_delta`, `message_stop`)
- **Google**: SSE format with `candidates` structure
- **Cohere**: SSE format with event types (`text-generation`, `tool-calls-generation`)
- **Ollama**: OpenAI-compatible SSE format

Streaming can be enabled/disabled per provider via the `stream` configuration option.

## Error Handling

Provider operations return `Result<T>` for proper error handling:

```rust
use anyhow::{Result, anyhow};

let result = provider.complete(&messages, &tools, &system, Some(&session_id)).await;
match result {
    Ok(stream) => { /* process stream */ }
    Err(e) => {
        // Handle error (e.g., provider unavailable, authentication failed)
        error!("Provider error: {}", e);
    }
}
```

The router will automatically fall back to the next provider in the chain on errors.

## Testing

Provider implementations should include:

1. **Unit tests** - Test individual methods (describe, status, available_models, parse_sse_line)
2. **Integration tests** - Test actual API calls with mock servers
3. **Streaming tests** - Test streaming response handling
4. **Error handling tests** - Test error scenarios (network failures, auth errors)

## Migration from Python Adapter

The Python adapter is being replaced by the Rust adapter with the provider system:

- **Old**: Python adapter with JSON config for providers
- **New**: Rust adapter with Provider trait, typed provider implementations, discovery, and routing

Benefits of the new system:
- Type safety through Rust's type system
- Better performance (no Python interpreter overhead)
- Unified interface for all providers
- Streaming support across all providers
- Intelligent provider selection with fallback
- Easier to add new providers
- Better error handling and debugging
- Configuration files (TOML/JSON) for easy provider management

The Python adapter remains available as a legacy implementation during the migration period.

## References

- Provider trait: `provider-core/src/lib.rs`
- Provider discovery: `provider-core/src/discovery.rs`
- Provider router: `provider-core/src/router.rs`
- Configuration: `provider-core/src/config.rs`
- OpenAI provider: `providers/openai/src/lib.rs`
- Anthropic provider: `providers/anthropic/src/lib.rs`
- Google provider: `providers/google/src/lib.rs`
- Cohere provider: `providers/cohere/src/lib.rs`
- Ollama provider: `providers/ollama/src/lib.rs`
- Adapter integration: `adapter/rust/src/provider_config.rs`
- Example configurations: `provider-config.example.toml`, `provider-config.example.json`
- NATS schemas: `schemas/v1/provider_*.json`
- Topic documentation: `schemas/v1/TOPICS.md`
