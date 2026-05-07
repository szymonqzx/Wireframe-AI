//! Performance benchmarks for Wireframe-AI plugins
//!
//! Run benchmarks with:
//! cargo bench --bench plugin_benchmarks
//!
//! For detailed flamegraphs:
//! cargo bench --bench plugin_benchmarks -- --profile-time 10

use agentic_sdk::{
    message_types::{ChatMessage, ContextPackage, TaskComplete, TaskSubmitted, TaskEnriched},
    plugin::Plugin,
    plugin_registry::PluginRegistry,
    plugins::{
        context::{EnrichmentStrategy, MemoryBackend, StorageBackend},
        interface::{InputMethod, OutputFormatter},
        orchestrator::{ExecutionStrategy, ResultSynthesizer, TaskPlanner},
        sandbox::{ResourceLimiter, SecurityPolicy, Tool},
    },
    ContextPackageBuilder, TaskSubmittedBuilder,
};
use chrono::Utc;
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};

// ============================================================================
// Mock Implementations for Benchmarking
// ============================================================================

struct BenchmarkStorageBackend;

#[async_trait::async_trait]
impl Plugin for BenchmarkStorageBackend {
    fn plugin_id(&self) -> &'static str {
        "storage-benchmark"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Benchmark storage backend"
    }

    async fn initialize(&mut self, _config: &serde_json::Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, agentic_sdk::plugin::PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl StorageBackend for BenchmarkStorageBackend {
    async fn ensure_session(&self, _session_id: &str) -> Result<(), agentic_sdk::plugins::context::StorageError> {
        Ok(())
    }

    async fn store_message(
        &self,
        _session_id: &str,
        _role: &str,
        _content: &str,
    ) -> Result<(), agentic_sdk::plugins::context::StorageError> {
        Ok(())
    }

    async fn load_session_history(
        &self,
        _session_id: &str,
        _limit: usize,
    ) -> Result<Vec<ChatMessage>, agentic_sdk::plugins::context::StorageError> {
        Ok(vec![])
    }
}

struct BenchmarkTaskPlanner;

#[async_trait::async_trait]
impl Plugin for BenchmarkTaskPlanner {
    fn plugin_id(&self) -> &'static str {
        "planner-benchmark"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Benchmark task planner"
    }

    async fn initialize(&mut self, _config: &serde_json::Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, agentic_sdk::plugin::PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl TaskPlanner for BenchmarkTaskPlanner {
    async fn decompose(
        &self,
        task: &TaskEnriched,
    ) -> Result<Vec<agentic_sdk::plugins::orchestrator::TaskDescription>, agentic_sdk::plugins::orchestrator::PlanningError> {
        let subtasks: Vec<agentic_sdk::plugins::orchestrator::TaskDescription> = task
            .user_input
            .split(" and ")
            .map(|s| agentic_sdk::plugins::orchestrator::TaskDescription {
                description: s.trim().to_string(),
                dependencies: vec![],
                metadata: json!({}),
            })
            .collect();
        Ok(subtasks)
    }
}

struct BenchmarkTool;

#[async_trait::async_trait]
impl Plugin for BenchmarkTool {
    fn plugin_id(&self) -> &'static str {
        "tool-benchmark"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Benchmark tool"
    }

    async fn initialize(&mut self, _config: &serde_json::Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, agentic_sdk::plugin::PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }
}

#[async_trait::async_trait]
impl Tool for BenchmarkTool {
    fn tool_name(&self) -> &'static str {
        "benchmark"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "input": {"type": "string"}
            },
            "required": ["input"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _sandbox_context: &agentic_sdk::plugins::sandbox::SandboxContext,
    ) -> Result<serde_json::Value, agentic_sdk::plugins::sandbox::ToolError> {
        Ok(json!({ "result": params }))
    }
}

// ============================================================================
// Plugin Registry Benchmarks
// ============================================================================

