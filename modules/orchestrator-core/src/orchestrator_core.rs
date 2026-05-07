//! Orchestrator core — NATS orchestration and plugin lifecycle management.

use agentic_sdk::message_types::{
    AgentJob, ExecutionConstraints, JobMetadata, ModelConfig, TaskComplete, TaskEnriched,
};
use agentic_sdk::plugins::orchestrator::{
    ExecutionStrategy, ResultSynthesizer, TaskDescription, TaskPlanner,
};
use agentic_sdk::PluginRegistry;
use std::sync::Arc;

/// Orchestrator core manages plugin lifecycle and coordinates task processing.
pub struct OrchestratorCore {
    registry: Arc<PluginRegistry>,
    planner: Option<Box<dyn TaskPlanner>>,
    execution: Option<Box<dyn ExecutionStrategy>>,
    synthesizer: Option<Box<dyn ResultSynthesizer>>,
}

impl OrchestratorCore {
    /// Create a new orchestrator core.
    pub fn new() -> Self {
        Self {
            registry: Arc::new(PluginRegistry::new()),
            planner: None,
            execution: None,
            synthesizer: None,
        }
    }

    /// Register the planner plugin.
    pub async fn register_planner(
        &mut self,
        planner: Box<dyn TaskPlanner>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.registry
            .register(planner as Box<dyn agentic_sdk::plugin::Plugin>)?;
        Ok(())
    }

    /// Register the execution plugin.
    pub async fn register_execution(
        &mut self,
        execution: Box<dyn ExecutionStrategy>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.registry
            .register(execution as Box<dyn agentic_sdk::plugin::Plugin>)?;
        Ok(())
    }

    /// Register the synthesizer plugin.
    pub async fn register_synthesizer(
        &mut self,
        synthesizer: Box<dyn ResultSynthesizer>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.registry
            .register(synthesizer as Box<dyn agentic_sdk::plugin::Plugin>)?;
        Ok(())
    }

    /// Convert TaskDescription to AgentJob.
    /// Optimized to avoid cloning the entire context for each job.
    fn convert_to_agent_job(
        &self,
        task_desc: TaskDescription,
        original_task: &TaskEnriched,
    ) -> AgentJob {
        AgentJob {
            job_id: uuid::Uuid::new_v4().to_string(),
            correlation_parent: original_task.correlation_id.clone(),
            task: agentic_sdk::message_types::TaskDescription {
                user_input: task_desc.description,
                sub_task: None,
                output_format: None,
                user_constraints: vec![],
            },
            // Avoid cloning context - use Arc if needed in future
            context: original_task.context.clone(),
            available_tool_capabilities: vec![],
            constraints: ExecutionConstraints::default(),
            model_config: ModelConfig::default(),
            metadata: JobMetadata::default(),
            adapter_hints: None,
            schema_version: 1,
        }
    }

    /// Process an enriched task through the orchestration pipeline.
    pub async fn process_task(
        &self,
        task: TaskEnriched,
    ) -> Result<TaskComplete, Box<dyn std::error::Error>> {
        // Phase 1: Planning
        let planner = self
            .planner
            .as_ref()
            .ok_or("No planner plugin configured")?;
        let task_descriptions = planner.decompose(&task).await?;

        // Convert TaskDescription to AgentJob in parallel
        let agent_jobs: Vec<AgentJob> = task_descriptions
            .into_iter()
            .map(|desc| self.convert_to_agent_job(desc, &task))
            .collect();

        // Phase 2: Execution
        let execution = self
            .execution
            .as_ref()
            .ok_or("No execution plugin configured")?;
        let job_ids = execution.dispatch_jobs(agent_jobs).await?;

        // Phase 3: Result collection
        let correlation_parent = &task.correlation_id;
        let results = execution
            .collect_results(correlation_parent, job_ids.len())
            .await?;

        // Phase 4: Synthesis
        let synthesizer = self
            .synthesizer
            .as_ref()
            .ok_or("No synthesizer plugin configured")?;
        let complete = synthesizer.synthesize(results, &task).await?;

        Ok(complete)
    }

    /// Get the plugin registry.
    pub fn registry(&self) -> Arc<PluginRegistry> {
        Arc::clone(&self.registry)
    }
}
