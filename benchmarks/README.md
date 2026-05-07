# Wireframe-AI Plugin Benchmarks

This directory contains performance benchmarks for Wireframe-AI plugins.

## Running Benchmarks

### Basic Benchmarks

Run all benchmarks:

```bash
cargo bench --bench plugin_benchmarks
```

Run specific benchmark groups:

```bash
cargo bench --bench plugin_benchmarks -- plugin_registry
cargo bench --bench plugin_benchmarks -- storage_backend
cargo bench --bench plugin_benchmarks -- task_planner
cargo bench --bench plugin_benchmarks -- tool
cargo bench --bench plugin_benchmarks -- plugin_lifecycle
cargo bench --bench plugin_benchmarks -- concurrent_operations
cargo bench --bench plugin_benchmarks -- memory_usage
```

### Detailed Output

For more detailed output with statistics:

```bash
cargo bench --bench plugin_benchmarks -- --verbose
```

### Flamegraphs

For flamegraph visualization:

```bash
cargo bench --bench plugin_benchmarks -- --profile-time 10
```

This will generate flamegraphs in `target/criterion/`.

### Custom Sample Size

Adjust the number of samples for more accurate results:

```bash
cargo bench --bench plugin_benchmarks -- --sample-size 1000
```

## Benchmark Groups

### Plugin Registry

Benchmarks for plugin registry operations:
- `plugin_registry_register` - Register a new plugin
- `plugin_registry_get` - Retrieve a plugin by ID
- `plugin_registry_is_registered` - Check if a plugin is registered
- `plugin_registry_list_plugins` - List all registered plugins

### Storage Backend

Benchmarks for storage backend operations:
- `storage_ensure_session` - Ensure a session exists
- `storage_store_message` - Store a message in a session
- `storage_load_session_history` - Load session history

### Task Planner

Benchmarks for task planner operations:
- `planner_decompose` - Decompose a simple task
- `planner_decompose_complex` - Decompose a complex task with many subtasks

### Tool

Benchmarks for tool operations:
- `tool_execute` - Execute a tool with standard parameters
- `tool_execute_large_payload` - Execute a tool with large payload
- `tool_input_schema` - Get tool input schema

### Plugin Lifecycle

Benchmarks for plugin lifecycle operations:
- `plugin_initialize` - Initialize a plugin
- `plugin_health_check` - Perform health check
- `plugin_shutdown` - Shutdown a plugin

### Concurrent Operations

Benchmarks for concurrent operations:
- `concurrent_store_message` - Store messages concurrently (10, 50, 100 tasks)

### Memory Usage

Benchmarks for memory usage patterns:
- `memory_large_context_package` - Handle large context packages
- `memory_large_message_history` - Handle large message histories

## Interpreting Results

Benchmark results are saved in `target/criterion/`. Each benchmark group has its own directory with:

- `report/index.html` - HTML report with charts and statistics
- `baseline/` - Baseline comparison data
- `new/` - Latest benchmark data

### Key Metrics

- **Time**: Mean execution time
- **Std Dev**: Standard deviation of execution times
- **Median**: Median execution time
- **Throughput**: Operations per second

## Adding New Benchmarks

To add a new benchmark:

1. Add a new benchmark function to `plugin_benchmarks.rs`
2. Create a benchmark group using `criterion_group!`
3. Add the group to `criterion_main!`

Example:

```rust
fn bench_my_new_feature(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("my_new_feature", |b| {
        b.to_async(&rt).iter(|| async {
            // Your benchmark code here
            black_box(async_operation().await);
        });
    });
}

criterion_group!(
    name = my_feature;
    config = Criterion::default().sample_size(100);
    targets = bench_my_new_feature
);

criterion_main!(
    // ... existing groups ...
    my_feature
);
```

## Performance Targets

Current performance targets for Wireframe-AI plugins:

| Operation | Target | Current |
|-----------|--------|---------|
| Plugin registration | < 1ms | TBD |
| Plugin retrieval | < 100µs | TBD |
| Session ensure | < 5ms | TBD |
| Message store | < 10ms | TBD |
| History load (100) | < 50ms | TBD |
| Task decomposition | < 100ms | TBD |
| Tool execution | < 50ms | TBD |
| Health check | < 10ms | TBD |

## Profiling

For detailed profiling, use:

```bash
# Install flamegraph
cargo install flamegraph

# Run with flamegraph
cargo flamegraph --bench plugin_benchmarks
```

This generates a flamegraph SVG showing where time is spent.

## Continuous Integration

Benchmarks can be integrated into CI to detect performance regressions:

```yaml
# Example GitHub Actions workflow
- name: Run benchmarks
  run: cargo bench --bench plugin_benchmarks

- name: Upload benchmark results
  uses: actions/upload-artifact@v3
  with:
    name: benchmark-results
    path: target/criterion/
```

## Comparison

To compare against a baseline:

```bash
# Save current results as baseline
cargo bench --bench plugin_benchmarks -- --save-baseline main

# Make changes
# ...

# Compare against baseline
cargo bench --bench plugin_benchmarks -- --baseline main
```

## Troubleshooting

### Inconsistent Results

If benchmark results are inconsistent:
- Close other applications
- Run on a quiet system
- Increase sample size with `--sample-size`
- Use `--warmup-time` to allow CPU to stabilize

### Out of Memory

If benchmarks run out of memory:
- Reduce the size of test data
- Run benchmarks individually
- Check for memory leaks in benchmark code

### Slow Benchmarks

If benchmarks are too slow:
- Reduce the number of iterations
- Use smaller test datasets
- Profile to find bottlenecks

## Best Practices

1. **Use `black_box`**: Always wrap values in `black_box()` to prevent compiler optimizations
2. **Async runtime**: Use `to_async(&rt)` for async benchmarks
3. **Sample size**: Use appropriate sample sizes for statistical significance
4. **Warmup**: Allow warmup time for JIT compilation and CPU scaling
5. **Isolation**: Run benchmarks on isolated systems for consistent results
6. **Documentation**: Document what each benchmark measures

## Resources

- [Criterion Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Flamegraph Documentation](https://github.com/flamegraph-rs/flamegraph)
