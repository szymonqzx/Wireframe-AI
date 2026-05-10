use agentic_sdk::message_types::TaskEnriched;
use agentic_sdk::orchestrator_patterns::fan_out;
use std::time::Instant;

fn main() {
    let enriched = TaskEnriched {
        session_id: "test-session".to_string(),
        correlation_id: "test-correlation".to_string(),
        user_input: "test".to_string(),
        context: Default::default(),
        inferred_constraints: vec![],
        enriched_at: 0,
    };

    let sub_tasks = vec!["this is a relatively long subtask description string to simulate a real subtask payload and make cloning somewhat expensive".to_string(); 200_000];

    println!(
        "Starting optimized benchmark with {} items...",
        sub_tasks.len()
    );
    let start = Instant::now();
    let jobs = fan_out(&enriched, sub_tasks);
    let elapsed = start.elapsed();

    println!("fan_out took: {:?}", elapsed);
    println!("Generated {} jobs", jobs.len());
}
