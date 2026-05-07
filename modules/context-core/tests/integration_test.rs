use agentic_sdk::envelope::Envelope;
use agentic_sdk::message_types::TaskSubmitted;
use wireframe_ai_context_core::ContextCore;

#[tokio::test]
async fn test_context_core_initialization() {
    let _context_core = ContextCore::new(10, 20, 8000);

    // Test that context core can be created
    // Private fields are not accessible in tests, but we can test the public API
}

#[tokio::test]
async fn test_context_core_handle_task_without_plugins() {
    let context_core = ContextCore::new(10, 20, 8000);

    let task = TaskSubmitted {
        session_id: "test_session".to_string(),
        user_input: "test input".to_string(),
        submitted_at: chrono::Utc::now().timestamp(),
    };
    let envelope = Envelope::new(
        "task.submitted",
        task.clone(),
        Some("test_session".to_string()),
    );

    // Should handle task even without plugins (graceful degradation)
    let result = context_core.handle_task(task, envelope).await;
    assert!(result.is_ok());
}
