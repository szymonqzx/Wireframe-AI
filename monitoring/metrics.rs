//! Metrics collection for Wireframe-AI
//!
//! This module provides Prometheus-compatible metrics collection
//! for monitoring system performance and plugin operations.

use anyhow::{Context, Result};
use prometheus::{Counter, Histogram, IntCounter, IntGauge, Registry};
use std::sync::Arc;
use std::time::Instant;

/// Metrics collector for Wireframe-AI
pub struct MetricsCollector {
    registry: Arc<Registry>,
    
    // NATS message metrics
    messages_published: IntCounter,
    messages_received: IntCounter,
    message_processing_duration: Histogram,
    
    // Plugin metrics
    plugin_initializations: IntCounter,
    plugin_health_checks: IntCounter,
    plugin_executions: IntCounter,
    plugin_execution_duration: Histogram,
    
    // Task metrics
    tasks_submitted: IntCounter,
    tasks_completed: IntCounter,
    tasks_failed: IntCounter,
    task_duration: Histogram,
    
    // Resource metrics
    active_connections: IntGauge,
    memory_usage: IntGauge,
    cpu_usage: IntGauge,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Result<Self> {
        let registry = Arc::new(Registry::new());

        // NATS message metrics
        let messages_published = IntCounter::new(
            "nats_messages_published_total",
            "Total number of messages published to NATS"
        ).context("Failed to create messages_published counter")?;

        let messages_received = IntCounter::new(
            "nats_messages_received_total",
            "Total number of messages received from NATS"
        ).context("Failed to create messages_received counter")?;

        let message_processing_duration = Histogram::with_opts(
            Histogram::opts(
                "nats_message_processing_duration_seconds",
                "Message processing duration in seconds",
                None
            ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0])
        ).context("Failed to create message_processing_duration histogram")?;

        // Plugin metrics
        let plugin_initializations = IntCounter::new(
            "plugin_initializations_total",
            "Total number of plugin initializations"
        ).context("Failed to create plugin_initializations counter")?;

        let plugin_health_checks = IntCounter::new(
            "plugin_health_checks_total",
            "Total number of plugin health checks"
        ).context("Failed to create plugin_health_checks counter")?;

        let plugin_executions = IntCounter::new(
            "plugin_executions_total",
            "Total number of plugin executions"
        ).context("Failed to create plugin_executions counter")?;

