//! Module-specific plugin traits for each Wireframe-AI module.

pub mod adapter;
pub mod context;
pub mod interface;
pub mod orchestrator;
pub mod sandbox;

// Re-exports for convenience
pub use adapter::{AIModel, ReasoningStrategy, ToolSelector};
pub use context::{EnrichmentStrategy, MemoryBackend, StorageBackend};
pub use interface::{InputMethod, OutputFormatter, UIComponent};
pub use orchestrator::{ExecutionStrategy, ResultSynthesizer, TaskPlanner};
pub use sandbox::{ResourceLimiter, SecurityPolicy, Tool};
