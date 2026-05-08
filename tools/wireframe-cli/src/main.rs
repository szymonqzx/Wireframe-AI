//! wireframe-cli — CLI toolkit for Wireframe AI module development.
//!
//! Commands:
//!   new <name>        — Scaffold a new module crate
//!   init              — Initialize a module in the current directory
//!   list-templates    — Show available module templates
//!   test <name>       — Run integration tests for a module
//!   deploy <name>     — Deploy a module to a remote host
//!   debug             — Start the message inspector/debugger
//!   validate          — Validate schemas and module interfaces
//!   replay <file>     — Replay messages from a capture file
//!   profile <name>    — Profile module performance
//!   module list       — List installed modules
//!   module start      — Start a specific module
//!   module stop       — Stop a specific module
//!   module status     — Check module status
//!   module logs       — View module logs
//!
//! ## Example
//!
//! ```bash
//! wireframe new my-custom-module
//! cd my-custom-module
//! cargo run
//!
//! wireframe test my-custom-module
//! wireframe debug --topic task.>
//! wireframe validate --module my-custom-module
//! wireframe replay capture.jsonl
//! wireframe module list
//! wireframe module start wireframe-ai-context-core
//! ```

use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};

// ── CLI ─────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "wireframe")]
#[command(about = "Wireframe AI — module development toolkit")]
#[command(version = "0.2.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scaffold a new module in a new directory
    New {
        /// Module name (kebab-case)
        name: String,
        /// Module template type
        #[arg(short, long, value_enum, default_value = "basic")]
        template: TemplateType,
        /// Output directory (default: <name>)
        #[arg(short, long)]
        out_dir: Option<String>,
    },
    /// Initialize a module in the current directory
    Init {
        /// Module name
        #[arg(short, long)]
        name: String,
        /// Module template type
        #[arg(short, long, value_enum, default_value = "basic")]
        template: TemplateType,
    },
    /// List available templates
    ListTemplates,
    /// Test a module (cargo test + integration checks)
    Test {
        /// Module crate name or path
        module: String,
        /// Run integration tests against a live NATS bus
        #[arg(long)]
        integration: bool,
    },
    /// Deploy a module to a remote host
    Deploy {
        /// Module crate name
        module: String,
        /// Target host (SSH target or docker context)
        #[arg(short, long)]
        target: Option<String>,
        /// Docker image tag
        #[arg(short, long)]
        docker: bool,
    },
    /// Debug: inspect NATS messages in real-time
    Debug {
        /// Topic pattern to subscribe to (default: >)
        #[arg(short, long, default_value = ">")]
        topic: String,
        /// Filter by correlation_id
        #[arg(short, long)]
        correlation: Option<String>,
        /// Output format: pretty, json, compact
        #[arg(short, long, default_value = "pretty")]
        format: DebugFormat,
        /// Capture messages to file for later replay
        #[arg(short, long)]
        capture: Option<String>,
    },
    /// Validate module interfaces and schemas
    Validate {
        /// Module path to validate (default: current directory)
        #[arg(short, long)]
        module: Option<String>,
        /// Validate against NATS topic registry
        #[arg(long)]
        registry: bool,
        /// Validate JSON schemas
        #[arg(long)]
        schemas: bool,
    },
    /// Replay captured messages from a file
    Replay {
        /// Capture file (.jsonl)
        file: String,
        /// NATS URL to replay to
        #[arg(short, long, default_value = "nats://localhost:4222")]
        nats_url: String,
        /// Replay speed multiplier (1.0 = real-time)
        #[arg(short, long, default_value = "1.0")]
        speed: f64,
        /// Filter by topic pattern
        #[arg(short, long)]
        topic: Option<String>,
    },
    /// Profile module performance
    Profile {
        /// Module crate name
        module: String,
        /// Duration in seconds
        #[arg(short, long, default_value = "30")]
        duration: u64,
        /// Output file for profiling report
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Hot-reload development mode for a module
    Dev {
        /// Module crate name
        module: String,
        /// NATS URL
        #[arg(short, long, default_value = "nats://localhost:4222")]
        nats_url: String,
    },
    /// Module management commands
    Module {
        #[command(subcommand)]
        module_cmd: ModuleCommands,
    },
}

#[derive(Subcommand)]
enum ModuleCommands {
    /// List all installed modules
    List {
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },
    /// Start a specific module
    Start {
        /// Module name
        name: String,
        /// Build mode (debug/release)
        #[arg(short, long, default_value = "debug")]
        build_mode: String,
        /// NATS URL
        #[arg(short, long, default_value = "nats://localhost:4222")]
        nats_url: String,
    },
    /// Stop a specific module
    Stop {
        /// Module name
        name: String,
        /// Force stop (SIGKILL)
        #[arg(long)]
        force: bool,
    },
    /// Check module status
    Status {
        /// Module name (optional, shows all if not specified)
        name: Option<String>,
    },
    /// View module logs
    Logs {
        /// Module name
        name: String,
        /// Number of lines to show
        #[arg(short, long, default_value = "100")]
        lines: usize,
        /// Follow log output
        #[arg(long)]
        follow: bool,
    },
}

#[derive(Clone, Debug, Default, clap::ValueEnum)]
enum TemplateType {
    /// Basic module with #[module] macro
    #[default]
    Basic,
    /// Adapter module (subscribes to agent.job)
    Adapter,
    /// Context module (subscribes to task.submitted)
    Context,
    /// Orchestrator module (subscribes to task.enriched)
    Orchestrator,
    /// Listener module (logs all messages)
    Listener,
    /// Service module (handles requests/responses)
    Service,
    /// Webhook receiver module
    Webhook,
    /// Integration module (connects to external APIs)
    Integration,
    /// Cache module with TTL
    Cache,
    /// Rate limiter module
    RateLimiter,
}

#[derive(Clone, Debug, Default, clap::ValueEnum)]
enum DebugFormat {
    #[default]
    Pretty,
    Json,
    Compact,
}

// ── Scaffold logic ──────────────────────────────────────────────────────────

fn scaffold_module(name: &str, template: &TemplateType, out_dir: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(out_dir)?;

    // Cargo.toml
    let cargo_toml = generate_cargo_toml(name, template);
    fs::write(out_dir.join("Cargo.toml"), cargo_toml)?;

    // src/main.rs
    let main_rs = generate_main_rs(name, template);
    fs::create_dir_all(out_dir.join("src"))?;
    fs::write(out_dir.join("src/main.rs"), main_rs)?;

    // .gitignore
    fs::write(out_dir.join(".gitignore"), generate_gitignore())?;

    // README.md
    fs::write(out_dir.join("README.md"), generate_readme(name, template))?;

    println!("Created module '{}' at {}", name, out_dir.display());
    println!("  cd {} && cargo run", out_dir.display());
    Ok(())
}

fn generate_cargo_toml(name: &str, template: &TemplateType) -> String {
    let sanitized = name.replace("-", "_");
    let mut deps = r#"[dependencies]
agentic-sdk = { workspace = true, features = ["macros"] }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
chrono = { workspace = true }"#
        .to_string();

    if matches!(template, TemplateType::Integration) {
        deps.push_str(
            "
reqwest = { version = \"0.12\", features = [\"json\"] }",
        );
    }

    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"
description = "Wireframe AI module — {}"

[[bin]]
name = "{}"
path = "src/main.rs"

{}
"#,
        sanitized, name, sanitized, deps
    )
}

