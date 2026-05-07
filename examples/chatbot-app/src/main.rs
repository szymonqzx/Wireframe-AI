//! Chatbot Application — Wireframe-AI Example
//!
//! A simple chatbot that reads user input from stdin,
//! submits tasks to the Wireframe-AI pipeline, and prints results.

use agentic_sdk::builders::TaskSubmittedBuilder;
use futures::StreamExt;
use tokio::io::{self, AsyncBufReadExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("Wireframe-AI Chatbot Example");
    println!("Connecting to NATS...");

    let client = async_nats::connect("nats://localhost:4222").await?;
    let mut subscriber = client.subscribe("task.complete").await?;

    println!("Connected. Type your message and press Enter.");
    println!("Type 'quit' to exit.\n");

    let stdin = io::BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    let client_clone = client.clone();
    tokio::spawn(async move {
        while let Some(msg) = subscriber.next().await {
            let payload = String::from_utf8_lossy(&msg.payload);
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&payload) {
                // task.complete is wrapped in an Envelope; the result lives inside payload.result
                let result = json
                    .get("payload")
                    .and_then(|p| p.get("result"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("(no result)");
                println!("\n[Bot] {}\n> ", result);
            }
        }
    });

    while let Some(line) = lines.next_line().await? {
        let input = line.trim();
        if input.eq_ignore_ascii_case("quit") {
            break;
        }
        if input.is_empty() {
            print!("> ");
            continue;
        }

        let session_id = format!("chatbot_{}", uuid::Uuid::new_v4());
        let envelope = TaskSubmittedBuilder::new()
            .session_id(&session_id)
            .user_input(input)
            .build_envelope()
            .unwrap();

        let payload = serde_json::to_vec(&envelope)?;
        if let Err(e) = client_clone.publish("task.submitted", payload.into()).await {
            eprintln!("Failed to send: {}", e);
        } else {
            println!("[You] {}\n", input);
        }
    }

    println!("Goodbye!");
    Ok(())
}
