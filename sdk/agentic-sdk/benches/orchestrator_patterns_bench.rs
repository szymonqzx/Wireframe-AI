use agentic_sdk::message_types::TaskEnriched;
use agentic_sdk::orchestrator_patterns::fan_out;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_fan_out(c: &mut Criterion) {
    let mut group = c.benchmark_group("fan_out");

    let enriched = TaskEnriched {
        session_id: "s1".to_string(),
        correlation_id: "c1".to_string(),
        user_input: "test".to_string(),
        context: Default::default(),
        inferred_constraints: vec![],
        enriched_at: 0,
    };

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("sub_tasks", size), size, |b, &size| {
            b.iter_with_setup(
                || {
                    let mut sub_tasks = Vec::with_capacity(size);
                    for i in 0..size {
                        sub_tasks.push(format!("Subtask number {}", i));
                    }
                    sub_tasks
                },
                |sub_tasks| {
                    black_box(fan_out(&enriched, sub_tasks));
                },
            );
        });
    }

    group.finish();
}

criterion_group!(benches, bench_fan_out);
criterion_main!(benches);
