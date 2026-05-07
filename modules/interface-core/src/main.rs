//! Wireframe-AI Interface Core Module
//!
//! Handles user interaction by:
//! - Reading user input via input plugins
//! - Publishing task.submitted messages
//! - Receiving task.complete messages
//! - Formatting results via output plugins
//!
//! Subscribes to: task.complete
//! Publishes to: task.submitted

use agentic_sdk::{Envelope, Module};
use agentic_sdk::plugins::interface::{InputError, FormatError};
use async_nats::Client;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};
use wireframe_ai_interface_core::InterfaceCore;
use wireframe_config::WireframeConfig;

#[derive(Clone)]
struct InterfaceCoreModule {
    core: Arc<InterfaceCore>,
    nats_client: Option<Client>,
    _task_sender: Option<mpsc::Sender<agentic_sdk::message_types::TaskSubmitted>>,
}

#[agentic_sdk::module(
    subscribes = ["task.complete"],
    publishes = ["task.submitted"],
    queue_group = "interface"
)]
impl Module for InterfaceCoreModule {
    async fn handle(&mut self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        // Deserialize TaskComplete
        let complete: agentic_sdk::message_types::TaskComplete = match serde_json::from_value(env.payload.clone()) {
            Ok(c) => c,
            Err(e) => {
                error!(error = ?e, "failed to deserialize TaskComplete");
                return vec![];
            }
        };

        info!(session = %complete.session_id, "received task.complete");

        // Format result using output plugin
        match self.core.format_result(&complete).await {
            Ok(formatted) => {
                println!("\n{}", formatted);
            }
            Err(FormatError::FormattingFailed(e)) => {
                error!(error = %e, "failed to format result");
                // Fallback to raw output
                println!("\n{}", complete.result);
            }
            Err(FormatError::SerializationFailed(e)) => {
                error!(error = %e, "failed to serialize result");
                println!("\n{}", complete.result);
            }
        }

        vec![]
    }
}

impl InterfaceCoreModule {
    async fn publish_task_submitted(&self, task: agentic_sdk::message_types::TaskSubmitted) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(client) = &self.nats_client {
            let envelope = Envelope::new("task.submitted", task.clone(), Some(task.session_id.clone()));
            let payload = serde_json::to_string(&envelope)?;
            client.publish("task.submitted", payload.into()).await?;
            info!(session = %task.session_id, "published task.submitted");
        }
        Ok(())
    }

    async fn run_cli_loop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting CLI loop");
        println!("Wireframe-AI Interface Core");
        println!("Type your request and press Enter, or 'quit' to exit\n");

        loop {
            // Read input using input plugin
            match self.core.read_input().await {
                Ok(task) => {
                    // Publish to NATS
                    if let Err(e) = self.publish_task_submitted(task).await {
                        error!(error = ?e, "failed to publish task.submitted");
                    }
                }
                Err(InputError::Interrupted) => {
                    info!("User requested quit");
                    break;
                }
                Err(InputError::ReadFailed(e)) => {
                    error!(error = %e, "failed to read input");
                    // Continue the loop even if input fails
                }
                Err(InputError::ParseFailed(e)) => {
                    error!(error = %e, "failed to parse input");
                    // Continue the loop even if input fails
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = WireframeConfig::from_env()?;
    let nats_url = config.nats_url().to_string(); // Clone the URL

    info!("Interface-core started — loading plugins");

    // Create interface core
    let core = Arc::new(InterfaceCore::new());

    // TODO: Load plugins from config when plugin loading is implemented
    // For now, we'll use default CLI input/output
    info!("Interface-core ready (using default CLI input/output)");

    // Connect to NATS
    let client = async_nats::connect(&nats_url).await?;
    info!("Connected to NATS at {}", nats_url);

    // Create channel for CLI loop to send tasks to NATS module
    let (task_sender, mut task_receiver) = mpsc::channel::<agentic_sdk::message_types::TaskSubmitted>(32);

    // Create module with NATS client
    let mut module = InterfaceCoreModule {
        core: core.clone(),
        nats_client: Some(client.clone()),
        _task_sender: Some(task_sender),
    };

    // Start the NATS module in the background
    let nats_url_clone = nats_url.clone();
    let module_for_nats = module.clone();
    let module_handle = tokio::spawn(async move {
        if let Err(e) = module_for_nats.run(&nats_url_clone).await {
            error!(error = ?e, "NATS module failed");
        }
    });

    // Also start a task to publish CLI input to NATS
    let client_clone = client.clone();
    let publish_handle = tokio::spawn(async move {
        while let Some(task) = task_receiver.recv().await {
            let envelope = Envelope::new("task.submitted", task.clone(), Some(task.session_id.clone()));
            if let Ok(payload) = serde_json::to_string(&envelope) {
                if let Err(e) = client_clone.publish("task.submitted", payload.into()).await {
                    error!(error = ?e, "failed to publish task.submitted from CLI");
                }
            }
        }
    });

    // Run CLI loop in the foreground
    info!("Interface-core starting CLI loop");
    info!("Press Ctrl+C to exit");

    // Set up graceful shutdown
    let shutdown_signal = tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl+c");
        info!("Received shutdown signal");
    });

    // Run CLI loop
    let cli_result = module.run_cli_loop().await;

    // Cancel background tasks
    module_handle.abort();
    publish_handle.abort();
    shutdown_signal.abort();

    // Announce offline
    let _ = agentic_sdk::announce_offline(&client, "wireframe-ai-interface-core", "0.1.0").await;

    cli_result
}
