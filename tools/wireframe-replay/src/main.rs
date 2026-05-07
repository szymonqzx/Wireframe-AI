//! wireframe-replay — Message Replay and Simulation Tool
//!
//! Reads captured messages from a JSONL file and replays them
//! to NATS at configurable speed with optional filtering.
//!
//! Usage:
//!   wireframe-replay capture.jsonl
//!   wireframe-replay capture.jsonl --speed 2.0 --topic task.>
//!   wireframe-replay capture.jsonl --nats-url nats://remote:4222

use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use tokio::time::{sleep, Duration};

#[derive(Parser)]
#[command(name = "wireframe-replay")]
#[command(about = "Wireframe AI — message replay and simulation")]
#[command(version = "0.1.0")]
struct Cli {
    /// Capture file (.jsonl)
    file: String,

    /// NATS URL to replay to
    #[arg(short, long, default_value = "nats://localhost:4222")]
    nats_url: String,

    /// Replay speed multiplier (1.0 = real-time, 2.0 = 2x)
    #[arg(short, long, default_value = "1.0")]
    speed: f64,

    /// Filter by topic pattern
    #[arg(short, long)]
    topic: Option<String>,

    /// Dry run: print what would be replayed without sending
    #[arg(long)]
    dry_run: bool,

    /// Loop replay indefinitely
    #[arg(long)]
    loop_replay: bool,
}

#[derive(Serialize, Deserialize)]
struct CaptureEntry {
    timestamp: i64,
    topic: String,
    payload: Value,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("Reading capture file: {}", cli.file);
    let entries = read_capture_file(&cli.file, &cli.topic)?;
    println!("Loaded {} messages", entries.len());

    if entries.is_empty() {
        println!("No messages to replay.");
        return Ok(());
    }

    let client = if !cli.dry_run {
        println!("Connecting to NATS at {}...", cli.nats_url);
        let c = async_nats::connect(&cli.nats_url).await?;
        println!("Connected.");
        Some(c)
    } else {
        println!("DRY RUN — messages will not be sent.");
        None
    };

    let mut iteration = 0;
    loop {
        iteration += 1;
        if cli.loop_replay {
            println!("\n--- Replay iteration {} ---", iteration);
        }

        let mut last_ts = entries.first().map(|e| e.timestamp).unwrap_or(0);
        let mut replayed = 0;
        let mut skipped = 0;

        for entry in &entries {
            // Compute delay based on timestamp difference (saturating to avoid overflow)
            let delta = entry.timestamp.saturating_sub(last_ts);
            let delay_ms = (delta.saturating_mul(1000)) as f64 / cli.speed;
            last_ts = entry.timestamp;

            if delay_ms > 0.0 && delay_ms < 30000.0 {
                sleep(Duration::from_millis(delay_ms as u64)).await;
            }

            if let Some(client) = &client {
                let payload = serde_json::to_vec(&entry.payload)?;
                if let Err(e) = client.publish(entry.topic.clone(), payload.into()).await {
                    tracing::warn!(topic = %entry.topic, error = %e, "publish failed");
                    skipped += 1;
                } else {
                    replayed += 1;
                }
            } else {
                println!(
                    "  [{}] {}",
                    entry.topic,
                    serde_json::to_string(&entry.payload).unwrap_or_default()
                );
                replayed += 1;
            }
        }

        println!("Replayed {} messages, skipped {}", replayed, skipped);

        if !cli.loop_replay {
            break;
        }
    }

    Ok(())
}

fn read_capture_file(path: &str, topic_filter: &Option<String>) -> Result<Vec<CaptureEntry>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let entry: CaptureEntry = serde_json::from_str(&line)?;

        if let Some(filter) = topic_filter {
            if !nats_topic_matches(&entry.topic, filter) {
                continue;
            }
        }

        entries.push(entry);
    }

    // Sort by timestamp
    entries.sort_by_key(|e| e.timestamp);
    Ok(entries)
}

/// Check whether a NATS topic matches a pattern that may contain wildcards.
///
/// - `>` matches any number of tokens at the end (e.g. `task.>` matches `task.a.b`).
/// - `*` matches exactly one token (e.g. `task.*.done` matches `task.foo.done`).
fn nats_topic_matches(topic: &str, pattern: &str) -> bool {
    if topic == pattern {
        return true;
    }
    let topic_parts: Vec<&str> = topic.split('.').collect();
    let pattern_parts: Vec<&str> = pattern.split('.').collect();

    let mut t = 0usize;
    let mut p = 0usize;

    while t < topic_parts.len() && p < pattern_parts.len() {
        let pat = pattern_parts[p];
        if pat == ">" {
            // > matches zero or more remaining tokens.
            return true;
        }
        if pat == "*" {
            // * matches exactly one token.
            t += 1;
            p += 1;
            continue;
        }
        if topic_parts[t] != pat {
            return false;
        }
        t += 1;
        p += 1;
    }

    // Exact match if both exhausted.
    t == topic_parts.len() && p == pattern_parts.len()
}
