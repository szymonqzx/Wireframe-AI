//! Distributed tracing for Wireframe-AI
//!
//! This module provides OpenTelemetry-compatible distributed tracing
//! for monitoring system performance and debugging.

use opentelemetry::trace::{Span, Tracer};
use opentelemetry::{global, Context};
use std::time::Instant;

/// Tracing configuration
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Enable tracing
    pub enabled: bool,
    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,
    /// Exporter type (stdout, jaeger, zipkin)
    pub exporter_type: ExporterType,
}

/// Tracing exporter type
#[derive(Debug, Clone, Copy)]
pub enum ExporterType {
    Stdout,
    Jaeger,
    Zipkin,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            sampling_rate: 0.1,
            exporter_type: ExporterType::Stdout,
        }
    }
}

/// Initialize tracing with the given configuration
pub fn init_tracing(config: &TracingConfig) -> Result<(), Box<dyn std::error::Error>> {
    if !config.enabled {
        println!("Tracing disabled");
        return Ok(());
    }

    // Initialize OpenTelemetry
    let exporter = match config.exporter_type {
        ExporterType::Stdout => {
            use opentelemetry_stdout::SpanExporter;
            SpanExporter::new()
        }
        ExporterType::Jaeger => {
            use opentelemetry_jaeger::Propagator;
            // Jaeger exporter setup
            return Err("Jaeger exporter not yet implemented".into());
        }
        ExporterType::Zipkin => {
            // Zipkin exporter setup
            return Err("Zipkin exporter not yet implemented".into());
        }
    };

    let provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_simple_exporter(exporter)
        .build();

    global::set_provider(provider);

    println!("Tracing initialized with sampling rate: {}", config.sampling_rate);
    Ok(())
}

/// Create a span for NATS message processing
pub fn create_nats_span(topic: &str) -> Span {
    let tracer = global::tracer("nats");
    tracer.start(format!("process_{}", topic.replace('.', "_")))
}

/// Create a span for plugin execution
pub fn create_plugin_span(plugin_id: &str, operation: &str) -> Span {
    let tracer = global::tracer("plugin");
    tracer.start(format!("{}_{}", plugin_id, operation))
}

/// Create a span for task processing
pub fn create_task_span(session_id: &str) -> Span {
    let tracer = global::tracer("task");
    tracer.start(format!("task_{}", session_id))
}

/// Record a span error
pub fn record_span_error(span: &Span, error: &str) {
    span.record("error", error);
}

/// Record a span attribute
pub fn record_span_attribute(span: &Span, key: &str, value: &str) {
    span.set_attribute(key.to_string(), value.to_string());
}

/// Execute a function within a span
pub fn with_span<F, R>(span: Span, f: F) -> R
where
    F: FnOnce() -> R,
{
    let _guard = span.enter();
    f()
}

/// Execute a function within a span with error handling
pub fn with_span_result<F, R, E>(span: Span, f: F) -> Result<R, E>
where
    F: FnOnce() -> Result<R, E>,
    E: std::fmt::Display,
{
    let _guard = span.enter();
    match f() {
        Ok(result) => Ok(result),
        Err(e) => {
            record_span_error(&span, &e.to_string());
            Err(e)
        }
    }
}

/// Measure execution time within a span
pub fn measure_span<F, R>(span: &Span, f: F) -> R
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed().as_secs_f64();
    span.set_attribute("duration_secs", duration);
    result
}

/// Tracing guard for automatic span lifecycle management
pub struct SpanGuard {
    span: Span,
}

impl SpanGuard {
    /// Create a new span guard
    pub fn new(span: Span) -> Self {
        Self { span }
    }
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        // Span is automatically ended when guard is dropped
    }
}

/// Helper macro for creating and managing spans
#[macro_export]
macro_rules! trace_span {
    ($name:expr, $block:expr) => {{
        let span = $crate::monitoring::tracing::create_task_span($name);
        let _guard = span.enter();
        $block
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_config_default() {
        let config = TracingConfig::default();
        assert!(config.enabled);
        assert_eq!(config.sampling_rate, 0.1);
    }

    #[test]
    fn test_init_tracing_disabled() {
        let config = TracingConfig {
            enabled: false,
            ..Default::default()
        };
        assert!(init_tracing(&config).is_ok());
    }

    #[test]
    fn test_create_nats_span() {
        let span = create_nats_span("task.submitted");
        assert_eq!(span.name(), "process_task_submitted");
    }

    #[test]
    fn test_create_plugin_span() {
        let span = create_plugin_span("storage-sqlite", "ensure_session");
        assert_eq!(span.name(), "storage-sqlite_ensure_session");
    }

    #[test]
    fn test_create_task_span() {
        let span = create_task_span("test-session");
        assert_eq!(span.name(), "task_test-session");
    }

    #[test]
    fn test_record_span_attribute() {
        let span = create_task_span("test");
        record_span_attribute(&span, "key", "value");
    }

    #[test]
    fn test_with_span() {
        let span = create_task_span("test");
        let result = with_span(span, || 42);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_with_span_result_ok() {
        let span = create_task_span("test");
        let result = with_span_result(span, || Ok::<i32, String>(42));
        assert!(result.is_ok());
    }

    #[test]
    fn test_with_span_result_err() {
        let span = create_task_span("test");
        let result = with_span_result(span, || Err::<i32, String>("error".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_measure_span() {
        let span = create_task_span("test");
        let result = measure_span(&span, || {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });
        assert_eq!(result, 42);
    }
}
