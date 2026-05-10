use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use agentic_sdk::module::EnvelopePublisher;
use async_nats::Client;

/// Helper function to mock a NATS server locally for testing.
async fn mock_nats_client() -> Client {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        if let Ok((mut socket, _)) = listener.accept().await {
            // Send INFO
            let info = b"INFO {\"server_id\":\"mock\",\"version\":\"2.9.15\",\"proto\":1,\"go\":\"go1.19.8\",\"host\":\"0.0.0.0\",\"port\":4222,\"headers\":true,\"max_payload\":1048576,\"client_id\":1,\"client_ip\":\"127.0.0.1\"}\r\n";
            let _ = socket.write_all(info).await;

            // Keep connection alive and respond to PING
            let mut buf = [0; 1024];
            while let Ok(n) = socket.read(&mut buf).await {
                if n == 0 {
                    break;
                }
                let text = String::from_utf8_lossy(&buf[..n]);
                if text.contains("PING") {
                    let _ = socket.write_all(b"PONG\r\n").await;
                }
            }
        }
    });

    async_nats::connect(format!("nats://127.0.0.1:{}", port))
        .await
        .unwrap()
}

#[tokio::test]
async fn test_envelope_publisher_new() {
    let nc = Arc::new(mock_nats_client().await);
    let publisher = EnvelopePublisher::new(nc.clone());

    assert_eq!(publisher.buffer_size().await, None);
}

#[tokio::test]
async fn test_envelope_publisher_with_buffering() {
    let nc = Arc::new(mock_nats_client().await);
    let max_size = 100;
    let max_age = Duration::from_millis(50);

    let publisher = EnvelopePublisher::new(nc.clone()).with_buffering(max_size, max_age);

    assert_eq!(publisher.buffer_size().await, Some(0));
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TestMessage {
    field: String,
}

#[tokio::test]
async fn test_envelope_publisher_publish() {
    let nc = Arc::new(mock_nats_client().await);
    let publisher = EnvelopePublisher::new(nc.clone());

    let payload = TestMessage {
        field: "test".to_string(),
    };
    let envelope = agentic_sdk::envelope::Envelope::new("test.subject", payload, None);

    // Test publish_immediate
    let result = publisher
        .publish_immediate("test.subject".to_string(), &envelope)
        .await;
    assert!(result.is_ok());

    // Test normal publish (unbuffered)
    let result = publisher
        .publish("test.subject".to_string(), &envelope)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_envelope_publisher_publish_buffered() {
    let nc = Arc::new(mock_nats_client().await);
    let max_size = 100;
    let max_age = Duration::from_millis(50);

    let publisher = EnvelopePublisher::new(nc.clone()).with_buffering(max_size, max_age);

    let payload = TestMessage {
        field: "test".to_string(),
    };
    let envelope = agentic_sdk::envelope::Envelope::new("test.subject", payload, None);

    // Test buffered publish
    let result = publisher
        .publish("test.subject".to_string(), &envelope)
        .await;
    assert!(result.is_ok());

    // Check that it's buffered
    assert_eq!(publisher.buffer_size().await, Some(1));

    // Test flush
    publisher.flush().await;
    assert_eq!(publisher.buffer_size().await, Some(0));
}
