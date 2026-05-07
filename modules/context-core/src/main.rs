//! Wireframe-AI Context Core Module
//!
//! Handles context enrichment for tasks by:
//! - Loading session history from storage plugins
//! - Searching memory via memory plugins
//! - Running enrichment pipeline
//! - Persisting assistant responses to memory
//!
//! Subscribes to: task.submitted, task.complete
//! Publishes to: task.enriched

use agentic_sdk::{Envelope, Module};
use serde_json::Value;
use std::sync::Arc;
use tracing::{error, info};
use wireframe_ai_context_core::ContextCore;
use wireframe_config::WireframeConfig;

struct ContextCoreModule {
    core: Arc<ContextCore>,
}

#[agentic_sdk::module(
    subscribes = ["task.submitted", "task.complete"],
    publishes = ["task.enriched"],
    queue_group = "task_handler"
)]
impl Module for ContextCoreModule {
    async fn handle(&mut self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        match env.topic.as_str() {
            "task.submitted" => self.handle_task_submitted(env).await,
            "task.complete" => self.handle_task_complete(env).await,
            _ => {
                error!(topic = %env.topic, "unknown topic");
                vec![]
            }
        }
    }
}

impl ContextCoreModule {
    async fn handle_task_submitted(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        // Deserialize TaskSubmitted
        let task: agentic_sdk::message_types::TaskSubmitted = match serde_json::from_value(env.payload.clone()) {
            Ok(t) => t,
            Err(e) => {
                error!(error = ?e, "failed to deserialize TaskSubmitted");
                return vec![];
            }
        };

        info!(session = %task.session_id, "processing task.submitted");

        // Convert envelope to expected type
        let typed_envelope = agentic_sdk::envelope::Envelope {
            message_id: env.message_id.clone(),
            session_id: env.session_id.clone(),
            correlation_id: env.correlation_id.clone(),
            topic: env.topic.clone(),
            payload: task.clone(),
            schema_version: env.schema_version.clone(),
            timestamp: env.timestamp,
        };

        // Process task
        match self.core.handle_task(task, typed_envelope).await {
            Ok(enriched) => {
                vec![env.reply("task.enriched", serde_json::to_value(enriched).unwrap_or_default())]
            }
            Err(e) => {
                error!(error = ?e, "failed to handle task");
                vec![]
            }
        }
    }

    async fn handle_task_complete(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        // Deserialize TaskComplete
        let complete: agentic_sdk::message_types::TaskComplete = match serde_json::from_value(env.payload.clone()) {
            Ok(c) => c,
            Err(e) => {
                error!(error = ?e, "failed to deserialize TaskComplete");
                return vec![];
            }
        };

        info!(session = %complete.session_id, "persisting task.complete as memory");

        // Persist to memory
        if let Err(e) = self.core.handle_complete(complete).await {
            error!(error = ?e, "failed to handle task.complete");
        }

        vec![]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = WireframeConfig::from_env()?;
    let nats_url = config.nats_url();
    let max_session_history = config.context.max_session_history;
    let max_memory_chunks = config.context.max_memory_chunks;
    let max_context_tokens = config.context.max_context_tokens;

    // Create context core
    let core = Arc::new(ContextCore::new(
        max_session_history,
        max_memory_chunks,
        max_context_tokens,
    ));

    // TODO: Load plugins from configuration
    // For now, this is a placeholder - actual plugin loading will be implemented in later phases

    let module = ContextCoreModule { core };

    info!("context core starting on {}", nats_url);
    module.run(&nats_url).await
}
