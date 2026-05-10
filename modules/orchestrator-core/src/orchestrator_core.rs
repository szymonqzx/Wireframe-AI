//! Orchestrator core — NATS orchestration and plugin lifecycle management.

use agentic_sdk::message_types::{
    AgentJob, ExecutionConstraints, JobMetadata, ModelConfig, TaskComplete, TaskEnriched,
};
use agentic_sdk::plugins::orchestrator::{
    ExecutionStrategy, ResultSynthesizer, TaskDescription, TaskPlanner,
};
use agentic_sdk::PluginRegistry;
use chrono::Utc;
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
        self.planner = Some(planner);
        Ok(())
    }

    /// Register the execution plugin.
    pub async fn register_execution(
        &mut self,
        execution: Box<dyn ExecutionStrategy>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.execution = Some(execution);
        Ok(())
    }

    /// Register the synthesizer plugin.
    pub async fn register_synthesizer(
        &mut self,
        synthesizer: Box<dyn ResultSynthesizer>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.synthesizer = Some(synthesizer);
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
    ///
    /// Falls back to direct execution if orchestration plugins are not configured.
    pub async fn process_task(
        &self,
        task: TaskEnriched,
    ) -> Result<TaskComplete, Box<dyn std::error::Error>> {
        // Check if all orchestration plugins are configured
        let has_orchestration = self.planner.is_some() && self.execution.is_some() && self.synthesizer.is_some();

        if has_orchestration {
            self.process_with_orchestration(task).await
        } else {
            tracing::warn!(
                "Orchestration plugins not configured, falling back to direct execution"
            );
            self.execute_directly(task).await
        }
    }

    /// Process task with full orchestration pipeline (requires all plugins).
    async fn process_with_orchestration(
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

    /// Execute task directly without orchestration (single-agent mode).
    ///
    /// When orchestration plugins are not configured, returns a TaskComplete
    /// explaining that execution plugin is required for actual model inference.
    async fn execute_directly(
        &self,
        task: TaskEnriched,
    ) -> Result<TaskComplete, Box<dyn std::error::Error>> {
        // If execution plugin is available, use it for single job
        if let Some(execution) = &self.execution {
            // Convert task to single job
            let task_desc = TaskDescription {
                description: task.user_input.clone(),
                dependencies: vec![],
                metadata: serde_json::json!({ "execution_mode": "direct" }),
            };
            let agent_job = self.convert_to_agent_job(task_desc, &task);

            let _job_ids = execution.dispatch_jobs(vec![agent_job]).await?;
            let correlation_parent = &task.correlation_id;
            let results = execution.collect_results(correlation_parent, 1).await?;

            // If synthesizer is available, use it; otherwise wrap first result
            if let Some(synthesizer) = &self.synthesizer {
                synthesizer.synthesize(results, &task).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
            } else {
                // Wrap single result directly
                self.wrap_single_result(results, &task)
            }
        } else {
            // No execution plugin available - return clear message
            // The orchestrator-core is designed to coordinate plugins, not execute directly
            Ok(TaskComplete {
                session_id: task.session_id.clone(),
                correlation_id: task.correlation_id.clone(),
                result: "Orchestrator-core requires execution plugin for model inference. The orchestrator coordinates plugins but does not have direct model access. Configure execution-parallel or execution-sequential plugin to enable task execution.".to_string(),
                side_effects: vec![],
                warnings: vec![
                    "No execution plugin configured".to_string(),
                    "Orchestrator-core cannot execute tasks directly - requires execution infrastructure".to_string(),
                ],
                completed_at: Utc::now().timestamp(),
            })
        }
    }

    /// Wrap a single agent result into TaskComplete (used when synthesizer is not configured).
    fn wrap_single_result(
        &self,
        results: Vec<agentic_sdk::message_types::AgentResult>,
        original_task: &TaskEnriched,
    ) -> Result<TaskComplete, Box<dyn std::error::Error>> {
        let result = results
            .into_iter()
            .next()
            .ok_or("No results from execution")?;

        let text = result.output.text.unwrap_or_default();
        let side_effects = result
            .output
            .files_written
            .into_iter()
            .map(|path| agentic_sdk::message_types::SideEffect {
                kind: "file_written".to_string(),
                description: path.to_string_lossy().to_string(),
                path: Some(path),
            })
            .collect();

        Ok(TaskComplete {
            session_id: original_task.session_id.clone(),
            correlation_id: original_task.correlation_id.clone(),
            result: text,
            side_effects,
            warnings: result.errors.iter().map(|e| format!("[{}] {}", e.code, e.message)).collect(),
            completed_at: Utc::now().timestamp(),
        })
    }

    /// Get the plugin registry.
    pub fn registry(&self) -> Arc<PluginRegistry> {
        Arc::clone(&self.registry)
    }
}
