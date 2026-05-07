//! Workflow Agent — Wireframe-AI Example
//!
//! Demonstrates a multi-step workflow:
//! 1. Receives a task
//! 2. Plans sub-tasks using DynamicPlanner
//! 3. Dispatches jobs via fan-out
//! 4. Collects results via fan-in
//! 5. Returns final synthesized output

use agentic_sdk::{
    builders::ContextPackageBuilder,
    envelope::Envelope,
    message_types::{AgentResult, TaskEnriched},
    orchestrator_patterns::{fan_in, fan_out, DynamicPlanner, Planner},
    Module,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

struct WorkflowAgent {
    pending_results: Arc<Mutex<HashMap<String, Vec<AgentResult>>>>,
    /// Maps correlation_id -> (session_id, expected_result_count).
    expected_counts: Arc<Mutex<HashMap<String, (String, usize)>>>,
    planner: DynamicPlanner,
}

#[agentic_sdk::module(
    subscribes = ["task.submitted", "agent.result"],
    publishes  = ["task.enriched", "agent.job", "task.complete"],
    queue_group = "workflow_agent"
)]
impl Module for WorkflowAgent {
    async fn handle(&mut self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        match env.topic.as_str() {
            "task.submitted" => self.handle_submitted(env).await,
            "agent.result" => self.handle_result(env).await,
            _ => vec![],
        }
    }
}

impl WorkflowAgent {
    async fn handle_submitted(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let submitted: agentic_sdk::message_types::TaskSubmitted =
            match serde_json::from_value(env.payload.clone()) {
                Ok(t) => t,
                Err(e) => {
                    tracing::error!(error = %e, "failed to parse task.submitted");
                    return vec![];
                }
            };

        tracing::info!(session = %submitted.session_id, "workflow received task");

        // Plan sub-tasks
        let plan = self.planner.plan(&submitted.user_input);
        let sub_tasks: Vec<String> = plan.steps.iter().map(|s| s.description.clone()).collect();

        tracing::info!(steps = sub_tasks.len(), "planned workflow steps");

        // Create enriched task
        let enriched = TaskEnriched {
            session_id: submitted.session_id.clone(),
            correlation_id: env.correlation_id.clone(),
            user_input: submitted.user_input.clone(),
            context: ContextPackageBuilder::new().build(),
            inferred_constraints: vec![],
            enriched_at: chrono::Utc::now().timestamp(),
        };

        // Track expected result count and initialize pending storage
        let expected_count = sub_tasks.len();
        let session_id = submitted.session_id.clone();
        {
            let mut pending = self.pending_results.lock().await;
            pending.insert(env.correlation_id.clone(), vec![]);
            let mut expected = self.expected_counts.lock().await;
            expected.insert(env.correlation_id.clone(), (session_id, expected_count));
        }

        // Fan out jobs
        let jobs = fan_out(&enriched, sub_tasks);

        let mut outputs: Vec<Envelope<Value>> = vec![Envelope::new(
            "task.enriched",
            serde_json::to_value(&enriched).unwrap_or_default(),
            Some(submitted.session_id),
        )];

        for job in jobs {
            outputs.push(Envelope::new(
                "agent.job",
                serde_json::to_value(&job.payload).unwrap_or_default(),
                Some(job.session_id),
            ));
        }

        outputs
    }

    async fn handle_result(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let result: AgentResult = match serde_json::from_value(env.payload.clone()) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(error = %e, "failed to parse agent.result");
                return vec![];
            }
        };

        let mut pending = self.pending_results.lock().await;
        let correlation_parent = result.correlation_parent.clone();

        let entry = pending.entry(correlation_parent.clone()).or_default();
        entry.push(result.clone());
        let received_count = entry.len();

        let mut expected = self.expected_counts.lock().await;
        let (session_id, expected_count) = expected
            .get(&correlation_parent)
            .cloned()
            .unwrap_or_else(|| (correlation_parent.clone(), 1));

        let should_complete = received_count >= expected_count;
        let results: Vec<AgentResult> = if should_complete {
            expected.remove(&correlation_parent);
            pending.remove(&correlation_parent).unwrap_or_default()
        } else {
            vec![]
        };
        drop(expected);
        drop(pending);

        if should_complete {
            tracing::info!(count = results.len(), session = %session_id, "workflow complete");

            let complete = fan_in(&session_id, &correlation_parent, &results);
            vec![Envelope::new(
                "task.complete",
                serde_json::to_value(&complete.payload).unwrap_or_default(),
                Some(complete.session_id),
            )]
        } else {
            vec![]
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let agent = WorkflowAgent {
        pending_results: Arc::new(Mutex::new(HashMap::new())),
        expected_counts: Arc::new(Mutex::new(HashMap::new())),
        planner: DynamicPlanner::new(3, 5),
    };

    agent.run("nats://localhost:4222").await
}
