//! Template for an Orchestrator Task Planner Plugin
//!
//! This template provides a starting point for implementing a custom
//! task planner for the Wireframe-AI Orchestrator module.
//!
//! To use this template:
//! 1. Copy this file to your plugin directory
//! 2. Replace "MyPlanner" with your plugin name
//! 3. Implement the TaskPlanner trait methods
//! 4. Add your specific planning logic
//! 5. Register the plugin in your module

use agentic_sdk::plugins::orchestrator::{TaskPlanner, PlanningError, TaskDescription};
use agentic_sdk::message_types::TaskEnriched;
use agentic_sdk::plugin::{Plugin, PluginError};
use async_trait::async_trait;
use serde_json::Value;

/// Your custom task planner implementation
pub struct MyPlanner {
    // Add your planner-specific fields here
    // Example:
    // max_depth: usize,
    // branching_factor: usize,
    // use_llm: bool,
}

impl MyPlanner {
    /// Create a new instance of your task planner
    pub fn new(/* Add your constructor parameters */) -> Self {
        Self {
            // Initialize your fields
            // max_depth: 5,
            // branching_factor: 3,
            // use_llm: false,
        }
    }
}

#[async_trait]
impl Plugin for MyPlanner {
    fn plugin_id(&self) -> &'static str {
        "planner-my-custom" // Replace with your plugin ID
    }

    fn version(&self) -> &'static str {
        "1.0.0" // Update with your version
    }

    fn description(&self) -> &'static str {
        "My custom task planner for Wireframe-AI" // Update with your description
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), PluginError> {
        // Parse configuration and initialize your planner
        // Example:
        // if let Some(max_depth) = config.get("max_depth").and_then(|v| v.as_u64()) {
        //     self.max_depth = max_depth as usize;
        // }
        //
        // if let Some(branching_factor) = config.get("branching_factor").and_then(|v| v.as_u64()) {
        //     self.branching_factor = branching_factor as usize;
        // }

        println!("MyPlanner initialized with config: {}", config);
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        // Check if your planner is healthy
        // Example:
        // if self.use_llm {
        //     // Check LLM connection
        // }

        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        // Cleanup resources
        println!("MyPlanner shutdown complete");
        Ok(())
    }
}

#[async_trait]
impl TaskPlanner for MyPlanner {
    async fn decompose(&self, task: &TaskEnriched) -> Result<Vec<TaskDescription>, PlanningError> {
        // Decompose the task into subtasks
        // This is where your custom planning logic goes

        println!("Decomposing task: {}", task.user_input);

        // Example 1: Simple keyword-based decomposition
        let subtasks: Vec<TaskDescription> = if task.user_input.contains(" and ") {
            task.user_input
                .split(" and ")
                .map(|s| TaskDescription {
                    description: s.trim().to_string(),
                    dependencies: vec![],
                    metadata: Value::Null,
                })
                .collect()
        } else {
            // Single task, no decomposition needed
            vec![TaskDescription {
                description: task.user_input.clone(),
                dependencies: vec![],
                metadata: Value::Null,
            }]
        };

        // Example 2: LLM-based decomposition (if use_llm is enabled)
        // if self.use_llm {
        //     let llm_response = call_llm_for_decomposition(&task.user_input).await?;
        //     return parse_llm_response(llm_response);
        // }

        // Example 3: Hierarchical decomposition with dependencies
        // let mut subtasks = Vec::new();
        // let parts = analyze_task_structure(&task.user_input);
        // for (i, part) in parts.iter().enumerate() {
        //     let dependencies = if i > 0 {
        //         vec![format!("subtask-{}", i)]
        //     } else {
        //         vec![]
        //     };
        //     subtasks.push(TaskDescription {
        //         description: part.clone(),
        //         dependencies,
        //         metadata: json!({ "priority": i }),
        //     });
        // }

        if subtasks.is_empty() {
            return Err(PlanningError::DecompositionFailed(
                "No subtasks generated".to_string(),
            ));
        }

        println!("Generated {} subtasks", subtasks.len());
        Ok(subtasks)
    }
}

// ============================================================================
// Example Configuration
// ============================================================================

/*
Add this to your module's configuration file:

plugins:
  planner:
    plugin_id: "planner-my-custom"
    config:
      max_depth: 5
      branching_factor: 3
      use_llm: false
*/

// ============================================================================
// Example Usage in Module
// ============================================================================

/*
use agentic_sdk::PluginRegistry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let registry = PluginRegistry::new();

    // Create and register your planner plugin
    let planner = Box::new(MyPlanner::new(/* params */));
    registry.register(planner).await?;

    // Retrieve and use the plugin
    let planner_plugin: Arc<MyPlanner> = registry.get("planner-my-custom").await?;

    let task = TaskEnriched {
        session_id: "test-session".to_string(),
        correlation_id: "test-correlation".to_string(),
        user_input: "Create a Python script and test it".to_string(),
        context: Default::default(),
        inferred_constraints: vec![],
        enriched_at: chrono::Utc::now().timestamp(),
    };

    let subtasks = planner_plugin.decompose(&task).await?;
    println!("Subtasks: {:?}", subtasks);

    Ok(())
}
*/

// ============================================================================
// Advanced: LLM-Based Planning
// ============================================================================

/*
If you want to use an LLM for planning, you can add this helper function:

async fn call_llm_for_decomposition(task: &str) -> Result<String, PlanningError> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", std::env::var("OPENAI_API_KEY").unwrap()))
        .json(&serde_json::json!({
            "model": "gpt-4",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a task planner. Decompose the user's task into subtasks. Return as JSON array of strings."
                },
                {
                    "role": "user",
                    "content": task
                }
            ]
        }))
        .send()
        .await
        .map_err(|e| PlanningError::LlmPlanningFailed(e.to_string()))?;

    let response_text = response
        .text()
        .await
        .map_err(|e| PlanningError::LlmPlanningFailed(e.to_string()))?;

    Ok(response_text)
}
*/

// ============================================================================
// Testing
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use agentic_sdk::ContextPackageBuilder;

    #[tokio::test]
    async fn test_planner_initialization() {
        let mut planner = MyPlanner::new();
        let config = json!({ "max_depth": 5 });
        assert!(planner.initialize(&config).await.is_ok());
    }

    #[tokio::test]
    async fn test_simple_decomposition() {
        let planner = MyPlanner::new();

        let task = TaskEnriched {
            session_id: "test-session".to_string(),
            correlation_id: "test-correlation".to_string(),
            user_input: "Create a script and test it".to_string(),
            context: ContextPackageBuilder::default().build().unwrap(),
            inferred_constraints: vec![],
            enriched_at: chrono::Utc::now().timestamp(),
        };

        let subtasks = planner.decompose(&task).await.unwrap();
        assert_eq!(subtasks.len(), 2);
    }

    #[tokio::test]
    async fn test_single_task_no_decomposition() {
        let planner = MyPlanner::new();

        let task = TaskEnriched {
            session_id: "test-session".to_string(),
            correlation_id: "test-correlation".to_string(),
            user_input: "Create a script".to_string(),
            context: ContextPackageBuilder::default().build().unwrap(),
            inferred_constraints: vec![],
            enriched_at: chrono::Utc::now().timestamp(),
        };

        let subtasks = planner.decompose(&task).await.unwrap();
        assert_eq!(subtasks.len(), 1);
    }

    #[tokio::test]
    async fn test_decomposition_with_dependencies() {
        // Test your custom dependency logic here
        let planner = MyPlanner::new();
        // Add your test
    }
}
