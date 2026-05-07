//! Hierarchical planner — breaks down complex tasks into subtasks.

use agentic_sdk::message_types::TaskEnriched;
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::orchestrator::TaskPlanner;
use async_trait::async_trait;
use serde_json::Value;

/// Hierarchical planner that decomposes tasks.
pub struct HierarchicalPlanner {
    max_depth: usize,
}

impl HierarchicalPlanner {
    pub fn new() -> Self {
        Self { max_depth: 3 }
    }

    pub fn with_max_depth(depth: usize) -> Self {
        Self { max_depth: depth }
    }
}

impl Default for HierarchicalPlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for HierarchicalPlanner {
    fn plugin_id(&self) -> &'static str {
        "planner-hierarchical"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Hierarchical planner for orchestrator"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        if let Some(depth) = config.get("max_depth").and_then(|v| v.as_u64()) {
            self.max_depth = depth as usize;
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
impl TaskPlanner for HierarchicalPlanner {
    async fn decompose(
        &self,
        task: &TaskEnriched,
    ) -> Result<
        Vec<agentic_sdk::plugins::orchestrator::TaskDescription>,
        agentic_sdk::plugins::orchestrator::PlanningError,
    > {
        // Simple hierarchical decomposition - split task into subtasks
        let subtasks = vec![
            agentic_sdk::plugins::orchestrator::TaskDescription {
                description: format!("Analyze: {}", task.user_input),
                dependencies: vec![],
                metadata: serde_json::json!({"phase": "analysis"}),
            },
            agentic_sdk::plugins::orchestrator::TaskDescription {
                description: format!("Research: {}", task.user_input),
                dependencies: vec!["analysis".to_string()],
                metadata: serde_json::json!({"phase": "research"}),
            },
            agentic_sdk::plugins::orchestrator::TaskDescription {
                description: format!("Implement: {}", task.user_input),
                dependencies: vec!["research".to_string()],
                metadata: serde_json::json!({"phase": "implementation"}),
            },
        ];
        Ok(subtasks)
    }
}
