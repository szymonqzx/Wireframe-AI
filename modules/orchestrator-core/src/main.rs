//! Wireframe-AI Orchestrator Core Module
//!
//! Handles task orchestration by:
//! - Decomposing tasks using planner plugins
//! - Executing subtasks using execution plugins
//! - Synthesizing results using synthesizer plugins
//!
//! Subscribes to: task.enriched
//! Publishes to: task.complete

use agentic_sdk::Module;
use tracing::{error, info};
use wireframe_ai_orchestrator_core::OrchestratorCore;
use wireframe_config::WireframeConfig;

const MAX_SESSION_ID_LENGTH: usize = 256;
const MAX_CORRELATION_ID_LENGTH: usize = 256;
const MAX_USER_INPUT_LENGTH: usize = 10000;

struct OrchestratorCoreModule {
    core: OrchestratorCore,
}

#[agentic_sdk::module(
    subscribes = ["task.enriched"],
    publishes = ["task.complete"],
    queue_group = "task_handler"
)]
impl Module for OrchestratorCoreModule {
    async fn handle(&mut self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        // Deserialize TaskEnriched
        let task: agentic_sdk::message_types::TaskEnriched = match serde_json::from_value(env.payload.clone()) {
            Ok(t) => t,
            Err(e) => {
                error!(error = ?e, "failed to deserialize TaskEnriched");
                return vec![];
            }
        };

        // Validate session_id
        if let Err(e) = validate_session_id(&task.session_id) {
            error!(error = %e, session = %task.session_id, "invalid session_id");
            return vec![];
        }

        // Validate correlation_id
        if let Err(e) = validate_correlation_id(&task.correlation_id) {
            error!(error = %e, correlation = %task.correlation_id, "invalid correlation_id");
            return vec![];
        }

        // Validate user_input
        if let Err(e) = validate_user_input(&task.user_input) {
            error!(error = %e, "invalid user_input length");
            return vec![];
        }

        info!(
            session = %task.session_id,
            correlation = %task.correlation_id,
            "enriched task received — processing through orchestrator core"
        );

        // Process task through orchestrator core
        match self.core.process_task(task.clone()).await {
            Ok(complete) => {
                vec![env.reply("task.complete", serde_json::to_value(complete).unwrap_or_default())]
            }
            Err(e) => {
                error!(error = ?e, "failed to process task");
                vec![]
            }
        }
    }
}

/// Validates session_id format and length
fn validate_session_id(session_id: &str) -> Result<(), String> {
    if session_id.len() > MAX_SESSION_ID_LENGTH {
        return Err(format!(
            "session_id exceeds maximum length of {}",
            MAX_SESSION_ID_LENGTH
        ));
    }
    if session_id.is_empty() {
        return Err("session_id cannot be empty".to_string());
    }
    if !session_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err("session_id contains invalid characters".to_string());
    }
    Ok(())
}

/// Validates correlation_id format and length
fn validate_correlation_id(correlation_id: &str) -> Result<(), String> {
    if correlation_id.len() > MAX_CORRELATION_ID_LENGTH {
        return Err(format!(
            "correlation_id exceeds maximum length of {}",
            MAX_CORRELATION_ID_LENGTH
        ));
    }
    if correlation_id.is_empty() {
        return Err("correlation_id cannot be empty".to_string());
    }
    if !correlation_id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err("correlation_id contains invalid characters".to_string());
    }
    Ok(())
}

/// Validates user_input length
fn validate_user_input(user_input: &str) -> Result<(), String> {
    if user_input.len() > MAX_USER_INPUT_LENGTH {
        return Err(format!(
            "user_input exceeds maximum length of {}",
            MAX_USER_INPUT_LENGTH
        ));
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = WireframeConfig::from_env()?;
    let nats_url = config.nats_url();

    info!("Orchestrator-core started — loading plugins");

    // Create orchestrator core
    let core = OrchestratorCore::new();

    // Note: Plugins will be manually registered here once they are created
    // For now, we'll proceed without plugins and handle the error gracefully
    info!("Orchestrator-core ready (plugins will be registered when available)");

    let module = OrchestratorCoreModule { core };

    info!("orchestrator core starting on {}", nats_url);
    module.run(&nats_url).await
}