fn generate_main_rs(name: &str, template: &TemplateType) -> String {
    let module_struct = to_pascal_case(name);

    match template {
        TemplateType::Basic => generate_basic_template(&module_struct),
        TemplateType::Adapter => generate_adapter_template(&module_struct),
        TemplateType::Context => generate_context_template(&module_struct),
        TemplateType::Orchestrator => generate_orchestrator_template(&module_struct),
        TemplateType::Listener => generate_listener_template(&module_struct),
        TemplateType::Service => generate_service_template(&module_struct),
        TemplateType::Webhook => generate_webhook_template(&module_struct),
        TemplateType::Integration => generate_integration_template(&module_struct),
        TemplateType::Cache => generate_cache_template(&module_struct),
        TemplateType::RateLimiter => generate_rate_limiter_template(&module_struct),
    }
}

fn generate_basic_template(struct_name: &str) -> String {
    format!(
        r#"//! {} — Wireframe AI module
//!
//! Subscribes to: example.topic
//! Publishes to: example.response

use agentic_sdk::{{Envelope, Module}};

struct {};

#[agentic_sdk::module(
    subscribes = ["example.topic"],
    publishes  = ["example.response"],
    queue_group = "example_handler"
)]
impl Module for {} {{
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {{
        tracing::info!(topic = %env.topic, "received message");

        let response = serde_json::json!({{
            "echo": env.payload,
            "processed_at": chrono::Utc::now().timestamp(),
        }});

        vec![env.reply("example.response", response)]
    }}
}}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    tracing_subscriber::fmt::init();
    {}.run("nats://localhost:4222").await
}}
"#,
        struct_name, struct_name, struct_name, struct_name
    )
}

fn generate_adapter_template(struct_name: &str) -> String {
    format!(
        r#"//! {} — Wireframe AI Reasoning Adapter
//!
//! Subscribes to: agent.job
//! Publishes to: agent.result

use agentic_sdk::{{builders::AgentResultBuilder, Envelope, Module}};

struct {};

#[agentic_sdk::module(
    subscribes = ["agent.job"],
    publishes  = ["agent.result"],
    queue_group = "agent_job_handler"
)]
impl Module for {} {{
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {{
        let job: agentic_sdk::message_types::AgentJob =
            match serde_json::from_value(env.payload.clone()) {{
                Ok(j) => j,
                Err(e) => {{
                    tracing::error!(error = %e, "failed to parse agent.job");
                    return vec![];
                }}
            }};

        tracing::info!(job_id = %job.job_id, "processing agent job");

        // TODO: Implement your adapter logic here
        let result_text = format!("Processed: {{}}", job.task.user_input);

        let result = AgentResultBuilder::new()
            .job_id(&job.job_id)
            .correlation_parent(&job.correlation_parent)
            .output_text(result_text)
            .build_envelope();

        match result {{
            Ok(envelope) => vec![envelope],
            Err(e) => {{
                tracing::error!(error = %e, "failed to build result");
                vec![]
            }}
        }}
    }}
}}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    tracing_subscriber::fmt::init();
    {}.run("nats://localhost:4222").await
}}
"#,
        struct_name, struct_name, struct_name, struct_name
    )
}

fn generate_context_template(struct_name: &str) -> String {
    format!(
        r#"//! {} — Wireframe AI Context Module
//!
//! Subscribes to: task.submitted
//! Publishes to: task.enriched

use agentic_sdk::{{builders::TaskSubmittedBuilder, Envelope, Module}};

struct {};

#[agentic_sdk::module(
    subscribes = ["task.submitted"],
    publishes  = ["task.enriched"],
    queue_group = "task_handler"
)]
impl Module for {} {{
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {{
        let submitted: agentic_sdk::message_types::TaskSubmitted =
            match serde_json::from_value(env.payload.clone()) {{
                Ok(t) => t,
                Err(e) => {{
                    tracing::error!(error = %e, "failed to parse task.submitted");
                    return vec![];
                }}
            }};

        tracing::info!(session = %submitted.session_id, "enriching task");

        // TODO: Implement your context enrichment logic here
        let context = agentic_sdk::builders::ContextPackageBuilder::new()
            .session_message("user", &submitted.user_input)
            .build();

        let enriched = agentic_sdk::message_types::TaskEnriched {{
            session_id: submitted.session_id.clone(),
            correlation_id: env.correlation_id.clone(),
            user_input: submitted.user_input,
            context,
            inferred_constraints: vec![],
            enriched_at: chrono::Utc::now().timestamp(),
        }};

        vec![env.reply("task.enriched", enriched)]
    }}
}}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    tracing_subscriber::fmt::init();
    {}.run("nats://localhost:4222").await
}}
"#,
        struct_name, struct_name, struct_name, struct_name
    )
}

fn generate_orchestrator_template(struct_name: &str) -> String {
    format!(
        r#"//! {} — Wireframe AI Orchestrator Module
//!
//! Subscribes to: task.enriched
//! Publishes to: agent.job, task.complete

use agentic_sdk::{{builders::AgentJobBuilder, Envelope, Module}};

struct {} {{
    concurrency: u32,
}}

#[agentic_sdk::module(
    subscribes = ["task.enriched"],
    publishes  = ["agent.job", "task.complete"],
    queue_group = "task_handler"
)]
impl Module for {} {{
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {{
        let enriched: agentic_sdk::message_types::TaskEnriched =
            match serde_json::from_value(env.payload.clone()) {{
                Ok(t) => t,
                Err(e) => {{
                    tracing::error!(error = %e, "failed to parse task.enriched");
                    return vec![];
                }}
            }};

        tracing::info!(session = %enriched.session_id, "orchestrating task");

        // TODO: Implement your orchestration logic here
        // Fan out to N agent jobs, collect results, synthesize task.complete

        let job = AgentJobBuilder::new()
            .correlation_parent(&env.correlation_id)
            .user_input(&enriched.user_input)
            .build_envelope();

        match job {{
            Ok(envelope) => vec![envelope],
            Err(e) => {{
                tracing::error!(error = %e, "failed to build agent.job");
                vec![]
            }}
        }}
    }}
}}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    tracing_subscriber::fmt::init();
    {} {{ concurrency: 3 }}.run("nats://localhost:4222").await
}}
"#,
        struct_name, struct_name, struct_name, struct_name
    )
}

fn generate_listener_template(struct_name: &str) -> String {
    format!(
        r#"//! {} — Wireframe AI Listener Module
//!
//! Subscribes to: > (all topics)
//! Publishes to: (none)
//!
//! This module logs all messages flowing through the system.

use agentic_sdk::{{Envelope, Module}};

struct {};

#[agentic_sdk::module(
    subscribes = ["task.>", "agent.>", "sys.>"],
    publishes  = [],
    queue_group = "listener"
)]
impl Module for {} {{
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {{
        tracing::info!(
            topic = %env.topic,
            correlation = %env.correlation_id,
            session = %env.session_id,
            payload = %serde_json::to_string_pretty(&env.payload).unwrap_or_default(),
            "message received"
        );
        vec![]
    }}
}}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    tracing_subscriber::fmt::init();
    {}.run("nats://localhost:4222").await
}}
"#,
        struct_name, struct_name, struct_name, struct_name
    )
}

