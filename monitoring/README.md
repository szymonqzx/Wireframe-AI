# Wireframe-AI Monitoring

This module provides monitoring and tracing infrastructure for Wireframe-AI.

## Components

### Metrics (`metrics.rs`)

Prometheus-compatible metrics collection for monitoring system performance.

**Metrics Collected:**
- NATS message metrics (published, received, processing duration)
- Plugin metrics (initializations, health checks, executions, duration)
- Task metrics (submitted, completed, failed, duration)
- Resource metrics (active connections, memory usage, CPU usage)

**Usage:**

```rust
use monitoring::metrics::{MetricsCollector, global_metrics};

// Use global metrics
global_metrics().record_message_published("task.submitted");

// Or create custom collector
let metrics = MetricsCollector::new();
metrics.record_task_submitted("session-123");
let result = metrics.record_task_duration(|| {
    // Your task processing logic
});
```

### Tracing (`tracing.rs`)

OpenTelemetry-compatible distributed tracing for monitoring and debugging.

**Features:**
- Span creation for NATS messages, plugin operations, and tasks
- Error recording and attribute tracking
- Automatic span lifecycle management
- Multiple exporter support (stdout, Jaeger, Zipkin)

**Usage:**

```rust
use monitoring::tracing::{init_tracing, TracingConfig, create_task_span, with_span};

// Initialize tracing
let config = TracingConfig::default();
init_tracing(&config)?;

// Create and use spans
let span = create_task_span("session-123");
let result = with_span(span, || {
    // Your operation
});
```

## Configuration

### Metrics Configuration

Metrics are automatically collected and exposed on `/metrics` endpoint when the metrics server is running.

### Tracing Configuration

Configure tracing via environment variables:

```bash
export WIREFRAME_TRACING_ENABLED=true
export WIREFRAME_TRACING_SAMPLING_RATE=0.1
export WIREFRAME_TRACING_EXPORTER=stdout  # or jaeger, zipkin
```

## Exporters

### Stdout Exporter

Default exporter that prints traces to stdout. Useful for development.

### Jaeger Exporter

Enable with `jaeger` feature:

```toml
[dependencies]
wireframe-monitoring = { path = "../monitoring", features = ["jaeger"] }
```

Configure Jaeger endpoint:

```bash
export JAEGER_AGENT_HOST=localhost
export JAEGER_AGENT_PORT=6831
```

### Zipkin Exporter

Enable with `zipkin` feature:

```toml
[dependencies]
wireframe-monitoring = { path = "../monitoring", features = ["zipkin"] }
```

Configure Zipkin endpoint:

```bash
export ZIPKIN_ENDPOINT=http://localhost:9411/api/v2/spans
```

## Prometheus Integration

To expose metrics for Prometheus:

```rust
use monitoring::metrics::global_metrics;
use prometheus::{Encoder, TextEncoder};

fn metrics_handler() -> String {
    let metrics = global_metrics();
    let encoder = TextEncoder::new();
    let metric_families = metrics.registry().gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}
```

## Grafana Dashboards

Import the provided Grafana dashboard configuration:

```json
{
  "dashboard": {
    "title": "Wireframe-AI Metrics",
    "panels": [
      {
        "title": "NATS Message Rate",
        "targets": [
          {
            "expr": "rate(nats_messages_published_total[1m])"
          }
        ]
      },
      {
        "title": "Task Duration",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, task_duration_seconds)"
          }
        ]
      }
    ]
  }
}
```

## Best Practices

1. **Use global metrics** for consistent monitoring across modules
2. **Create meaningful spans** with descriptive names
3. **Record errors** in spans for debugging
4. **Add attributes** to spans for context
5. **Use sampling** in production to reduce overhead
6. **Monitor resource usage** to detect issues early

## Troubleshooting

### Metrics Not Appearing

- Ensure metrics collector is initialized
- Check that metrics are being recorded
- Verify Prometheus is scraping the metrics endpoint

### Traces Not Appearing

- Check that tracing is enabled
- Verify exporter configuration
- Ensure sampling rate is appropriate
- Check exporter connectivity (Jaeger/Zipkin)

### High Overhead

- Reduce sampling rate
- Disable tracing in production if not needed
- Use stdout exporter for development only

## Next Steps

- See [Performance Optimization Guide](../docs/Performance-Optimization-Guide.md) for optimization techniques
- See [Production Deployment Guide](../docs/Production-Deployment-Guide.md) for deployment strategies