        let plugin_execution_duration = Histogram::with_opts(
            Histogram::opts(
                "plugin_execution_duration_seconds",
                "Plugin execution duration in seconds",
                None
            ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0])
        ).context("Failed to create plugin_execution_duration histogram")?;

        // Task metrics
        let tasks_submitted = IntCounter::new(
            "tasks_submitted_total",
            "Total number of tasks submitted"
        ).context("Failed to create tasks_submitted counter")?;

        let tasks_completed = IntCounter::new(
            "tasks_completed_total",
            "Total number of tasks completed"
        ).context("Failed to create tasks_completed counter")?;

        let tasks_failed = IntCounter::new(
            "tasks_failed_total",
            "Total number of tasks failed"
        ).context("Failed to create tasks_failed counter")?;

        let task_duration = Histogram::with_opts(
            Histogram::opts(
                "task_duration_seconds",
                "Task duration in seconds"
            ).None
            .buckets(vec![1.0, 5.0, 10.0, 30.0, 60.0, 300.0, 600.0])
        ).context("Failed to create task_duration histogram")?;

        // Resource metrics
        let active_connections = IntGauge::new(
            "active_connections",
            "Number of active connections"
        ).context("Failed to create active_connections gauge")?;

        let memory_usage = IntGauge::new(
            "memory_usage_bytes",
            "Memory usage in bytes"
        ).context("Failed to create memory_usage gauge")?;

        let cpu_usage = IntGauge::new(
            "cpu_usage_percent",
            "CPU usage percentage"
        ).context("Failed to create cpu_usage gauge")?;

        // Register all metrics
        registry.register(Box::new(messages_published.clone()))
            .context("Failed to register messages_published metric")?;
        registry.register(Box::new(messages_received.clone()))
            .context("Failed to register messages_received metric")?;
        registry.register(Box::new(message_processing_duration.clone()))
            .context("Failed to register message_processing_duration metric")?;
        registry.register(Box::new(plugin_initializations.clone()))
            .context("Failed to register plugin_initializations metric")?;
        registry.register(Box::new(plugin_health_checks.clone()))
            .context("Failed to register plugin_health_checks metric")?;
        registry.register(Box::new(plugin_executions.clone()))
            .context("Failed to register plugin_executions metric")?;
        registry.register(Box::new(plugin_execution_duration.clone()))
            .context("Failed to register plugin_execution_duration metric")?;
        registry.register(Box::new(tasks_submitted.clone()))
            .context("Failed to register tasks_submitted metric")?;
        registry.register(Box::new(tasks_completed.clone()))
            .context("Failed to register tasks_completed metric")?;
        registry.register(Box::new(tasks_failed.clone()))
            .context("Failed to register tasks_failed metric")?;
        registry.register(Box::new(task_duration.clone()))
            .context("Failed to register task_duration metric")?;
        registry.register(Box::new(active_connections.clone()))
            .context("Failed to register active_connections metric")?;
        registry.register(Box::new(memory_usage.clone()))
            .context("Failed to register memory_usage metric")?;
        registry.register(Box::new(cpu_usage.clone()))
            .context("Failed to register cpu_usage metric")?;

        Ok(Self {
            registry,
            messages_published,
            messages_received,
            message_processing_duration,
            plugin_initializations,
            plugin_health_checks,
            plugin_executions,
            plugin_execution_duration,
            tasks_submitted,
            tasks_completed,
            tasks_failed,
            task_duration,
            active_connections,
            memory_usage,
            cpu_usage,
        })
    }
    
    /// Get the Prometheus registry
    pub fn registry(&self) -> Arc<Registry> {
        Arc::clone(&self.registry)
    }
    
    /// Record a message published
    pub fn record_message_published(&self, topic: &str) {
        self.messages_published.inc();
        tracing::debug!("Message published to {}", topic);
    }
    
    /// Record a message received
    pub fn record_message_received(&self, topic: &str) {
        self.messages_received.inc();
        tracing::debug!("Message received from {}", topic);
    }
    
    /// Record message processing duration
    pub fn record_message_processing<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed().as_secs_f64();
        self.message_processing_duration.observe(duration);
        result
    }
    
    /// Record a plugin initialization
    pub fn record_plugin_initialization(&self, plugin_id: &str) {
        self.plugin_initializations.inc();
        tracing::debug!("Plugin initialized: {}", plugin_id);
    }
    
    /// Record a plugin health check
    pub fn record_plugin_health_check(&self, plugin_id: &str, healthy: bool) {
        self.plugin_health_checks.inc();
        tracing::debug!("Plugin health check: {} - {}", plugin_id, healthy);
    }
    
    /// Record a plugin execution
    pub fn record_plugin_execution<F, R>(&self, plugin_id: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.plugin_executions.inc();
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed().as_secs_f64();
        self.plugin_execution_duration.observe(duration);
        tracing::debug!("Plugin executed: {} in {:.3}s", plugin_id, duration);
        result
    }
    
    /// Record a task submitted
    pub fn record_task_submitted(&self, session_id: &str) {
        self.tasks_submitted.inc();
        tracing::debug!("Task submitted: {}", session_id);
    }
    
    /// Record a task completed
    pub fn record_task_completed(&self, session_id: &str, duration_secs: f64) {
        self.tasks_completed.inc();
        self.task_duration.observe(duration_secs);
        tracing::debug!("Task completed: {} in {:.3}s", session_id, duration_secs);
    }
    
    /// Record a task failed
    pub fn record_task_failed(&self, session_id: &str) {
        self.tasks_failed.inc();
        tracing::debug!("Task failed: {}", session_id);
    }
    
    /// Record a task duration
    pub fn record_task_duration<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed().as_secs_f64();
        self.task_duration.observe(duration);
        result
    }
    
    /// Update active connections
    pub fn update_active_connections(&self, count: i64) {
        self.active_connections.set(count);
    }
    
    /// Update memory usage
    pub fn update_memory_usage(&self, bytes: i64) {
        self.memory_usage.set(bytes);
    }
    
    /// Update CPU usage
    pub fn update_cpu_usage(&self, percent: i64) {
        self.cpu_usage.set(percent);
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new().expect("Failed to create default metrics collector")
    }
}

/// Global metrics collector instance
static GLOBAL_METRICS: std::sync::OnceLock<MetricsCollector> = std::sync::OnceLock::new();

/// Get the global metrics collector
///
/// # Panics
/// Panics if the metrics collector cannot be initialized.
pub fn global_metrics() -> &'static MetricsCollector {
    GLOBAL_METRICS.get_or_init(|| {
        MetricsCollector::new().expect("Failed to initialize global metrics collector")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new().expect("Failed to create metrics collector");
        assert!(collector.registry().gather().is_ok());
    }

    #[test]
    fn test_record_message_published() {
        let collector = MetricsCollector::new().expect("Failed to create metrics collector");
        collector.record_message_published("test.topic");
        assert_eq!(collector.messages_published.get(), 1);
    }

    #[test]
    fn test_record_message_received() {
        let collector = MetricsCollector::new().expect("Failed to create metrics collector");
        collector.record_message_received("test.topic");
        assert_eq!(collector.messages_received.get(), 1);
    }

    #[test]
    fn test_record_task_submitted() {
        let collector = MetricsCollector::new().expect("Failed to create metrics collector");
        collector.record_task_submitted("test-session");
        assert_eq!(collector.tasks_submitted.get(), 1);
    }

    #[test]
    fn test_record_task_completed() {
        let collector = MetricsCollector::new().expect("Failed to create metrics collector");
        collector.record_task_completed("test-session", 5.0);
        assert_eq!(collector.tasks_completed.get(), 1);
    }

    #[test]
    fn test_record_task_failed() {
        let collector = MetricsCollector::new().expect("Failed to create metrics collector");
        collector.record_task_failed("test-session");
        assert_eq!(collector.tasks_failed.get(), 1);
    }

    #[test]
    fn test_update_active_connections() {
        let collector = MetricsCollector::new().expect("Failed to create metrics collector");
        collector.update_active_connections(10);
        assert_eq!(collector.active_connections.get(), 10);
    }

    #[test]
    fn test_global_metrics() {
        let metrics = global_metrics();
        metrics.record_message_published("test.topic");
        assert_eq!(metrics.messages_published.get(), 1);
    }
}
