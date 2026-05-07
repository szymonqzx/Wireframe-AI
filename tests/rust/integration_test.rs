//! Integration tests for Wireframe-AI message flow
//!
//! Tests the complete message flow from task submission to completion.

use agentic_sdk::envelope::Envelope;
use agentic_sdk::message_types::{TaskComplete, TaskSubmitted};
use async_nats::Client;
use futures::StreamExt;
use serde_json::Value;
use std::time::Duration;
use tokio::time::timeout;

/// Helper function to create a test NATS client
async fn create_test_client() -> Client {
    let nats_url = std::env::var("TEST_NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    async_nats::connect(&nats_url)
        .await
        .expect("Failed to connect to NATS for integration test")
}

/// Helper function to publish a task and wait for completion
async fn submit_task_and_wait(client: &Client, user_input: &str) -> Result<TaskComplete, String> {
    let session_id = format!("test_session_{}", uuid::Uuid::new_v4());
    let submitted = TaskSubmitted {
        session_id: session_id.clone(),
        user_input: user_input.to_string(),
        submitted_at: chrono::Utc::now().timestamp(),
    };

    let envelope = Envelope::new("task.submitted", submitted, Some(session_id.clone()));
    let correlation_id = envelope.correlation_id.clone();
    let payload = serde_json::to_vec(&envelope)
        .map_err(|e| format!("Failed to serialize envelope: {}", e))?;

    client
        .publish("task.submitted", payload.into())
        .await
        .map_err(|e| format!("Failed to publish task: {}", e))?;

    // Subscribe to task.complete and wait for our result
    let mut subscription = client
        .subscribe("task.complete")
        .await
        .map_err(|e| format!("Failed to subscribe to task.complete: {}", e))?;

    let deadline = Duration::from_secs(30);
    let start = tokio::time::Instant::now();

    while start.elapsed() < deadline {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        match timeout(remaining, subscription.next()).await {
            Ok(Some(msg)) => {
                let env: Envelope<TaskComplete> = serde_json::from_slice(&msg.payload)
                    .map_err(|e| format!("Failed to parse task.complete: {}", e))?;

                if env.correlation_id == correlation_id {
                    return Ok(env.payload);
                }
            }
            Ok(None) => return Err("Subscription ended unexpectedly".to_string()),
            Err(_) => return Err("Timeout waiting for task.complete".to_string()),
        }
    }

    Err("Timeout waiting for task completion".to_string())
}

#[tokio::test]
#[ignore = "Requires running NATS server and modules"]
async fn test_end_to_end_message_flow() {
    let client = create_test_client().await;

    let result = submit_task_and_wait(&client, "test task").await;
    assert!(result.is_ok(), "Task should complete successfully");

    let complete = result.unwrap();
    assert!(!complete.result.is_empty(), "Result should not be empty");
    assert_eq!(complete.side_effects.len(), 0, "No side effects for simple test");
}

#[tokio::test]
#[ignore = "Requires running NATS server and modules"]
async fn test_multiple_concurrent_tasks() {
    let client = create_test_client().await;

    let handles: Vec<_> = (0..3)
        .map(|i| {
            let client = client.clone();
            tokio::spawn(async move {
                submit_task_and_wait(&client, &format!("concurrent task {}", i)).await
            })
        })
        .collect();

    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent task should complete successfully");
    }
}

#[tokio::test]
#[ignore = "Requires running NATS server and modules"]
async fn test_task_envelope_validation() {
    let client = create_test_client().await;

    let session_id = format!("test_session_{}", uuid::Uuid::new_v4());
    let submitted = TaskSubmitted {
        session_id: session_id.clone(),
        user_input: "validation test".to_string(),
        submitted_at: chrono::Utc::now().timestamp(),
    };

    let envelope = Envelope::new("task.submitted", submitted, Some(session_id));

    // Validate envelope structure
    assert!(!envelope.message_id.is_empty(), "Message ID should not be empty");
    assert!(!envelope.session_id.is_empty(), "Session ID should not be empty");
    assert!(!envelope.correlation_id.is_empty(), "Correlation ID should not be empty");
    assert_eq!(envelope.topic, "task.submitted", "Topic should be task.submitted");
    assert_eq!(envelope.schema_version, 1, "Schema version should be 1");

    // Validate envelope
    let validation_result = envelope.validate();
    assert!(validation_result.is_ok(), "Envelope should validate successfully");
}

#[tokio::test]
#[ignore = "Requires running NATS server and modules"]
async fn test_correlation_tracking() {
    let client = create_test_client().await;

    let session_id = format!("test_session_{}", uuid::Uuid::new_v4());
    let submitted = TaskSubmitted {
        session_id: session_id.clone(),
        user_input: "correlation test".to_string(),
        submitted_at: chrono::Utc::now().timestamp(),
    };

    let envelope = Envelope::new("task.submitted", submitted, Some(session_id));
    let correlation_id = envelope.correlation_id.clone();

    // Create child envelope
    let child = envelope.child("test.topic", serde_json::Value::Null, 1);
    assert_eq!(child.correlation_parent(), correlation_id, "Child should inherit parent correlation");
    assert_ne!(child.correlation_id, correlation_id, "Child should have unique correlation ID");

    // Create reply envelope
    let reply = envelope.reply("test.reply", serde_json::Value::Null);
    assert_eq!(reply.session_id, envelope.session_id, "Reply should inherit session ID");
    assert_ne!(reply.correlation_id, correlation_id, "Reply should have new correlation ID");
}
