//! Integration tests for interface-core with plugins.

use agentic_sdk::message_types::TaskComplete;
use wireframe_ai_interface_core::InterfaceCore;

#[tokio::test]
async fn test_interface_core_create() {
    let interface = InterfaceCore::new();
    let count = interface.registry().count();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_interface_core_plugin_lifecycle() {
    let interface = InterfaceCore::new();

    // Test default plugin loading
    interface.ensure_default_input().await;
    interface.ensure_default_output().await;

    let count = interface.registry().count();
    // Should have 0 plugins in registry since we set them directly
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_interface_core_formatting() {
    let interface = InterfaceCore::new();

    let complete = TaskComplete {
        session_id: "test-session-123".to_string(),
        correlation_id: uuid::Uuid::new_v4().to_string(),
        result: "Test successful completion".to_string(),
        side_effects: vec![],
        warnings: vec![],
        completed_at: chrono::Utc::now().timestamp(),
    };

    let formatted = interface.format_result(&complete).await.unwrap();
    assert!(formatted.contains("Test successful completion"));
}

#[tokio::test]
async fn test_interface_core_empty_result_formatting() {
    let interface = InterfaceCore::new();

    let complete = TaskComplete {
        session_id: "test-session-empty".to_string(),
        correlation_id: uuid::Uuid::new_v4().to_string(),
        result: "".to_string(),
        side_effects: vec![],
        warnings: vec![],
        completed_at: chrono::Utc::now().timestamp(),
    };

    let formatted = interface.format_result(&complete).await.unwrap();
    assert!(formatted.contains("Result:"));
}

#[tokio::test]
async fn test_interface_core_large_result_formatting() {
    let interface = InterfaceCore::new();

    let large_result = "x".repeat(10000);
    let complete = TaskComplete {
        session_id: "test-session-large".to_string(),
        correlation_id: uuid::Uuid::new_v4().to_string(),
        result: large_result.clone(),
        side_effects: vec![],
        warnings: vec![],
        completed_at: chrono::Utc::now().timestamp(),
    };

    let formatted = interface.format_result(&complete).await.unwrap();
    assert!(formatted.starts_with("Result:"));
}

#[tokio::test]
async fn test_interface_core_special_characters_formatting() {
    let interface = InterfaceCore::new();

    let special_result = "Test with special chars: \n\t\r\"'\\";
    let complete = TaskComplete {
        session_id: "test-session-special".to_string(),
        correlation_id: uuid::Uuid::new_v4().to_string(),
        result: special_result.to_string(),
        side_effects: vec![],
        warnings: vec![],
        completed_at: chrono::Utc::now().timestamp(),
    };

    let formatted = interface.format_result(&complete).await.unwrap();
    assert!(formatted.contains("special chars"));
}