fn bench_plugin_registry_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("plugin_registry_register", |b| {
        b.to_async(&rt).iter(|| async {
            let registry = PluginRegistry::new();
            let plugin = Box::new(BenchmarkStorageBackend);
            black_box(registry.register(plugin).await.unwrap());
        });
    });

    c.bench_function("plugin_registry_get", |b| {
        b.to_async(&rt).iter(|| async {
            let registry = PluginRegistry::new();
            let plugin = Box::new(BenchmarkStorageBackend);
            registry.register(plugin).await.unwrap();
            black_box(registry.get::<BenchmarkStorageBackend>("storage-benchmark").await.unwrap());
        });
    });

    c.bench_function("plugin_registry_is_registered", |b| {
        b.to_async(&rt).iter(|| async {
            let registry = PluginRegistry::new();
            let plugin = Box::new(BenchmarkStorageBackend);
            registry.register(plugin).await.unwrap();
            black_box(registry.is_registered("storage-benchmark").await);
        });
    });

    c.bench_function("plugin_registry_list_plugins", |b| {
        b.to_async(&rt).iter(|| async {
            let registry = PluginRegistry::new();
            for i in 0..10 {
                let plugin = Box::new(BenchmarkStorageBackend);
                registry.register(plugin).await.unwrap();
            }
            black_box(registry.list_plugins().await);
        });
    });
}

// ============================================================================
// Storage Backend Benchmarks
// ============================================================================

fn bench_storage_backend(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let storage = Arc::new(BenchmarkStorageBackend);

    c.bench_function("storage_ensure_session", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(storage.ensure_session("test-session").await.unwrap());
        });
    });

    c.bench_function("storage_store_message", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(
                storage
                    .store_message("test-session", "user", "Test message content")
                    .await
                    .unwrap(),
            );
        });
    });

    c.bench_function("storage_load_session_history", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(storage.load_session_history("test-session", 100).await.unwrap());
        });
    });
}

// ============================================================================
// Task Planner Benchmarks
// ============================================================================

fn bench_task_planner(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let planner = Arc::new(BenchmarkTaskPlanner);

    let task = TaskEnriched {
        session_id: "test-session".to_string(),
        correlation_id: "test-correlation".to_string(),
        user_input: "Task 1 and Task 2 and Task 3".to_string(),
        context: ContextPackageBuilder::default().build().unwrap(),
        inferred_constraints: vec![],
        enriched_at: Utc::now().timestamp(),
    };

    c.bench_function("planner_decompose", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(planner.decompose(&task).await.unwrap());
        });
    });

    c.bench_function("planner_decompose_complex", |b| {
        let complex_task = TaskEnriched {
            session_id: "test-session".to_string(),
            correlation_id: "test-correlation".to_string(),
            user_input: "Task 1 and Task 2 and Task 3 and Task 4 and Task 5 and Task 6 and Task 7 and Task 8 and Task 9 and Task 10".to_string(),
            context: ContextPackageBuilder::default().build().unwrap(),
            inferred_constraints: vec![],
            enriched_at: Utc::now().timestamp(),
        };

        b.to_async(&rt).iter(|| async {
            black_box(planner.decompose(&complex_task).await.unwrap());
        });
    });
}

// ============================================================================
// Tool Benchmarks
// ============================================================================

fn bench_tool(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let tool = Arc::new(BenchmarkTool);

    let context = agentic_sdk::plugins::sandbox::SandboxContext {
        working_dir: "/tmp".to_string(),
        environment: vec![],
        allowed_paths: vec![],
    };

    c.bench_function("tool_execute", |b| {
        b.to_async(&rt).iter(|| async {
            let params = json!({ "input": "test" });
            black_box(tool.execute(params, &context).await.unwrap());
        });
    });

    c.bench_function("tool_execute_large_payload", |b| {
        b.to_async(&rt).iter(|| async {
            let large_params = json!({ "input": "x".repeat(10000) });
            black_box(tool.execute(large_params, &context).await.unwrap());
        });
    });

    c.bench_function("tool_input_schema", |b| {
        b.iter(|| {
            black_box(tool.input_schema());
        });
    });
}

