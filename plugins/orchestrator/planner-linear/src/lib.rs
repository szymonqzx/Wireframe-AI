//! Linear N-copy task planner — creates N identical subtasks from a single enriched task.

use agentic_sdk::message_types::TaskEnriched;
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::orchestrator::{PlanningError, TaskDescription, TaskPlanner};
use async_trait::async_trait;
use serde_json::Value;

/// Linear planner that creates N identical copies of a task.
pub struct LinearPlanner {
    concurrency: u32,
}

impl LinearPlanner {
    pub fn new() -> Self {
        Self {
            concurrency: 3, // Default concurrency
        }
    }

    pub fn with_concurrency(concurrency: u32) -> Self {
        Self { concurrency }
    }
}

impl Default for LinearPlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for LinearPlanner {
    fn plugin_id(&self) -> &'static str {
        "planner-linear"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Linear N-copy task planner"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        if let Some(concurrency) = config.get("concurrency").and_then(|v| v.as_u64()) {
            self.concurrency = concurrency as u32;
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
impl TaskPlanner for LinearPlanner {
    async fn decompose(&self, task: &TaskEnriched) -> Result<Vec<TaskDescription>, PlanningError> {
        let mut descriptions = Vec::new();

        for _ in 0..self.concurrency {
            let task_desc = TaskDescription {
                description: task.user_input.clone(),
                dependencies: vec![],
                metadata: serde_json::json!({}),
            };

            descriptions.push(task_desc);
        }

        Ok(descriptions)
    }
}
