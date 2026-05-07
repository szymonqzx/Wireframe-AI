//! Benchmark: measure pipeline throughput under load.
//!
//! Tests how many messages the envelope serialization + NATS publish cycle
//! can handle per second.
//!
//! Run: cargo test --test benchmark_test -- --nocapture
//! Requires a running NATS server on localhost:4222.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use serde_json::Value;

use agentic_sdk::envelope::Envelope;
use agentic_sdk::message_types::{
    AgentJob, ChatMessage, ContextPackage, ExecutionConstraints, JobMetadata, MemoryChunk,
    ModelConfig, TaskDescription,
};

/// Connect to NATS or skip if not available.
async fn connect_or_skip() -> Option<async_nats::Client> {
    match async_nats::connect("nats://localhost:4222").await {
        Ok(c) => Some(c),
        Err(_) => {
            eprintln!("Skipping benchmark — no NATS server on localhost:4222");
            None
        }
    }
}

#[tokio::test]
async fn bench_envelope_serialization() {
    // Benchmark 1: Serialization throughput (pure CPU, no NATS)
    let payload = serde_json::json!({
        "data": "x".repeat(1024),
        "nested": {
            "field1": "value1",
            "field2": 42,
            "field3": [1, 2, 3, 4, 5],
        }
    });

    let count = 10_000;
    let start = Instant::now();

    for _ in 0..count {
        let env = Envelope::<Value>::new(
            "test.bench",
            payload.clone(),
            Some("bench_session".into()),
        );
        let _json = serde_json::to_string(&env).unwrap();
    }

    let elapsed = start.elapsed();
    let rate = count as f64 / elapsed.as_secs_f64();
    println!(
        "Benchmark: envelope serialization — {:.0} msg/s ({} in {:.2}s)",
        rate, count, elapsed.as_secs_f64()
    );
    assert!(
        rate > 5_000.0,
        "Serialization rate too low: {:.0} msg/s",
        rate
    );
}

#[tokio::test]
async fn bench_agentjob_roundtrip() {
    // Benchmark 2: Full AgentJob serialization roundtrip (the most common operation)
    let job = AgentJob {
        job_id: "bench-job-id".into(),
        correlation_parent: "bench-parent".into(),
        task: TaskDescription {
            user_input:
                "Write a Python script to sort CSV by column 2 and compute statistics".into(),
            sub_task: None,
            output_format: None,
            user_constraints: vec![],
        },
        context: ContextPackage {
            memory_chunks: vec![MemoryChunk {
                id: "mem_1".into(),
                content: "User prefers Python 3.11+".into(),
                source: "session_abc".into(),
                relevance_score: 0.85,
            }],
            session_history: vec![
                ChatMessage {
                    role: "user".into(),
                    content: "Help me sort CSV".into(),
                    timestamp: 1714880000,
                },
                ChatMessage {
                    role: "assistant".into(),
                    content: "I can help with that!".into(),
                    timestamp: 1714880001,
                },
            ],
            readonly_files: vec![],
            safe_env: HashMap::new(),
            working_dir: PathBuf::from("/tmp"),
            max_context_tokens: 32768,
        },
        available_tool_capabilities: vec![],
        constraints: ExecutionConstraints::default(),
        model_config: ModelConfig::default(),
        metadata: JobMetadata::default(),
        schema_version: 1,
        adapter_hints: None,
    };

    let count = 1_000;
    let start = Instant::now();

    for _ in 0..count {
        let env = Envelope::new("agent.job", &job, Some("bench_session".into()));
        let json = serde_json::to_string(&env).unwrap();
        let _parsed: Envelope<AgentJob> = serde_json::from_str(&json).unwrap();
    }

    let elapsed = start.elapsed();
    let rate = count as f64 / elapsed.as_secs_f64();
    println!(
        "Benchmark: AgentJob roundtrip — {:.0} msg/s ({} in {:.2}s)",
        rate, count, elapsed.as_secs_f64()
    );
    assert!(
        rate > 500.0,
        "AgentJob roundtrip rate too low: {:.0} msg/s",
        rate
    );
}

#[tokio::test]
async fn bench_nats_publish() {
    // Benchmark 3: NATS publish throughput (requires NATS server)
    use futures::StreamExt;

    let client = match connect_or_skip().await {
        Some(c) => c,
        None => return,
    };

    let payload = serde_json::json!({
        "message": "benchmark payload",
        "timestamp": 1714880000,
    });

    let count = 500;
    let batch_size = 50;

    // Subscribe first so we don't miss messages
    let mut sub = client.subscribe("bench.test").await.unwrap();

    let start = Instant::now();
    for i in 0..count {
        let env =
            Envelope::<Value>::new("bench.test", payload.clone(), Some("bench_pub".into()));
        let data = serde_json::to_vec(&env).unwrap();
        client.publish("bench.test", data.into()).await.unwrap();

        // Batch-drain every batch_size messages
        if i > 0 && i % batch_size == 0 {
            let drain_start = Instant::now();
            let mut received = 0;
            while received < batch_size {
                let _ = tokio::time::timeout(Duration::from_millis(500), sub.next()).await;
                received += 1;
            }
            let drain_time = drain_start.elapsed();
            if drain_time > Duration::from_millis(100) {
                // NATS may be falling behind — insert a small pause
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }

    // Drain remaining messages
    let remaining = count % batch_size;
    if remaining > 0 {
        let mut received = 0;
        while received < remaining {
            let _ = tokio::time::timeout(Duration::from_millis(500), sub.next()).await;
            received += 1;
        }
    }

    let elapsed = start.elapsed();
    let rate = count as f64 / elapsed.as_secs_f64();
    println!(
        "Benchmark: NATS publish — {:.0} msg/s ({} in {:.2}s)",
        rate, count, elapsed.as_secs_f64()
    );
    // Accept any positive rate (varies by machine)
    assert!(rate > 10.0, "NATS publish rate too low: {:.0} msg/s", rate);
}
