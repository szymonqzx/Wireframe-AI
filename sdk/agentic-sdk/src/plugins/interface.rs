//! Plugin traits for the Interface module.

use crate::message_types::TaskComplete;
use crate::message_types::TaskSubmitted;
use crate::plugin::Plugin;
use async_trait::async_trait;
use thiserror::Error;

/// Input method (CLI, web, API, etc.).
///
/// Implementations provide different ways to receive user input.
#[async_trait]
pub trait InputMethod: Plugin {
    /// Read input from the user.
    async fn read_input(&self) -> Result<TaskSubmitted, InputError>;
}

/// Output formatter.
///
/// Implementations format results for display (markdown, JSON, HTML, etc.).
#[async_trait]
pub trait OutputFormatter: Plugin {
    /// Format a task completion result.
    async fn format_result(&self, result: &TaskComplete) -> Result<String, FormatError>;
}

/// UI component (progress bars, rich output, etc.).
///
/// Implementations provide interactive UI elements.
#[async_trait]
pub trait UIComponent: Plugin {
    /// Render the UI component with current state.
    async fn render(&self, state: &UIState) -> Result<(), UIError>;
}

// ============================================================================
// Data Types
// ============================================================================

/// UI state for rendering.
#[derive(Debug, Clone)]
pub struct UIState {
    pub progress: f64,
    pub status: String,
    pub messages: Vec<String>,
}

// ============================================================================
// Error Types
// ============================================================================

/// Input method error.
#[derive(Error, Debug)]
pub enum InputError {
    #[error("Read failed: {0}")]
    ReadFailed(String),

    #[error("Parse failed: {0}")]
    ParseFailed(String),

    #[error("Interrupted")]
    Interrupted,
}

/// Output formatter error.
#[derive(Error, Debug)]
pub enum FormatError {
    #[error("Formatting failed: {0}")]
    FormattingFailed(String),

    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
}

/// UI component error.
#[derive(Error, Debug)]
pub enum UIError {
    #[error("Render failed: {0}")]
    RenderFailed(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),
}
