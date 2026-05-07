//! CLI input method — reads user input from stdin.

use agentic_sdk::message_types::TaskSubmitted;
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::interface::InputMethod;
use async_trait::async_trait;
use serde_json::Value;
use uuid::Uuid;

/// CLI input that reads from stdin.
pub struct CliInput {
    prompt: String,
}

impl CliInput {
    pub fn new() -> Self {
        Self {
            prompt: "> ".to_string(),
        }
    }

    pub fn with_prompt(prompt: String) -> Self {
        Self { prompt }
    }
}

impl Default for CliInput {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for CliInput {
    fn plugin_id(&self) -> &'static str {
        "input-cli"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "CLI input method for interface"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        if let Some(prompt) = config.get("prompt").and_then(|v| v.as_str()) {
            self.prompt = prompt.to_string();
        }
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, agentic_sdk::plugin::PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }
}

#[async_trait]
impl InputMethod for CliInput {
    async fn read_input(
        &self,
    ) -> Result<TaskSubmitted, agentic_sdk::plugins::interface::InputError> {
        use std::io::{self, Write};

        print!("{}", self.prompt);
        io::stdout()
            .flush()
            .map_err(|e| agentic_sdk::plugins::interface::InputError::ReadFailed(e.to_string()))?;

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .map_err(|e| agentic_sdk::plugins::interface::InputError::ReadFailed(e.to_string()))?;

        let user_input = input.trim().to_string();

        // Construct a basic TaskSubmitted
        Ok(TaskSubmitted {
            session_id: Uuid::new_v4().to_string(),
            user_input,
            submitted_at: chrono::Utc::now().timestamp(),
        })
    }
}
