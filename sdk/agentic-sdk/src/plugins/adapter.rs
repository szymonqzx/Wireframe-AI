//! Plugin traits for the Adapter module.

use crate::message_types::AgentJob;
use crate::message_types::AgentOutput;
use crate::plugin::Plugin;
use async_trait::async_trait;
use serde_json::Value;
use thiserror::Error;

/// AI model interface.
///
/// Implementations provide access to different LLM providers
/// (OpenAI, Anthropic, local models, etc.).
#[async_trait]
pub trait AIModel: Plugin {
    /// Generate text completion.
    async fn generate(&self, prompt: &str, context: &AgentJob) -> Result<String, ModelError>;

    /// Generate structured output with schema.
    async fn generate_structured(
        &self,
        prompt: &str,
        schema: &Value,
        context: &AgentJob,
    ) -> Result<Value, ModelError>;
}

/// Tool selection strategy.
///
/// Implementations select which tools to use for a given task,
/// supporting different selection approaches (semantic, rule-based, LLM-based).
#[async_trait]
pub trait ToolSelector: Plugin {
    /// Select tools for a task.
    async fn select_tools(
        &self,
        task: &str,
        available: &[ToolCapability],
    ) -> Result<Vec<String>, SelectionError>;
}

/// Reasoning strategy (chain-of-thought, tree-of-thought, etc.).
///
/// Implementations provide different reasoning approaches.
#[async_trait]
pub trait ReasoningStrategy: Plugin {
    /// Execute reasoning on an agent job.
    async fn reason(&self, context: &AgentJob) -> Result<AgentOutput, ReasoningError>;
}

// ============================================================================
// Data Types
// ============================================================================

/// Tool capability description.
#[derive(Debug, Clone)]
pub struct ToolCapability {
    pub name: String,
    pub description: String,
    pub schema: Value,
}

// ============================================================================
// Error Types
// ============================================================================

/// AI model error.
#[derive(Error, Debug)]
pub enum ModelError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Model not available")]
    ModelNotAvailable,
}

/// Tool selection error.
#[derive(Error, Debug)]
pub enum SelectionError {
    #[error("Selection failed: {0}")]
    SelectionFailed(String),

    #[error("No suitable tools found")]
    NoSuitableTools,

    #[error("Invalid tool list")]
    InvalidToolList,
}

/// Reasoning strategy error.
#[derive(Error, Debug)]
pub enum ReasoningError {
    #[error("Reasoning failed: {0}")]
    ReasoningFailed(String),

    #[error("Context too large")]
    ContextTooLarge,

    #[error("Max iterations exceeded")]
    MaxIterationsExceeded,
}