fn generate_service_template(struct_name: &str) -> String {
    format!(
        r#"//! {} — Wireframe AI Service Module
//!
//! Subscribes to: service.request
//! Publishes to: service.response
//!
//! Handles request/response pattern with structured inputs and outputs.

use agentic_sdk::{{Envelope, Module}};

struct {};

#[agentic_sdk::module(
    subscribes = ["service.request"],
    publishes  = ["service.response"],
    queue_group = "service_handler"
)]
impl Module for {} {{
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {{
        let request = &env.payload;
        let action = request
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        tracing::info!(action, "handling service request");

        // TODO: Implement your service logic here
        let response = serde_json::json!({{
            "action": action,
            "status": "ok",
            "result": {{}},
            "handled_at": chrono::Utc::now().timestamp(),
        }});

        vec![env.reply("service.response", response)]
    }}
}}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    tracing_subscriber::fmt::init();
    {}.run("nats://localhost:4222").await
}}
"#,
        struct_name, struct_name, struct_name, struct_name
    )
}

fn generate_webhook_template(struct_name: &str) -> String {
    format!(
        r#"//! {} — Wireframe AI Webhook Receiver
//!
//! Subscribes to: webhook.receive
//! Publishes to: webhook.processed
//!
//! Receives external webhook payloads and normalizes them.

use agentic_sdk::{{Envelope, Module}};

struct {};

#[agentic_sdk::module(
    subscribes = ["webhook.receive"],
    publishes  = ["webhook.processed"],
    queue_group = "webhook_handler"
)]
impl Module for {} {{
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {{
        let source = env.payload.get("source").and_then(|v| v.as_str()).unwrap_or("unknown");
        tracing::info!(source, "received webhook");

        let normalized = serde_json::json!({{
            "original": env.payload,
            "received_at": chrono::Utc::now().timestamp(),
            "source": source,
        }});

        vec![env.reply("webhook.processed", normalized)]
    }}
}}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    tracing_subscriber::fmt::init();
    {}.run("nats://localhost:4222").await
}}
"#,
        struct_name, struct_name, struct_name, struct_name
    )
}

fn generate_integration_template(struct_name: &str) -> String {
    format!(
        r#"//! {} — Wireframe AI Integration Module
//!
//! Subscribes to: integration.request
//! Publishes to: integration.response
//!
//! Bridges Wireframe-AI with external APIs and services.

use agentic_sdk::{{Envelope, Module}};

struct {} {{
    client: reqwest::Client,
}}

impl Default for {} {{
    fn default() -> Self {{
        Self {{
            client: reqwest::Client::new(),
        }}
    }}
}}

#[agentic_sdk::module(
    subscribes = ["integration.request"],
    publishes  = ["integration.response"],
    queue_group = "integration_handler"
)]
impl Module for {} {{
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {{
        let service = env.payload.get("service").and_then(|v| v.as_str()).unwrap_or("unknown");
        let url = env.payload.get("url").and_then(|v| v.as_str()).unwrap_or("https://jsonplaceholder.typicode.com/todos/1");
        tracing::info!(service, url, "handling integration request");

        let response = match self.client.get(url).send().await {{
            Ok(res) => {{
                let status = res.status().as_u16();
                let body = res.json::<serde_json::Value>().await.unwrap_or(serde_json::json!({{}}));
                serde_json::json!({{
                    "service": service,
                    "status": "success",
                    "http_status": status,
                    "data": body,
                    "handled_at": chrono::Utc::now().timestamp(),
                }})
            }},
            Err(err) => {{
                serde_json::json!({{
                    "service": service,
                    "status": "error",
                    "error": err.to_string(),
                    "handled_at": chrono::Utc::now().timestamp(),
                }})
            }}
        }};

        vec![env.reply("integration.response", response)]
    }}
}}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    tracing_subscriber::fmt::init();
    {}::default().run("nats://localhost:4222").await
}}
"#,
        struct_name, struct_name, struct_name, struct_name, struct_name
    )
}

fn generate_cache_template(struct_name: &str) -> String {
    format!(
        r#"//! {} — Wireframe AI Cache Module
//!
//! Subscribes to: cache.get, cache.set, cache.invalidate
//! Publishes to: cache.value, cache.ack
//!
//! Provides TTL-based caching for agent workflows.

use agentic_sdk::{{Envelope, Module}};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

struct CacheEntry {{
    value: serde_json::Value,
    expires_at: i64,
}}

struct {} {{
    store: Arc<RwLock<HashMap<String, CacheEntry>>>,
    default_ttl_seconds: u64,
}}

#[agentic_sdk::module(
    subscribes = ["cache.get", "cache.set", "cache.invalidate"],
    publishes  = ["cache.value", "cache.ack"],
    queue_group = "cache_handler"
)]
impl Module for {} {{
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {{
        match env.topic.as_str() {{
            "cache.get" => {{
                let key = env.payload.get("key").and_then(|v| v.as_str()).unwrap_or("");
                let store = self.store.read().await;
                let value = store.get(key).and_then(|entry| {{
                    if entry.expires_at > chrono::Utc::now().timestamp() {{
                        Some(entry.value.clone())
                    }} else {{
                        None
                    }}
                }});
                let response = serde_json::json!({{
                    "key": key,
                    "value": value,
                    "found": value.is_some(),
                }});
                vec![env.reply("cache.value", response)]
            }}
            "cache.set" => {{
                let key = env.payload.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let ttl = env.payload.get("ttl").and_then(|v| v.as_u64()).unwrap_or(self.default_ttl_seconds);
                let value = env.payload.get("value").cloned().unwrap_or(serde_json::Value::Null);
                let mut store = self.store.write().await;
                store.insert(key, CacheEntry {{
                    value,
                    expires_at: chrono::Utc::now().timestamp() + ttl as i64,
                }});
                vec![env.reply("cache.ack", serde_json::json!({{ "status": "set" }}))]
            }}
            "cache.invalidate" => {{
                let key = env.payload.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let mut store = self.store.write().await;
                store.remove(&key);
                vec![env.reply("cache.ack", serde_json::json!({{ "status": "invalidated" }}))]
            }}
            _ => vec![],
        }}
    }}
}}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    tracing_subscriber::fmt::init();
    {} {{
        store: Arc::new(RwLock::new(HashMap::new())),
        default_ttl_seconds: 300,
    }}.run("nats://localhost:4222").await
}}
"#,
        struct_name, struct_name, struct_name, struct_name
    )
}

