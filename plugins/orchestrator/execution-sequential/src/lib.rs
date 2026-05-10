//! Sequential execution strategy — executes tasks one at a time.

use agentic_sdk::envelope::Envelope;
use agentic_sdk::message_types::{AgentJob, AgentResult};
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::orchestrator::{ExecutionError, ExecutionStrategy};
use async_nats::Client;
use async_trait::async_trait;
use futures::StreamExt;
use serde_json::Value;
use std::collections::HashSet;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info};

/// Sequential execution strategy with NATS dispatch and collection.
pub struct SequentialExecution {
    nats_client: Client,
    result_timeout_secs: u64,
}

impl SequentialExecution {
    pub fn new(nats_client: Client) -> Self {
        Self {
            nats_client,
            result_timeout_secs: 600, // Default 10 minutes
        }
    }

    pub fn with_timeout(nats_client: Client, timeout_secs: u64) -> Self {
        Self {
            nats_client,
            result_timeout_secs: timeout_secs,
        }
    }

    pub fn nats_client(&self) -> &Client {
        &self.nats_client
    }
}

#[async_trait]
impl Plugin for SequentialExecution {
    fn plugin_id(&self) -> &'static str {
        "execution-sequential"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Sequential execution strategy for orchestrator"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        if let Some(timeout) = config.get("result_timeout_secs").and_then(|v| v.as_u64()) {
            self.result_timeout_secs = timeout;
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
impl ExecutionStrategy for SequentialExecution {
    async fn dispatch_jobs(
        &self,
        jobs: Vec<AgentJob>,
    ) -> Result<Vec<String>, ExecutionError> {
        let mut job_ids = Vec::new();

        // Dispatch jobs sequentially (one at a time)
        for (i, job) in jobs.into_iter().enumerate() {
            let client = self.nats_client.clone();
            let job_id = job.job_id.clone();
            let session_id = job.task.user_input.clone(); // Use user_input as session placeholder

            job_ids.push(job_id.clone());

            let job_envelope = Envelope::new("agent.job", job, Some(session_id));

            let payload = match serde_json::to_string(&job_envelope) {
                Ok(p) => p,
                Err(e) => {
                    error!(error = ?e, "failed to serialize agent.job {}", i);
                    continue;
                }
            };

            if let Err(e) = client.publish("agent.job", payload.into()).await {
                error!(error = ?e, "failed to publish agent.job {}", i);
            } else {
                info!("published agent.job {} to subject 'agent.job' (sequential)", i);
            }
            debug!("published agent.job {} (sequential)", i);
        }

        Ok(job_ids)
    }

    async fn collect_results(
        &self,
        correlation_parent: &str,
        expected_count: usize,
    ) -> Result<Vec<AgentResult>, ExecutionError> {
        // Subscribe to agent.result
        let mut result_sub = self
            .nats_client
            .queue_subscribe("agent.result", "orchestrator_sequential_collector".to_string())
            .await
            .map_err(|e| {
                error!(error = ?e, "failed to subscribe to agent.result");
                ExecutionError::CollectionFailed(e.to_string())
            })?;

        let mut results: Vec<AgentResult> = Vec::new();
        let mut seen_job_ids: HashSet<String> = HashSet::new();
        let deadline = tokio::time::Instant::now() + Duration::from_secs(self.result_timeout_secs);

        while results.len() < expected_count {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                error!(
                    correlation = %correlation_parent,
                    "timed out collecting agent.results — got {}/{}",
                    results.len(),
                    expected_count
                );
                break;
            }

            let result_msg = match timeout(remaining, result_sub.next()).await {
                Ok(Some(msg)) => msg,
                Ok(None) => {
                    error!("agent.result subscription ended unexpectedly");
                    break;
                }
                Err(_) => {
                    error!(
                        correlation = %correlation_parent,
                        "timed out collecting agent.results — got {}/{}",
                        results.len(),
                        expected_count
                    );
                    break;
                }
            };

            let result_envelope: Envelope<AgentResult> =
                match serde_json::from_slice(&result_msg.payload) {
                    Ok(e) => e,
                    Err(e) => {
                        error!(error = ?e, "failed to parse agent.result");
                        continue;
                    }
                };

            let agent_result = result_envelope.payload;

            // Only collect results matching our parent correlation
            if agent_result.correlation_parent != correlation_parent {
                debug!("ignoring agent.result for different correlation parent");
                continue;
            }

            // Deduplicate by job_id
            if !seen_job_ids.insert(agent_result.job_id.clone()) {
                debug!("duplicate agent.result for job {}", agent_result.job_id);
                continue;
            }

            info!(
                job = %agent_result.job_id,
                "collected agent.result ({}/{})",
                results.len() + 1,
                expected_count
            );
            results.push(agent_result);
        }

        Ok(results)
    }
}
