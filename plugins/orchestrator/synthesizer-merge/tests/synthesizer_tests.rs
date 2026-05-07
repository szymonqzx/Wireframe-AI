use agentic_sdk::message_types::{AgentOutput, AgentResult, ContextPackage, TaskEnriched};
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::orchestrator::ResultSynthesizer;
use synthesizer_merge::MergeSynthesizer;

#[tokio::test]
async fn test_synthesizer_lifecycle() {
    let mut synthesizer = MergeSynthesizer::new();

    let config = serde_json::json!({});
    synthesizer.initialize(&config).await.unwrap();

    let healthy = synthesizer.health_check().await.unwrap();
    assert!(healthy);

    synthesizer.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_synthesize_results() {
    let synthesizer = MergeSynthesizer::new();

    let task = TaskEnriched {
        session_id: "test-session".to_string(),
        correlation_id: "test-correlation".to_string(),
        user_input: "test task".to_string(),
        context: ContextPackage::default(),
        inferred_constraints: vec![],
        enriched_at: chrono::Utc::now().timestamp(),
    };

    let result1 = AgentResult {
        job_id: "job-1".to_string(),
        correlation_parent: "test-correlation".to_string(),
        output: AgentOutput {
            text: Some("Result from agent 1".to_string()),
            structured: None,
            files_written: vec![],
            commands_run: vec![],
        },
        errors: vec![],
        tool_invocations: vec![],
        usage: None,
        completed_at: chrono::Utc::now().timestamp(),
    };

    let result2 = AgentResult {
        job_id: "job-2".to_string(),
        correlation_parent: "test-correlation".to_string(),
        output: AgentOutput {
            text: Some("Result from agent 2".to_string()),
            structured: None,
            files_written: vec![],
            commands_run: vec![],
        },
        errors: vec![],
        tool_invocations: vec![],
        usage: None,
        completed_at: chrono::Utc::now().timestamp(),
    };

    let complete = synthesizer
        .synthesize(vec![result1, result2], &task)
        .await
        .unwrap();

    assert_eq!(complete.session_id, "test-session");
    assert_eq!(complete.correlation_id, "test-correlation");
    assert!(complete.result.contains("Agent 1 Output"));
    assert!(complete.result.contains("Agent 2 Output"));
}

#[tokio::test]
async fn test_synthesize_empty_results() {
    let synthesizer = MergeSynthesizer::new();

    let task = TaskEnriched {
        session_id: "test-session".to_string(),
        correlation_id: "test-correlation".to_string(),
        user_input: "test task".to_string(),
        context: ContextPackage::default(),
        inferred_constraints: vec![],
        enriched_at: chrono::Utc::now().timestamp(),
    };

    let result = synthesizer.synthesize(vec![], &task).await;
    assert!(result.is_err());
}