fn generate_rate_limiter_template(struct_name: &str) -> String {
    format!(
        r#"//! {} — Wireframe AI Rate Limiter
//!
//! Subscribes to: rate.check, rate.configure
//! Publishes to: rate.status, rate.ack
//!
//! Token-bucket rate limiting for agent workflows.

use agentic_sdk::{{Envelope, Module}};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

struct Bucket {{
    tokens: f64,
    last_update: i64,
    rate: f64,
    capacity: f64,
}}

struct {} {{
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
}}

#[agentic_sdk::module(
    subscribes = ["rate.check", "rate.configure"],
    publishes  = ["rate.status", "rate.ack"],
    queue_group = "rate_handler"
)]
impl Module for {} {{
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {{
        match env.topic.as_str() {{
            "rate.check" => {{
                let key = env.payload.get("key").and_then(|v| v.as_str()).unwrap_or("default").to_string();
                let mut buckets = self.buckets.lock().await;
                let bucket = buckets.entry(key.clone()).or_insert_with(|| Bucket {{
                    tokens: 10.0,
                    last_update: chrono::Utc::now().timestamp(),
                    rate: 1.0,
                    capacity: 10.0,
                }});

                let now = chrono::Utc::now().timestamp() as f64;
                let elapsed = now - bucket.last_update as f64;
                bucket.tokens = (bucket.tokens + elapsed * bucket.rate).min(bucket.capacity);
                bucket.last_update = now as i64;

                let allowed = bucket.tokens >= 1.0;
                if allowed {{
                    bucket.tokens -= 1.0;
                }}

                let reset_at = if bucket.rate > 0.0 {{
                    bucket.last_update + (1.0 / bucket.rate) as i64
                }} else {{
                    bucket.last_update + 3600
                }};
                let response = serde_json::json!({{
                    "key": key,
                    "allowed": allowed,
                    "remaining": bucket.tokens,
                    "reset_at": reset_at,
                }});
                vec![env.reply("rate.status", response)]
            }}
            "rate.configure" => {{
                let key = env.payload.get("key").and_then(|v| v.as_str()).unwrap_or("default").to_string();
                let rate = env.payload.get("rate").and_then(|v| v.as_f64()).unwrap_or(1.0);
                let capacity = env.payload.get("capacity").and_then(|v| v.as_f64()).unwrap_or(10.0);
                let mut buckets = self.buckets.lock().await;
                buckets.insert(key, Bucket {{
                    tokens: capacity,
                    last_update: chrono::Utc::now().timestamp(),
                    rate,
                    capacity,
                }});
                vec![env.reply("rate.ack", serde_json::json!({{ "status": "configured" }}))]
            }}
            _ => vec![],
        }}
    }}
}}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {{
    tracing_subscriber::fmt::init();
    {} {{
        buckets: Arc::new(Mutex::new(HashMap::new())),
    }}.run("nats://localhost:4222").await
}}
"#,
        struct_name, struct_name, struct_name, struct_name
    )
}

fn generate_gitignore() -> &'static str {
    "/target\nCargo.lock\n*.log\n"
}

fn generate_readme(name: &str, template: &TemplateType) -> String {
    format!(
        r#"# {}

Wireframe AI module generated from `{}` template.

## Running

```bash
cargo run
```

## Configuration

Set `NATS_URL` environment variable to override the default NATS server:

```bash
NATS_URL=nats://localhost:4222 cargo run
```

## Workspace Setup

This module uses workspace dependencies. Add it to the `members` array in the
workspace `Cargo.toml` before building:

```toml
members = [
    # ... existing members ...
    "{}",
]
```

## Template: {:?}

This module was scaffolded with `wireframe-cli`.
"#,
        name, name, name, template
    )
}

// ── Command handlers ──────────────────────────────────────────────────────────

fn handle_test(module: &str, integration: bool) -> anyhow::Result<()> {
    println!("Testing module: {}", module);

    let mut cmd = Command::new("cargo");
    cmd.args(["test", "-p", module]);

    if integration {
        cmd.arg("--features").arg("integration-tests");
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Tests failed for module: {}", module);
    }

    println!("Tests passed for {}", module);
    Ok(())
}

/// Validate a module/crate name to prevent path traversal and injection.
/// Only alphanumeric ASCII, hyphens, and underscores are allowed.
fn validate_module_name(name: &str) -> anyhow::Result<()> {
    if name.is_empty() {
        anyhow::bail!("module name cannot be empty");
    }
    if name.len() > 64 {
        anyhow::bail!("module name exceeds 64 characters");
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        anyhow::bail!(
            "module name '{}' contains invalid characters; only alphanumeric, hyphens, and underscores are allowed",
            name
        );
    }
    Ok(())
}

fn handle_deploy(module: &str, target: &Option<String>, docker: bool) -> anyhow::Result<()> {
    validate_module_name(module)?;
    println!("Deploying module: {}", module);

    // Step 1: Build the module in release mode.
    println!("Building {} in release mode...", module);
    let build_status = Command::new("cargo")
        .args(["build", "--release", "-p", module])
        .status()?;
    if !build_status.success() {
        anyhow::bail!("Build failed for module: {}", module);
    }

    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let binary_candidates = [
        format!("{}/release/{}", target_dir, module),
        format!("{}/release/{}", target_dir, module.replace("-", "_")),
    ];
    let binary = binary_candidates
        .iter()
        .find(|p| Path::new(p).exists())
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Binary not found after build. Tried: {}. \
                 Run from workspace root or set CARGO_TARGET_DIR.",
                binary_candidates.join(", ")
            )
        })?;

    // Validate host early: reject values that look like CLI option flags or
    // contain shell metacharacters to prevent injection in ssh / scp commands.
    if let Some(host) = target {
        if host.is_empty() {
            anyhow::bail!("Empty target host is not allowed");
        }
        if host.starts_with('-') {
            anyhow::bail!(
                "Invalid target host '{}': host names starting with '-' are not allowed",
                host
            );
        }
        // Reject characters commonly used for shell/command injection.
        let forbidden: &[char] = &[
            ';', '|', '&', '$', '`', '<', '>', '*', '?', '{', '}', '(', ')',
        ];
        if let Some(c) = host.chars().find(|c| forbidden.contains(c)) {
            anyhow::bail!(
                "Invalid target host '{}': contains forbidden character '{}'",
                host,
                c
            );
        }
    }

    if docker {
        println!("Building Docker image for {}...", module);
        let dockerfile = "deploy/docker/Dockerfile.module";
        if !Path::new(dockerfile).exists() {
            anyhow::bail!("Generic Dockerfile not found at {}", dockerfile);
        }
        let image_tag = format!("{}:latest", module);
        let mut cmd = Command::new("docker");
        cmd.args([
            "build",
            "-f",
            dockerfile,
            "--build-arg",
            &format!("CRATE_NAME={}", module),
            "--build-arg",
            &format!("BINARY_NAME={}", module),
            "-t",
            &image_tag,
            ".",
        ]);
        println!("Running: {:?}", cmd);
        let status = cmd.status()?;
        if !status.success() {
            anyhow::bail!("Docker build failed for module: {}", module);
        }
        println!("Docker image built: {}", image_tag);

        if let Some(host) = target {
            println!("Pushing Docker image to {}...", host);
            let mut docker_save = Command::new("docker");
            docker_save.args(["save", &image_tag]);
            docker_save.stdout(std::process::Stdio::piped());
            let mut save_child = docker_save
                .spawn()
                .map_err(|e| anyhow::anyhow!("docker save failed: {}", e))?;

            // Pass host after `--` so ssh treats it as a positional argument
            // even if the host somehow contains leading dashes (already rejected above).
            let mut ssh = Command::new("ssh");
            ssh.arg("--");
            ssh.arg(host);
            ssh.arg("docker");
            ssh.arg("load");
            ssh.stdin(save_child.stdout.take().unwrap());
            println!("Running: docker save ... | ssh -- {} docker load", host);
            let ssh_status = ssh
                .status()
                .map_err(|e| anyhow::anyhow!("ssh docker load failed: {}", e))?;
            let save_status = save_child
                .wait()
                .map_err(|e| anyhow::anyhow!("docker save process failed: {}", e))?;
            if !save_status.success() {
                anyhow::bail!("docker save failed for image {}", image_tag);
            }
            if !ssh_status.success() {
                anyhow::bail!("Docker image push failed for module: {}", module);
            }
        }
        return Ok(());
    }

    if let Some(host) = target {
        println!("Deploying binary to host: {}", host);
        let remote_path = format!("/opt/wireframe/modules/{}-new", module);
        let mut scp = Command::new("scp");
        scp.arg("--");
        scp.arg(&binary);
        scp.arg(format!("{}:{}", host, remote_path));
        println!("Running: {:?}", scp);
        let status = scp.status()?;
        if !status.success() {
            anyhow::bail!("scp failed for module: {}", module);
        }

        let dest_path = format!("/opt/wireframe/modules/{}", module);
        let remote_cmd = format!(
            "mv {} {} && systemctl restart {}",
            shell_escape(&remote_path),
            shell_escape(&dest_path),
            shell_escape(&format!("wireframe-{}", module))
        );
        let mut ssh = Command::new("ssh");
        ssh.arg("--");
        ssh.arg(host);
        ssh.arg(&remote_cmd);
        println!("Running: {:?}", ssh);
        let status = ssh.status()?;
        if !status.success() {
            anyhow::bail!("Remote restart failed for module: {}", module);
        }
        println!("Deployed {} to {}", module, host);
    } else {
        let local_dir = Path::new("modules").join(module);
        std::fs::create_dir_all(&local_dir)?;
        let dest = local_dir.join(module);
        std::fs::copy(&binary, &dest)?;
        println!("Copied binary to {}", dest.display());
    }

    Ok(())
}

