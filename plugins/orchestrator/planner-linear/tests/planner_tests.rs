use agentic_sdk::message_types::{ContextPackage, TaskEnriched};
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::orchestrator::TaskPlanner;
use planner_linear::LinearPlanner;

#[tokio::test]
async fn test_planner_lifecycle() {
    let mut planner = LinearPlanner::new();

    // Test plugin lifecycle
    let config = serde_json::json!({"concurrency": 5});
    planner.initialize(&config).await.unwrap();

    let healthy = planner.health_check().await.unwrap();
    assert!(healthy);

    planner.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_linear_decompose() {
    let planner = LinearPlanner::with_concurrency(3);

    let task = TaskEnriched {
        session_id: "test-session".to_string(),
        correlation_id: "test-correlation".to_string(),
        user_input: "test task".to_string(),
        context: ContextPackage::default(),
        inferred_constraints: vec![],
        enriched_at: chrono::Utc::now().timestamp(),
    };

    let descriptions = planner.decompose(&task).await.unwrap();
    assert_eq!(descriptions.len(), 3);

    // All descriptions should have the same user_input as description
    for desc in &descriptions {
        assert_eq!(desc.description, "test task");
    }
}
