use agentic_sdk::serialization::BatchSerializer;
use serde::Serialize;
use std::time::Instant;

#[derive(Serialize, Clone)]
struct DummyData {
    id: i32,
    name: String,
    value: f64,
}

fn main() {
    let mut serializer = BatchSerializer::new();
    let num_items = 500_000; // Large number to make cloning time prominent

    let mut items = Vec::with_capacity(num_items);
    for i in 0..num_items {
        items.push((
            format!("a_very_long_key_string_that_will_take_some_time_to_clone_during_the_serialization_process_{}", i),
            DummyData {
                id: i as i32,
                name: format!("name_{}", i),
                value: i as f64 * std::f64::consts::PI,
            },
        ));
    }

    // Benchmark serialize_batch_with_owned_keys (the optimized, owned variant)
    // Run a few times to warmup
    for _ in 0..3 {
        let _ = serializer
            .serialize_batch_with_owned_keys(items.clone())
            .unwrap();
    }

    let start = Instant::now();
    let _ = serializer.serialize_batch_with_owned_keys(items).unwrap();
    let duration = start.elapsed();

    println!(
        "serialize_batch_with_owned_keys ({} items): {:?}",
        num_items, duration
    );
}