fn handle_debug(
    topic: &str,
    correlation: &Option<String>,
    format: &DebugFormat,
    capture: &Option<String>,
) -> anyhow::Result<()> {
    println!("Starting debug inspector...");

    let mut cmd = find_subcommand("wireframe-debug").unwrap_or_else(|| {
        let mut c = Command::new("cargo");
        c.args(["run", "--release", "-p", "wireframe-debug", "--"]);
        c
    });

    cmd.arg("--topic").arg(topic);
    if let Some(corr) = correlation {
        cmd.arg("--correlation").arg(corr);
    }
    let format_str = match format {
        DebugFormat::Pretty => "pretty",
        DebugFormat::Json => "json",
        DebugFormat::Compact => "compact",
    };
    cmd.arg("--format").arg(format_str);
    if let Some(cap) = capture {
        cmd.arg("--capture").arg(cap);
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("wireframe-debug exited with non-zero status");
    }
    Ok(())
}

fn handle_validate(module: &Option<String>, registry: bool, schemas: bool) -> anyhow::Result<()> {
    let module_path = module.as_deref().unwrap_or(".");
    println!("Validating module at: {}", module_path);

    let mut has_errors = false;

    // Interface validation: check Cargo.toml for agentic-sdk dependency
    let cargo_toml = Path::new(module_path).join("Cargo.toml");
    if cargo_toml.exists() {
        let content = fs::read_to_string(&cargo_toml)?;
        if content.contains("agentic-sdk") {
            println!("  OK: agentic-sdk dependency found");
        } else {
            println!("  WARN: agentic-sdk dependency not found");
            has_errors = true;
        }

        // Check if schema-validation feature is enabled
        let has_schema_feature = content.contains("schema-validation");
        if schemas && has_schema_feature {
            println!("  OK: schema-validation feature enabled in Cargo.toml");
        }
    } else {
        println!("  ERROR: Cargo.toml not found at {}", cargo_toml.display());
        has_errors = true;
    }

    // Check src/main.rs for Module trait implementation and topic registry
    let main_rs = Path::new(module_path).join("src/main.rs");
    if main_rs.exists() {
        let content = fs::read_to_string(&main_rs)?;
        if content.contains("impl Module for") {
            println!("  OK: Module trait implementation found");
        } else {
            println!("  WARN: No Module trait implementation found");
            has_errors = true;
        }

        if registry {
            validate_topic_registry(&content, &mut has_errors);
        }
    } else {
        println!("  ERROR: src/main.rs not found");
        has_errors = true;
    }

    if schemas {
        validate_schemas_directory(module_path, &mut has_errors);
    }

    if has_errors {
        anyhow::bail!("Validation failed with errors.");
    }

    println!("Validation complete.");
    Ok(())
}

/// Extract topics declared in `#[agentic_sdk::module(subscribes = [...], publishes = [...])]`
/// and validate naming conventions using `syn` for robust parsing.
fn validate_topic_registry(source: &str, has_errors: &mut bool) {
    fn parse_lit_str(input: syn::parse::ParseStream<'_>) -> syn::Result<syn::LitStr> {
        input.parse()
    }

    let ast = match syn::parse_file(source) {
        Ok(f) => f,
        Err(_) => {
            println!("  WARN: Could not parse source as Rust; skipping topic registry validation");
            return;
        }
    };

    let mut all_topics: Vec<(String, &'static str)> = Vec::new();

    for item in ast.items {
        let attrs = match item {
            syn::Item::Impl(i) => i.attrs,
            syn::Item::Struct(s) => s.attrs,
            syn::Item::Fn(f) => f.attrs,
            _ => continue,
        };

        for attr in attrs {
            // Match #[agentic_sdk::module(...)]
            let path = attr.path();
            if path.segments.len() != 2 {
                continue;
            }
            if path.segments[0].ident != "agentic_sdk" || path.segments[1].ident != "module" {
                continue;
            }

            let syn::Meta::List(list) = attr.meta else {
                continue;
            };
            let _ = list.parse_nested_meta(|meta| {
                if meta.path.is_ident("subscribes") {
                    let content;
                    syn::bracketed!(content in meta.input);
                    let parsed: syn::punctuated::Punctuated<syn::LitStr, syn::Token![,]> =
                        content.parse_terminated(parse_lit_str, syn::Token![,])?;
                    for lit in parsed {
                        all_topics.push((lit.value(), "subscribes"));
                    }
                } else if meta.path.is_ident("publishes") {
                    let content;
                    syn::bracketed!(content in meta.input);
                    let parsed: syn::punctuated::Punctuated<syn::LitStr, syn::Token![,]> =
                        content.parse_terminated(parse_lit_str, syn::Token![,])?;
                    for lit in parsed {
                        all_topics.push((lit.value(), "publishes"));
                    }
                } else {
                    // Skip unknown keys by parsing the rest as an expression.
                    let _: syn::Expr = meta.input.parse()?;
                }
                Ok(())
            });
        }
    }

    if all_topics.is_empty() {
        println!("  WARN: No topics found in #[agentic_sdk::module] attribute");
    }

    let mut seen_subscribes: Vec<String> = Vec::new();
    for (topic, kind) in &all_topics {
        // Validate naming convention
        if topic.is_empty() {
            println!("  ERROR: [{}] Empty topic string found", kind);
            *has_errors = true;
            continue;
        }
        if topic.contains(' ') {
            println!("  ERROR: [{}] Topic '{}' contains spaces", kind, topic);
            *has_errors = true;
        }
        let valid_chars: String = topic
            .chars()
            .filter(|&c| {
                !c.is_ascii_lowercase() && c != '.' && c != '*' && c != '>' && !c.is_ascii_digit()
            })
            .collect();
        if !valid_chars.is_empty() {
            println!(
                "  ERROR: [{}] Topic '{}' contains invalid characters: {}",
                kind, topic, valid_chars
            );
            *has_errors = true;
        }

        // Reserved namespace check
        if topic.starts_with("sys.")
            && !topic.starts_with("sys.module.online")
            && !topic.starts_with("sys.module.offline")
        {
            println!(
                "  WARN: [{}] Topic '{}' uses reserved 'sys.' namespace",
                kind, topic
            );
        }

        // Duplicate subscription check
        if *kind == "subscribes" {
            if seen_subscribes.contains(topic) {
                println!("  WARN: Duplicate subscription topic '{}'", topic);
            }
            seen_subscribes.push(topic.clone());
        }
    }

    if !all_topics.is_empty() {
        println!(
            "  OK: {} topics registered ({} subscribes, {} publishes)",
            all_topics.len(),
            all_topics
                .iter()
                .filter(|(_, k)| *k == "subscribes")
                .count(),
            all_topics.iter().filter(|(_, k)| *k == "publishes").count(),
        );
    }
}

/// Validate the workspace schemas/ directory.
fn validate_schemas_directory(module_path: &str, has_errors: &mut bool) {
    // Look for schemas/ directory: prefer adjacent to module, then workspace root.
    let candidates = [
        Path::new(module_path).join("schemas"),
        Path::new(module_path).join("../schemas"),
        Path::new(module_path).join("../../schemas"),
        Path::new("schemas").to_path_buf(),
    ];

    let schemas_dir = candidates.iter().find(|p| p.is_dir());
    match schemas_dir {
        Some(dir) => {
            let mut schema_files = Vec::new();
            collect_json_files(dir, &mut schema_files);

            if schema_files.is_empty() {
                println!("  WARN: No .json schema files found in {}", dir.display());
            } else {
                let mut invalid = 0usize;
                for path in &schema_files {
                    match fs::read_to_string(path) {
                        Ok(content) => {
                            if serde_json::from_str::<serde_json::Value>(&content).is_err() {
                                println!(
                                    "  ERROR: Schema file {} is not valid JSON",
                                    path.display()
                                );
                                invalid += 1;
                            }
                        }
                        Err(e) => {
                            println!("  ERROR: Cannot read schema file {}: {}", path.display(), e);
                            invalid += 1;
                        }
                    }
                }
                if invalid == 0 {
                    println!(
                        "  OK: {} schema files valid in {}",
                        schema_files.len(),
                        dir.display()
                    );
                } else {
                    println!(
                        "  ERROR: {} of {} schema files are invalid",
                        invalid,
                        schema_files.len()
                    );
                    *has_errors = true;
                }
            }
        }
        None => {
            println!(
                "  WARN: schemas/ directory not found (tried: {:?})",
                candidates
            );
        }
    }
}

/// Recursively collect all .json files under a directory.
fn collect_json_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                collect_json_files(&path, out);
            } else if path.extension().map(|ext| ext == "json").unwrap_or(false) {
                out.push(path);
            }
        }
    }
}

