//! Sequential execution strategy — executes tasks one at a time.

use agentic_sdk::message_types::{AgentJob, AgentOutput, AgentResult};
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::orchestrator::ExecutionStrategy;
use async_trait::async_trait;
use serde_json::Value;

/// Sequential execution strategy.
pub struct SequentialExecution {
    timeout_seconds: u64,
}

impl SequentialExecution {
    pub fn new() -> Self {
        Self {
            timeout_seconds: 300,
        }
    }

    pub fn with_timeout(timeout: u64) -> Self {
        Self {
            timeout_seconds: timeout,
        }
    }
}

impl Default for SequentialExecution {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for SequentialExecution {
    fn plugin_id(&self) -> &'static str {
        "execution-sequential"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Sequential execution strategy for orchestrator"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        if let Some(timeout) = config.get("timeout_seconds").and_then(|v| v.as_u64()) {
            self.timeout_seconds = timeout;
        }
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, agentic_sdk::plugin::PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }
}

#[async_trait]
impl ExecutionStrategy for SequentialExecution {
    async fn dispatch_jobs(
        &self,
        jobs: Vec<AgentJob>,
    ) -> Result<Vec<String>, agentic_sdk::plugins::orchestrator::ExecutionError> {
        // Dispatch jobs sequentially
        let mut job_ids = Vec::new();
        for job in jobs {
            // Placeholder: simulate dispatch
            job_ids.push(job.job_id.clone());
        }
        Ok(job_ids)
    }

    async fn collect_results(
        &self,
        correlation_parent: &str,
        expected_count: usize,
    ) -> Result<Vec<AgentResult>, agentic_sdk::plugins::orchestrator::ExecutionError> {
        // Placeholder: simulate result collection
        let mut results = Vec::new();
        for i in 0..expected_count {
            results.push(AgentResult {
                job_id: format!("job_{}", i),
                correlation_parent: correlation_parent.to_string(),
                output: AgentOutput {
                    text: Some(format!("Result {}", i)),
                    structured: None,
                    files_written: vec![],
                    commands_run: vec![],
                },
                tool_invocations: vec![],
                errors: vec![],
                usage: None,
                completed_at: chrono::Utc::now().timestamp(),
            });
        }
        Ok(results)
    }
}
