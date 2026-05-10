💡 **What:** Adds a `fan_out_bench` binary to `agentic-sdk` for measuring the performance of `fan_out` in `orchestrator_patterns.rs`, and adds the missing `tempfile` dev-dependency required by tests in `config.rs`.

🎯 **Why:** An initial attempt to optimize `fan_out` by hoisting `correlation_id` and `context` clones out of the loop was reverted: because `AgentJob.correlation_parent` is an owned `String` and `AgentJob.context` is an owned `HashMap`, the per-iteration clones cannot be avoided that way, and hoisting only added 2 extra allocations on top of the existing `2N`. The remaining changes are the benchmark binary (useful for future, evidence-backed optimization attempts) and a minor formatting tweak in `fan_out`.

📊 **Benchmark:** Run with `cargo run -p agentic-sdk --bin fan_out_bench --release`. Results from a single, un-warmed run with 200,000 sub-tasks were ~860–890ms; differences within that range are within measurement noise for this methodology.