fn handle_replay(
    file: &str,
    nats_url: &str,
    speed: f64,
    topic: &Option<String>,
) -> anyhow::Result<()> {
    if !Path::new(file).exists() {
        anyhow::bail!("Capture file not found: {}", file);
    }

    let mut cmd = find_subcommand("wireframe-replay").unwrap_or_else(|| {
        let mut c = Command::new("cargo");
        c.args(["run", "--release", "-p", "wireframe-replay", "--"]);
        c
    });

    cmd.arg(file)
        .arg("--nats-url")
        .arg(nats_url)
        .arg("--speed")
        .arg(speed.to_string());
    if let Some(t) = topic {
        cmd.arg("--topic").arg(t);
    }

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("wireframe-replay exited with non-zero status");
    }
    Ok(())
}

fn handle_profile(module: &str, duration: u64, output: &Option<String>) -> anyhow::Result<()> {
    validate_module_name(module)?;
    let report_file = output
        .clone()
        .unwrap_or_else(|| format!("{}-profile.json", module));
    println!("Profiling module: {}", module);
    println!("  Duration: {} seconds", duration);
    println!("  Report: {}", report_file);

    // Step 1: build in release mode
    println!("\nBuilding {} in release mode...", module);
    let build_status = Command::new("cargo")
        .args(["build", "--release", "-p", module])
        .status()?;
    if !build_status.success() {
        anyhow::bail!("Build failed for module: {}", module);
    }

    // Cargo binary names may keep dashes or use underscores depending on the
    // crate's [[bin]] section. Try the most common patterns.
    let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".to_string());
    let candidates = [
        format!("{}/release/{}", target_dir, module),
        format!("{}/release/{}", target_dir, module.replace("-", "_")),
    ];

    let binary = candidates
        .iter()
        .find(|p| Path::new(p).exists())
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Binary not found. Tried: {}. \
                 Run this command from the workspace root, or set CARGO_TARGET_DIR.",
                candidates.join(", ")
            )
        })?;

    // Step 2: run the module and collect process metrics
    println!("Running {} for {} seconds...", module, duration);

    let run_start = std::time::Instant::now();
    let mut child = Command::new(&binary)
        .env("WIREFRAME_AI_PROFILE", "true")
        .env("WIREFRAME_AI_NATS_URL", "nats://localhost:4222")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| anyhow::anyhow!("Failed to start module: {}", e))?;

    let pid = child.id();
    let child_pid = sysinfo::Pid::from(pid as usize);

    // Spawn background threads to drain stdout/stderr so the child never
    // blocks on full OS pipe buffers while we are profiling.
    let stdout_buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let stderr_buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    if let Some(mut out) = child.stdout.take() {
        let buf = Arc::clone(&stdout_buf);
        std::thread::spawn(move || {
            let mut local = String::new();
            let _ = std::io::Read::read_to_string(&mut out, &mut local);
            if let Ok(mut b) = buf.lock() {
                *b = local;
            }
        });
    }
    if let Some(mut err) = child.stderr.take() {
        let buf = Arc::clone(&stderr_buf);
        std::thread::spawn(move || {
            let mut local = String::new();
            let _ = std::io::Read::read_to_string(&mut err, &mut local);
            if let Ok(mut b) = buf.lock() {
                *b = local;
            }
        });
    }

    use sysinfo::{ProcessRefreshKind, RefreshKind, System};
    let mut system = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::everything()),
    );

    let mut samples: Vec<serde_json::Value> = Vec::new();
    let mut max_memory_kb: u64 = 0;
    let mut total_memory_kb: u64 = 0;
    let mut sample_count: u64 = 0;

    let interval = std::time::Duration::from_secs(1);
    let end_time = run_start + std::time::Duration::from_secs(duration);

    while std::time::Instant::now() < end_time {
        std::thread::sleep(interval);
        system.refresh_process(child_pid);

        if let Some(process) = system.process(child_pid) {
            let mem_kb = process.memory() / 1024;
            let cpu = process.cpu_usage();
            max_memory_kb = max_memory_kb.max(mem_kb);
            total_memory_kb += mem_kb;
            sample_count += 1;

            samples.push(serde_json::json!({
                "elapsed_ms": run_start.elapsed().as_millis(),
                "memory_kb": mem_kb,
                "cpu_percent": cpu,
            }));
        }
    }

    // Check whether the process exited on its own before we had to kill it.
    let exited_early = match child.try_wait()? {
        Some(status) => !status.success(),
        None => false,
    };

    // Collect output from the background drain threads.
    let stdout_text = stdout_buf.lock().map(|b| b.clone()).unwrap_or_default();
    let stderr_text = stderr_buf.lock().map(|b| b.clone()).unwrap_or_default();

    // Cross-platform graceful termination: send a kill signal and wait() to
    // reap the process, preventing zombies and leaked handles.
    if let Err(e) = child.kill() {
        eprintln!(
            "Warning: failed to send kill signal to child process: {}",
            e
        );
    }
    let exit_status = child.wait().ok().and_then(|s| s.code());
    let run_duration = run_start.elapsed();

    // Collect final process stats if still available
    system.refresh_process(child_pid);
    let final_mem_kb = system.process(child_pid).map(|p| p.memory() / 1024);

    let report = serde_json::json!({
        "module": module,
        "requested_duration_seconds": duration,
        "actual_run_duration_ms": run_duration.as_millis(),
        "pid": pid,
        "profiled_at": chrono::Utc::now().timestamp(),
        "started_ok": true,
        "exited_early": exited_early,
        "exit_code": exit_status,
        "memory": {
            "max_kb": max_memory_kb,
            "avg_kb": total_memory_kb.checked_div(sample_count).unwrap_or(0),
            "final_kb": final_mem_kb,
            "samples": sample_count,
        },
        "cpu_samples": samples,
        "stderr_preview": stderr_text.lines().take(20).collect::<Vec<_>>(),
        "stdout_bytes": stdout_text.len(),
        "stderr_bytes": stderr_text.len(),
        "note": "Profile report with per-second memory and CPU samples. For deep CPU flamegraphs use: perf, samply, or VTune.",
    });
    fs::write(&report_file, serde_json::to_string_pretty(&report)?)?;
    println!("Profile report written to: {}", report_file);

    Ok(())
}

