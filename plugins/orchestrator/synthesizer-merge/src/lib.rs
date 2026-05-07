//! Result synthesizer — merges multiple agent results into a single task completion.

use agentic_sdk::message_types::{AgentResult, SideEffect, TaskComplete, TaskEnriched};
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::orchestrator::{ResultSynthesizer, SynthesisError};
use async_trait::async_trait;
use serde_json::Value;

/// Merge synthesizer that combines results from multiple agents.
pub struct MergeSynthesizer;

impl MergeSynthesizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MergeSynthesizer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for MergeSynthesizer {
    fn plugin_id(&self) -> &'static str {
        "synthesizer-merge"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Merge synthesizer for combining agent results"
    }

    async fn initialize(
        &mut self,
        _config: &Value,
    ) -> Result<(), agentic_sdk::plugin::PluginError> {
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
impl ResultSynthesizer for MergeSynthesizer {
    async fn synthesize(
        &self,
        results: Vec<AgentResult>,
        original_task: &TaskEnriched,
    ) -> Result<TaskComplete, SynthesisError> {
        let collected = results.len();
        if collected == 0 {
            return Err(SynthesisError::NoResults);
        }

        let mut combined_result = String::new();
        let mut all_side_effects: Vec<SideEffect> = Vec::new();
        let mut all_warnings: Vec<String> = Vec::new();

        for (i, r) in results.iter().enumerate() {
            if let Some(ref text) = r.output.text {
                if !combined_result.is_empty() {
                    combined_result.push_str("\n\n---\n\n");
                }
                combined_result.push_str(&format!("## Agent {} Output\n\n{}", i + 1, text));
            }
            if let Some(ref structured) = r.output.structured {
                if !combined_result.is_empty() {
                    combined_result.push_str("\n\n---\n\n");
                }
                combined_result.push_str(&format!(
                    "## Agent {} Structured Output\n\n{}",
                    i + 1,
                    serde_json::to_string_pretty(structured).unwrap_or_default()
                ));
            }
            for f in &r.output.files_written {
                all_side_effects.push(SideEffect {
                    kind: "file_written".into(),
                    description: format!("Agent {} wrote: {}", i + 1, f.display()),
                    path: Some(f.clone()),
                });
            }
            for cmd in &r.output.commands_run {
                all_side_effects.push(SideEffect {
                    kind: "command_executed".into(),
                    description: format!("Agent {} ran: {}", i + 1, cmd),
                    path: None,
                });
            }
            for tool in &r.tool_invocations {
                all_side_effects.push(SideEffect {
                    kind: "tool_invocation".into(),
                    description: format!(
                        "Agent {} called tool {} ({}ms)",
                        i + 1,
                        tool.tool_name,
                        tool.duration_ms
                    ),
                    path: None,
                });
            }
            for err in &r.errors {
                all_warnings.push(format!(
                    "Agent {} error [{}]: {}",
                    i + 1,
                    err.code,
                    err.message
                ));
            }
        }

        Ok(TaskComplete {
            session_id: original_task.session_id.clone(),
            correlation_id: original_task.correlation_id.clone(),
            result: if combined_result.is_empty() {
                format!(
                    "Processed task with {} agents (no text output produced)",
                    collected
                )
            } else {
                combined_result
            },
            side_effects: all_side_effects,
            warnings: all_warnings,
            completed_at: chrono::Utc::now().timestamp(),
        })
    }
}
