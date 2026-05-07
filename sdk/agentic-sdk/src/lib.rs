//! # agentic-sdk — Wireframe AI Rust SDK
//!
//! The SDK is what makes module authoring simple. It provides:
//!
//! - **Envelope**: the universal message wrapper every module uses
//! - **Message types**: all payload structs for the topic namespace
//! - **Module trait**: the three-method interface contract
//! - **NATS helpers**: connection, heartbeat, sys.module.online/offline
//!
//! ## Example
//!
//! ```ignore
//! use agentic_sdk::{Envelope, Module, message_types::TaskSubmitted};
//!
//! struct MyModule;
//!
//! impl Module for MyModule {
//!     fn subscribes() -> &'static [&'static str] { &["task.submitted"] }
//!     fn publishes() -> &'static [&'static str]  { &["task.enriched"] }
//!     fn handle(&mut self, env: Envelope<serde_json::Value>) -> Vec<Envelope<serde_json::Value>> {
//!         vec![]
//!     }
//! }
//! ```

pub mod builders;
pub mod compatibility;
pub mod config;
pub mod envelope;
pub mod error;
pub mod message_types;
pub mod module;
pub mod orchestrator_patterns;
pub mod pipeline;
pub mod plugin;
pub mod plugin_registry;
pub mod plugins;
pub mod reasoning;
pub mod registry;
pub mod serialization;
pub mod switch;

// Re-exports
pub use async_trait::async_trait;
pub use builders::envelope_helpers;
pub use builders::{
    AgentJobBuilder, AgentResultBuilder, BuilderError, ContextPackageBuilder,
    ExecutionConstraintsBuilder, ModelConfigBuilder, TaskCompleteBuilder, TaskSubmittedBuilder,
};
pub use compatibility::{CompatibilityChecker, CompatibilityResult, ModuleInterface};
pub use config::{ConfigError, ModuleConfig, ModulePlugins, PluginConfig, PluginSpec};
pub use envelope::{
    load_schema, validate_envelope_payload, validate_payload_schema, Envelope, EnvelopeError,
};
pub use error::{retry_with_backoff, SdkError, SdkResult};
pub use message_types::*;
pub use module::{announce_offline, announce_online, publish_error, start_heartbeat, Module};
pub use pipeline::{Pipeline, PipelineError, PipelineStep};
pub use plugin::{Plugin, PluginError};
pub use plugin_registry::PluginRegistry;
pub use plugins::*;
pub use registry::{ModuleMetadata, ModuleRegistry};
pub use serialization::{
    deserialize_from_reader, from_json_slice, from_json_str, serialize_to_writer, to_compact_json,
    to_pretty_json, JsonSerializer, JsonDeserializer,
};
pub use switch::{ModuleSwitchAck, ModuleSwitchCoordinator, ModuleSwitchRequest, SwitchStatus};

// Optional: #[module] proc-macro (enabled with "macros" feature)
#[cfg(feature = "macros")]
pub use agentic_sdk_macros::module;