fn handle_dev(module: &str, nats_url: &str) -> anyhow::Result<()> {
    validate_module_name(module)?;

    // Prefer cargo-watch if installed
    let cargo_watch = Command::new("cargo")
        .args(["watch", "--version"])
        .output()?;
    if cargo_watch.status.success() {
        println!("Starting hot-reload development mode for: {}", module);
        println!("  NATS URL: {}", nats_url);
        let mut cmd = Command::new("cargo");
        cmd.args(["watch", "-x", &format!("run -p {}", module)]);
        cmd.env("WIREFRAME_AI_NATS_URL", nats_url);
        let status = cmd.status()?;
        if !status.success() {
            anyhow::bail!("cargo watch exited with non-zero status");
        }
        return Ok(());
    }

    // Fallback: manual recompile loop.
    // Poll source timestamps and re-run `cargo run -p <module>` on change.
    println!(
        "cargo-watch not found. Starting manual recompile loop for: {}",
        module
    );
    println!("  NATS URL: {}", nats_url);
    println!("  Press Ctrl+C to stop.\n");

    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    let workspace_root = std::env::current_dir()?;

    // Module source may live in the workspace root (e.g. `my-module/src`) or in a
    // nested directory like `modules/my-module/src` or `examples/my-module/src`.
    let src_dir = {
        let direct = Path::new(module).join("src");
        if direct.exists() {
            direct
        } else {
            // Try common workspace subdirectories
            let nested = ["modules", "examples", "tools"]
                .iter()
                .map(|d| Path::new(d).join(module).join("src"))
                .find(|p| p.exists());
            match nested {
                Some(p) => p,
                None => {
                    anyhow::bail!(
                        "Could not find source directory for module '{}'. \
                         Tried: {} and common nested paths.",
                        module,
                        direct.display()
                    );
                }
            }
        }
    };

    fn scan_mtimes(dir: &Path, mtimes: &mut HashMap<PathBuf, SystemTime>) -> anyhow::Result<()> {
        if !dir.exists() {
            return Ok(());
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let meta = entry.metadata()?;
            // Skip symlinks to avoid cycles and unexpected paths.
            if meta.file_type().is_symlink() {
                continue;
            }
            if meta.is_dir() {
                scan_mtimes(&path, mtimes)?;
            } else if meta.is_file() {
                if let Ok(mtime) = meta.modified() {
                    mtimes.insert(path, mtime);
                }
            }
        }
        Ok(())
    }

    fn changed(old: &HashMap<PathBuf, SystemTime>, new: &HashMap<PathBuf, SystemTime>) -> bool {
        if old.len() != new.len() {
            return true;
        }
        new.iter().any(|(path, mtime)| old.get(path) != Some(mtime))
    }

    let mut previous: HashMap<PathBuf, SystemTime> = HashMap::new();
    scan_mtimes(&workspace_root.join(&src_dir), &mut previous)?;

    let mut first_run = true;
    let mut child: Option<std::process::Child> = None;

    loop {
        // Check whether the current child exited on its own (crash or clean stop).
        if let Some(ref mut c) = child {
            match c.try_wait() {
                Ok(Some(status)) => {
                    let code = status
                        .code()
                        .map_or_else(|| "signal".to_string(), |c| c.to_string());
                    println!("\n[dev] {} exited unexpectedly (exit code: {}). Restarting on next change...\n", module, code);
                    let _ = c.wait(); // reap zombie
                    child = None;
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("[dev] Warning: could not check child status: {}", e);
                }
            }
        }

        let mut current: HashMap<PathBuf, SystemTime> = HashMap::new();
        scan_mtimes(&workspace_root.join(&src_dir), &mut current)?;

        if first_run || changed(&previous, &current) {
            first_run = false;
            previous = current;
            println!("\n[dev] Change detected — rebuilding {}...\n", module);

            // Terminate the previous instance so the port / NATS subscription is released.
            if let Some(mut c) = child.take() {
                println!("[dev] Stopping previous instance (pid {})...", c.id());
                if let Err(e) = c.kill() {
                    eprintln!("[dev] Warning: failed to kill previous instance: {}", e);
                }
                // Reap the process to prevent zombies and free handles.
                let _ = c.wait();
            }

            let mut cmd = Command::new("cargo");
            cmd.args(["run", "-p", module]);
            cmd.env("WIREFRAME_AI_NATS_URL", nats_url);
            cmd.current_dir(&workspace_root);
            // Inherit stdout/stderr so the module output is visible and the
            // child never blocks on full pipe buffers.
            cmd.stdout(std::process::Stdio::inherit());
            cmd.stderr(std::process::Stdio::inherit());

            match cmd.spawn() {
                Ok(c) => {
                    println!(
                        "[dev] {} spawned (pid {}). Waiting for changes...\n",
                        module,
                        c.id()
                    );
                    child = Some(c);
                }
                Err(e) => {
                    eprintln!("[dev] Failed to start {}: {}", module, e);
                    println!("\n[dev] Build/run failed — fix errors and save to retry.\n");
                }
            }
        }

        std::thread::sleep(Duration::from_secs(2));
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Try to find a subcommand binary in PATH, falling back to cargo run.
fn find_subcommand(name: &str) -> Option<Command> {
    if which(name).is_ok() {
        Some(Command::new(name))
    } else {
        None
    }
}

/// Check if a command exists in PATH.
fn which(name: &str) -> Result<std::path::PathBuf, std::io::Error> {
    let path_env = std::env::var_os("PATH").ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "PATH environment variable not set",
        )
    })?;
    for dir in std::env::split_paths(&path_env) {
        let candidate = dir.join(name);
        #[cfg(target_os = "windows")]
        let candidate = candidate.with_extension("exe");
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("{} not found in PATH", name),
    ))
}

