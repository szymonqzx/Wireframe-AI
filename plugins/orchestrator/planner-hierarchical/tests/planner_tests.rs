//! Tests for the hierarchical planner plugin.

use agentic_sdk::message_types::TaskEnriched;
use agentic_sdk::plugins::orchestrator::TaskPlanner;
use agentic_sdk::Plugin;
use planner_hierarchical::HierarchicalPlanner;

#[tokio::test]
async fn test_planner_hierarchical_plugin_id() {
    let planner = HierarchicalPlanner::new();
    assert_eq!(planner.plugin_id(), "planner-hierarchical");
}

#[tokio::test]
async fn test_planner_hierarchical_decompose() {
    let planner = HierarchicalPlanner::new();
    let task = TaskEnriched {
        session_id: "test".to_string(),
        correlation_id: "test".to_string(),
        user_input: "test task".to_string(),
        context: Default::default(),
        inferred_constraints: vec![],
        enriched_at: 0,
    };
    let result = planner.decompose(&task).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 3);
}
