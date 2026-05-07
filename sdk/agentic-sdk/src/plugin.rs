//! Plugin trait and error types for the agentic SDK.
//!
//! This module provides the Plugin trait that allows modules to be
//! dynamically loaded and managed by the SDK.

use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;
use std::any::Any;
use thiserror::Error;

/// Error type for plugin operations.
#[derive(Error, Debug, Serialize)]
pub enum PluginError {
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Incompatible version: expected {expected}, got {got}")]
    IncompatibleVersion { expected: String, got: String },

    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("Shutdown failed: {0}")]
    ShutdownFailed(String),
}

/// Trait for plugins that can be dynamically loaded and managed.
#[async_trait]
pub trait Plugin: Send + Sync + Any {
    /// Returns the unique plugin ID.
    fn plugin_id(&self) -> &'static str;

    /// Returns the version of this plugin.
    fn version(&self) -> &'static str;

    /// Returns a description of this plugin.
    fn description(&self) -> &'static str;

    /// Initializes the plugin with the provided configuration.
    async fn initialize(&mut self, config: &Value) -> Result<(), PluginError>;

    /// Performs a health check on the plugin.
    async fn health_check(&self) -> Result<bool, PluginError>;

    /// Shuts down the plugin and cleans up resources.
    async fn shutdown(&mut self) -> Result<(), PluginError>;
}
