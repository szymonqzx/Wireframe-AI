//! Plugin traits for the Orchestrator module.

use crate::message_types::{AgentJob, AgentResult, TaskComplete, TaskEnriched};
use crate::plugin::Plugin;
use async_trait::async_trait;
use serde_json::Value;
use thiserror::Error;

/// Task decomposition strategy.
///
/// Implementations break down tasks into subtasks, supporting
/// different planning approaches (linear, hierarchical, recursive).
#[async_trait]
pub trait TaskPlanner: Plugin {
    /// Decompose a task into subtasks.
    async fn decompose(&self, task: &TaskEnriched) -> Result<Vec<TaskDescription>, PlanningError>;
}

/// Fan-out execution strategy.
///
/// Implementations handle dispatching and collecting results,
/// supporting different execution patterns (parallel, sequential, adaptive).
#[async_trait]
pub trait ExecutionStrategy: Plugin {
    /// Dispatch jobs to workers.
    async fn dispatch_jobs(&self, jobs: Vec<AgentJob>) -> Result<Vec<String>, ExecutionError>;

    /// Collect results by correlation ID.
    async fn collect_results(
        &self,
        correlation_parent: &str,
        expected_count: usize,
    ) -> Result<Vec<AgentResult>, ExecutionError>;
}

/// Result synthesis strategy.
///
/// Implementations merge multiple agent results into a final answer,
/// supporting different synthesis approaches (merge, LLM-based, weighted).
#[async_trait]
pub trait ResultSynthesizer: Plugin {
    /// Synthesize results into a final task completion.
    async fn synthesize(
        &self,
        results: Vec<AgentResult>,
        original_task: &TaskEnriched,
    ) -> Result<TaskComplete, SynthesisError>;
}

// ============================================================================
// Data Types
// ============================================================================

/// A task description for decomposition.
#[derive(Debug, Clone)]
pub struct TaskDescription {
    pub description: String,
    pub dependencies: Vec<String>,
    pub metadata: Value,
}

// ============================================================================
// Error Types
// ============================================================================

/// Task planning error.
#[derive(Error, Debug)]
pub enum PlanningError {
    #[error("Decomposition failed: {0}")]
    DecompositionFailed(String),

    #[error("LLM planning failed: {0}")]
    LlmPlanningFailed(String),

    #[error("Invalid task structure: {0}")]
    InvalidTaskStructure(String),
}

/// Execution strategy error.
#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Job dispatch failed: {0}")]
    DispatchFailed(String),

    #[error("Result collection failed: {0}")]
    CollectionFailed(String),

    #[error("Timeout waiting for results")]
    Timeout,

    #[error("Correlation mismatch")]
    CorrelationMismatch,
}

/// Result synthesis error.
#[derive(Error, Debug)]
pub enum SynthesisError {
    #[error("Synthesis failed: {0}")]
    SynthesisFailed(String),

    #[error("LLM synthesis failed: {0}")]
    LlmSynthesisFailed(String),

    #[error("No results to synthesize")]
    NoResults,
}