/// Escape a string for safe use inside a single-quoted shell command.
fn shell_escape(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    if s.chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/')
    {
        return s.to_string();
    }
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn to_pascal_case(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect()
}

// ── Module Management Commands ──────────────────────────────────────────────────

fn handle_module_command(cmd: &ModuleCommands) -> anyhow::Result<()> {
    match cmd {
        ModuleCommands::List { detailed } => {
            handle_module_list(*detailed)?;
        }
        ModuleCommands::Start {
            name,
            build_mode,
            nats_url,
        } => {
            handle_module_start(name, build_mode, nats_url)?;
        }
        ModuleCommands::Stop { name, force } => {
            handle_module_stop(name, *force)?;
        }
        ModuleCommands::Status { name } => {
            handle_module_status(name)?;
        }
        ModuleCommands::Logs {
            name,
            lines,
            follow,
        } => {
            handle_module_logs(name, *lines, *follow)?;
        }
    }
    Ok(())
}

fn handle_module_list(detailed: bool) -> anyhow::Result<()> {
    println!("Installed Wireframe-AI modules:");
    println!();

    let modules = vec![
        (
            "wireframe-ai-interface",
            "Kernel interface module",
            "kernel/interface",
        ),
        (
            "wireframe-ai-context-core",
            "Context module with plugin support",
            "modules/context-core",
        ),
        (
            "wireframe-ai-orchestrator-core",
            "Orchestrator module with plugin support",
            "modules/orchestrator-core",
        ),
        (
            "wireframe-ai-sandbox-core",
            "Sandbox module with plugin support",
            "modules/sandbox-core",
        ),
        (
            "wireframe-adapter-rust",
            "Rust reasoning adapter",
            "adapter/rust",
        ),
        ("wireframe-tui", "Terminal UI interface", "tools/tui-chat"),
    ];

    for (name, description, path) in modules {
        if detailed {
            println!("  {} — {}", name, description);
            println!("    Path: {}", path);
            println!();
        } else {
            println!("  {}", name);
        }
    }

    Ok(())
}

fn handle_module_start(name: &str, build_mode: &str, nats_url: &str) -> anyhow::Result<()> {
    println!("Starting module: {}", name);
    println!("Build mode: {}", build_mode);
    println!("NATS URL: {}", nats_url);

    let build_flag = if build_mode == "release" {
        "--release"
    } else {
        ""
    };

    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg(build_flag).arg("-p").arg(name);

    if let Err(e) = cmd.spawn() {
        return Err(anyhow::anyhow!("Failed to start module {}: {}", name, e));
    }

    println!("Module {} started successfully", name);
    Ok(())
}

fn handle_module_stop(name: &str, force: bool) -> anyhow::Result<()> {
    println!("Stopping module: {}", name);

    // Extract process name from module name
    let process_name = name.replace("wireframe-ai-", "").replace("wireframe-", "");

    let signal = if force { "SIGKILL" } else { "SIGTERM" };

    #[cfg(unix)]
    {
        use std::process::Command;
        let output = Command::new("pkill")
            .arg(if force { "-9" } else { "-15" })
            .arg(&process_name)
            .output()?;

        if output.status.success() {
            println!("Module {} stopped ({})", name, signal);
        } else {
            println!("No running process found for {}", name);
        }
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        let output = Command::new("taskkill")
            .arg(if force { "/F" } else { "" })
            .arg("/IM")
            .arg(&format!("{}.exe", process_name))
            .output()?;

        if output.status.success() {
            println!("Module {} stopped ({})", name, signal);
        } else {
            println!("No running process found for {}", name);
        }
    }

    Ok(())
}

fn handle_module_status(name: &Option<String>) -> anyhow::Result<()> {
    if let Some(module_name) = name {
        println!("Status for module: {}", module_name);
        // Check if module process is running
        let process_name = module_name
            .replace("wireframe-ai-", "")
            .replace("wireframe-", "");

        #[cfg(unix)]
        {
            use std::process::Command;
            let output = Command::new("pgrep").arg(&process_name).output()?;

            if output.status.success() {
                let pid = String::from_utf8_lossy(&output.stdout);
                println!("  Status: Running (PID: {})", pid.trim());
            } else {
                println!("  Status: Stopped");
            }
        }

        #[cfg(windows)]
        {
            use std::process::Command;
            let output = Command::new("tasklist")
                .arg("/FI")
                .arg(&format!("IMAGENAME eq {}.exe", process_name))
                .output()?;

            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.contains(&format!("{}.exe", process_name)) {
                    println!("  Status: Running");
                } else {
                    println!("  Status: Stopped");
                }
            }
        }
    } else {
        println!("Status for all modules:");
        println!("  wireframe-ai-interface: Unknown (use specific module name)")
    }

    Ok(())
}

fn handle_module_logs(name: &str, lines: usize, follow: bool) -> anyhow::Result<()> {
    println!("Showing logs for module: {} (last {} lines)", name, lines);

    // In a real implementation, this would read from log files
    // For now, we'll show a placeholder message
    println!("Note: Log file reading not yet implemented");
    println!("Module logs are typically written to stdout/stderr");
    println!("Use the debug command to inspect NATS messages instead:");

    if follow {
        println!("  wireframe debug --topic > --follow");
    } else {
        println!("  wireframe debug --topic >");
    }

    Ok(())
}

// ── Main ────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            name,
            template,
            out_dir,
        } => {
            let dir = out_dir.unwrap_or_else(|| name.clone());
            let path = Path::new(&dir);
            if path.exists() {
                eprintln!("Error: directory '{}' already exists", dir);
                std::process::exit(1);
            }
            scaffold_module(&name, &template, path)?;
        }
        Commands::Init { name, template } => {
            let path = std::env::current_dir()?;
            scaffold_module(&name, &template, &path)?;
        }
        Commands::ListTemplates => {
            println!("Available templates:");
            println!("  basic         — Basic module with #[module] macro");
            println!("  adapter       — Reasoning adapter (agent.job -> agent.result)");
            println!("  context       — Context enrichment (task.submitted -> task.enriched)");
            println!(
                "  orchestrator  — Task orchestrator (task.enriched -> agent.job + task.complete)"
            );
            println!("  listener      — Message listener (logs all messages)");
            println!("  service       — Request/response service handler");
            println!("  webhook       — Webhook receiver module");
            println!("  integration   — External API integration bridge");
            println!("  cache         — TTL-based caching module");
            println!("  rate-limiter  — Token-bucket rate limiter");
        }
        Commands::Test {
            module,
            integration,
        } => {
            if let Err(e) = handle_test(&module, integration) {
                eprintln!("Test failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Deploy {
            module,
            target,
            docker,
        } => {
            if let Err(e) = handle_deploy(&module, &target, docker) {
                eprintln!("Deploy failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Debug {
            topic,
            correlation,
            format,
            capture,
        } => {
            if let Err(e) = handle_debug(&topic, &correlation, &format, &capture) {
                eprintln!("Debug failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Validate {
            module,
            registry,
            schemas,
        } => {
            if let Err(e) = handle_validate(&module, registry, schemas) {
                eprintln!("Validation failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Replay {
            file,
            nats_url,
            speed,
            topic,
        } => {
            if let Err(e) = handle_replay(&file, &nats_url, speed, &topic) {
                eprintln!("Replay failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Profile {
            module,
            duration,
            output,
        } => {
            if let Err(e) = handle_profile(&module, duration, &output) {
                eprintln!("Profile failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Dev { module, nats_url } => {
            // Dev mode contains an infinite polling loop; run it on a blocking thread
            // so it does not starve the async runtime.
            if let Err(e) =
                tokio::task::spawn_blocking(move || handle_dev(&module, &nats_url)).await?
            {
                eprintln!("Dev mode failed: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Module { module_cmd } => {
            if let Err(e) = handle_module_command(&module_cmd) {
                eprintln!("Module command failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