// ============================================================================
// Plugin Lifecycle Benchmarks
// ============================================================================

fn bench_plugin_lifecycle(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("plugin_initialize", |b| {
        b.to_async(&rt).iter(|| async {
            let mut plugin = BenchmarkStorageBackend;
            let config = json!({});
            black_box(plugin.initialize(&config).await.unwrap());
        });
    });

    c.bench_function("plugin_health_check", |b| {
        b.to_async(&rt).iter(|| async {
            let plugin = BenchmarkStorageBackend;
            black_box(plugin.health_check().await.unwrap());
        });
    });

    c.bench_function("plugin_shutdown", |b| {
        b.to_async(&rt).iter(|| async {
            let mut plugin = BenchmarkStorageBackend;
            black_box(plugin.shutdown().await.unwrap());
        });
    });
}

// ============================================================================
// Concurrent Operations Benchmarks
// ============================================================================

fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_operations");

    for num_tasks in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::new("concurrent_store_message", num_tasks), num_tasks, |b, &num_tasks| {
            b.to_async(&rt).iter(|| async {
                let storage = Arc::new(BenchmarkStorageBackend);
                let mut handles = vec![];

                for i in 0..num_tasks {
                    let storage_clone = storage.clone();
                    let handle = tokio::spawn(async move {
                        storage_clone
                            .store_message(&format!("session-{}", i), "user", "Test message")
                            .await
                            .unwrap();
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.await.unwrap();
                }
            });
        });
    }

    group.finish();
}

// ============================================================================
// Memory Usage Benchmarks
// ============================================================================

fn bench_memory_usage(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    c.bench_function("memory_large_context_package", |b| {
        b.to_async(&rt).iter(|| async {
            let mut context = ContextPackageBuilder::default().build().unwrap();
            
            // Add large amounts of data
            for i in 0..1000 {
                context.memory_chunks.push(agentic_sdk::message_types::MemoryChunk {
                    id: format!("chunk-{}", i),
                    content: "x".repeat(1000),
                    source: "test".to_string(),
                    relevance_score: 0.5,
                });
            }

            black_box(context);
        });
    });

    c.bench_function("memory_large_message_history", |b| {
        b.to_async(&rt).iter(|| async {
            let mut messages = Vec::new();
            
            for i in 0..1000 {
                messages.push(ChatMessage {
                    role: "user".to_string(),
                    content: "x".repeat(1000),
                    timestamp: Utc::now().timestamp(),
                });
            }

            black_box(messages);
        });
    });
}

// ============================================================================
// Benchmark Groups
// ============================================================================

criterion_group!(
    name = plugin_registry;
    config = Criterion::default().sample_size(100);
    targets = bench_plugin_registry_operations
);

criterion_group!(
    name = storage_backend;
    config = Criterion::default().sample_size(100);
    targets = bench_storage_backend
);

criterion_group!(
    name = task_planner;
    config = Criterion::default().sample_size(100);
    targets = bench_task_planner
);

criterion_group!(
    name = tool;
    config = Criterion::default().sample_size(100);
    targets = bench_tool
);

criterion_group!(
    name = plugin_lifecycle;
    config = Criterion::default().sample_size(100);
    targets = bench_plugin_lifecycle
);

criterion_group!(
    name = concurrent_operations;
    config = Criterion::default().sample_size(50);
    targets = bench_concurrent_operations
);

criterion_group!(
    name = memory_usage;
    config = Criterion::default().sample_size(50);
    targets = bench_memory_usage
);

criterion_main!(
    plugin_registry,
    storage_backend,
    task_planner,
    tool,
    plugin_lifecycle,
    concurrent_operations,
    memory_usage
);
