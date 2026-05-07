//! wireframe-debug — Message Inspector for Wireframe AI
//!
//! Connects to NATS and streams messages in real-time with filtering,
//! pretty-printing, and capture capabilities.
//!
//! Usage:
//!   wireframe-debug --topic task.>
//!   wireframe-debug --correlation abc-123 --format json
//!   wireframe-debug --topic > --capture messages.jsonl

use anyhow::Result;
use clap::Parser;
use futures::StreamExt;
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Parser)]
#[command(name = "wireframe-debug")]
#[command(about = "Wireframe AI — message inspector and debugger")]
#[command(version = "0.1.0")]
struct Cli {
    /// NATS URL
    #[arg(short, long, default_value = "nats://localhost:4222")]
    nats_url: String,

    /// Topic pattern to subscribe to
    #[arg(short, long, default_value = ">")]
    topic: String,

    /// Filter by correlation_id
    #[arg(short, long)]
    correlation: Option<String>,

    /// Output format: pretty, json, compact
    #[arg(short, long, default_value = "pretty")]
    format: String,

    /// Capture messages to file
    #[arg(short, long)]
    capture: Option<String>,

    /// Show only envelope headers (no payload)
    #[arg(long)]
    headers_only: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    println!("Connecting to NATS at {}...", cli.nats_url);
    let client = async_nats::connect(&cli.nats_url).await?;
    println!("Subscribing to topic: {}", cli.topic);

    let mut subscriber = client.subscribe(cli.topic.clone()).await?;
    println!("Listening for messages. Press Ctrl+C to stop.\n");

    let mut capture_file = if let Some(path) = &cli.capture {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        println!("Capturing to: {}\n", path);
        Some(file)
    } else {
        None
    };

    while let Some(message) = subscriber.next().await {
        let topic = message.subject.to_string();
        let payload = String::from_utf8_lossy(&message.payload);

        // Parse payload
        let parsed: Value = match serde_json::from_str(&payload) {
            Ok(v) => v,
            Err(_) => {
                // Not JSON, print raw
                if cli.headers_only {
                    println!("[{}] (raw, {} bytes)", topic, message.payload.len());
                } else {
                    println!("[{}] {}", topic, payload);
                }
                continue;
            }
        };

        // Filter by correlation_id
        if let Some(corr) = &cli.correlation {
            let msg_corr = parsed
                .get("correlation_id")
                .and_then(|v| v.as_str())
                .or_else(|| parsed.get("correlation_parent").and_then(|v| v.as_str()));
            if msg_corr != Some(corr) {
                continue;
            }
        }

        // Format output
        let output = match cli.format.as_str() {
            "json" => serde_json::to_string(&parsed).unwrap_or_default(),
            "compact" => format!(
                "[{}] correlation={} topic={}",
                topic,
                parsed
                    .get("correlation_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("-"),
                parsed.get("topic").and_then(|v| v.as_str()).unwrap_or("-"),
            ),
            _ => {
                // pretty
                let pretty = serde_json::to_string_pretty(&parsed).unwrap_or_default();
                format!(
                    "\n[{}] {}\n{}",
                    chrono::Utc::now().to_rfc3339(),
                    topic,
                    pretty
                )
            }
        };

        if cli.headers_only {
            let corr = parsed
                .get("correlation_id")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            let sess = parsed
                .get("session_id")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            println!(
                "[{}] corr={} sess={} ({} bytes)",
                topic,
                corr,
                sess,
                payload.len()
            );
        } else {
            println!("{}", output);
        }

        // Capture to file
        if let Some(file) = &mut capture_file {
            let line = serde_json::json!({
                "timestamp": chrono::Utc::now().timestamp(),
                "topic": topic,
                "payload": parsed,
            });
            if let Ok(jsonl) = serde_json::to_string(&line) {
                let _ = writeln!(file, "{}", jsonl);
            }
        }
    }

    Ok(())
}
