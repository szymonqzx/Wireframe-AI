//! Advanced orchestration patterns for Wireframe AI modules.
//!
//! Provides reusable patterns for task distribution, aggregation,
//! and dynamic planning in multi-agent workflows.

use crate::envelope::Envelope;
use crate::message_types::{AgentJob, AgentResult, TaskComplete, TaskEnriched};
use serde_json::Value;
use std::collections::HashMap;

/// Fan-out pattern: split one task into N parallel jobs.
pub fn fan_out(enriched: &TaskEnriched, sub_tasks: Vec<String>) -> Vec<Envelope<AgentJob>> {
    let mut jobs = Vec::with_capacity(sub_tasks.len());
    let session_id = Some(enriched.session_id.clone());

    let metadata = crate::message_types::JobMetadata {
        submitter: "orchestrator".to_string(),
        priority: 5,
        tags: [("workflow".to_string(), "fan_out".to_string())].into(),
    };
    let correlation_parent = enriched.correlation_id.clone();
    let context = enriched.context.clone();

    for (idx, sub_task) in sub_tasks.into_iter().enumerate() {
        let job = AgentJob {
            job_id: format!("{}-{}", correlation_parent, idx),
            correlation_parent: correlation_parent.clone(),
            task: crate::message_types::TaskDescription {
                user_input: sub_task.clone(),
                sub_task: Some(crate::message_types::SubTask {
                    title: format!("subtask-{}", idx),
                    description: sub_task,
                    expected_artifacts: vec![],
                }),
                output_format: None,
                user_constraints: vec![],
            },
            context: context.clone(),
            available_tool_capabilities: vec![],
            constraints: Default::default(),
            model_config: Default::default(),
            metadata: metadata.clone(),
            adapter_hints: None,
            schema_version: 1,
        };
        jobs.push(Envelope::new("agent.job", job, session_id.clone()));
    }
    jobs
}

/// Fan-in pattern: aggregate N agent results into a single TaskComplete.
pub fn fan_in(
    session_id: &str,
    correlation_id: &str,
    results: &[AgentResult],
) -> Envelope<TaskComplete> {
    let mut outputs = Vec::new();
    let mut all_side_effects = Vec::new();
    let mut warnings = Vec::new();

    for result in results {
        outputs.push(result.output.text.clone().unwrap_or_default());
        for effect in &result.output.files_written {
            all_side_effects.push(crate::message_types::SideEffect {
                kind: "file_written".to_string(),
                description: effect.to_string_lossy().to_string(),
                path: Some(effect.clone()),
            });
        }
        for err in &result.errors {
            warnings.push(format!("[{}] {}", err.code, err.message));
        }
    }

    let combined_result = outputs.join("\n\n---\n\n");

    let complete = TaskComplete {
        session_id: session_id.to_string(),
        correlation_id: correlation_id.to_string(),
        result: combined_result,
        side_effects: all_side_effects,
        warnings,
        completed_at: chrono::Utc::now().timestamp(),
    };

    Envelope::new("task.complete", complete, Some(session_id.to_string()))
}

/// Map-reduce pattern: map tasks, then reduce results.
#[allow(clippy::type_complexity)]
pub struct MapReduce<T> {
    mapper: Box<dyn Fn(&TaskEnriched) -> Vec<String> + Send + Sync>,
    reducer: Box<dyn Fn(&[AgentResult]) -> T + Send + Sync>,
}

impl<T> MapReduce<T> {
    pub fn new(
        mapper: impl Fn(&TaskEnriched) -> Vec<String> + Send + Sync + 'static,
        reducer: impl Fn(&[AgentResult]) -> T + Send + Sync + 'static,
    ) -> Self {
        Self {
            mapper: Box::new(mapper),
            reducer: Box::new(reducer),
        }
    }

    pub fn map(&self, enriched: &TaskEnriched) -> Vec<String> {
        (self.mapper)(enriched)
    }

    pub fn reduce(&self, results: &[AgentResult]) -> T {
        (self.reducer)(results)
    }
}

/// Trait for pluggable task planners.
///
/// Implement this trait to integrate an LLM-based planner, a rule-based planner,
/// or any other planning strategy. The SDK provides a default heuristic implementation
/// suitable for demos and fallback behavior.
pub trait Planner: Send + Sync {
    /// Analyze a task and generate a plan.
    fn plan(&self, task: &str) -> Plan;
}

/// Dynamic planner: generates sub-tasks based on task analysis.
///
/// By default this uses a lightweight heuristic analyzer. In production,
/// replace it with an LLM-backed implementation of the `Planner` trait.
pub struct DynamicPlanner {
    pub max_depth: u32,
    pub max_branching: u32,
}

impl DynamicPlanner {
    pub fn new(max_depth: u32, max_branching: u32) -> Self {
        Self {
            max_depth,
            max_branching,
        }
    }
}

impl Planner for DynamicPlanner {
    fn plan(&self, task: &str) -> Plan {
        let mut steps = heuristic_plan(task, self.max_branching);
        // Enforce depth limit by truncating steps that exceed max_depth tiers.
        let depth_limited: Vec<PlanStep> = steps
            .into_iter()
            .enumerate()
            .filter(|(i, _)| (*i as u32) < self.max_depth * self.max_branching)
            .map(|(_, s)| s)
            .collect();
        steps = depth_limited;
        Plan {
            steps,
            depth: self.max_depth.min(1),
            estimated_tokens: task.len() * 2,
        }
    }
}

/// A planned sequence of steps.
pub struct Plan {
    pub steps: Vec<PlanStep>,
    pub depth: u32,
    pub estimated_tokens: usize,
}

/// A single step in a plan.
pub struct PlanStep {
    pub id: String,
    pub description: String,
    pub dependencies: Vec<String>,
    pub estimated_cost_cents: u64,
}

/// State machine for workflow execution.
pub struct WorkflowStateMachine {
    states: HashMap<String, WorkflowState>,
    transitions: Vec<StateTransition>,
}

#[derive(Clone, Debug)]
pub enum WorkflowState {
    Pending,
    Running,
    WaitingForDependency(String),
    Completed(Value),
    Failed(String),
}

#[derive(Clone, Debug)]
pub struct StateTransition {
    pub from: String,
    pub to: String,
    pub condition: TransitionCondition,
}

#[derive(Clone, Debug)]
pub enum TransitionCondition {
    Always,
    OnSuccess,
    OnFailure,
    Custom(String),
}

impl Default for WorkflowStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowStateMachine {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            transitions: vec![],
        }
    }

    pub fn add_state(&mut self, id: impl Into<String>, state: WorkflowState) {
        self.states.insert(id.into(), state);
    }

    pub fn add_transition(&mut self, transition: StateTransition) {
        self.transitions.push(transition);
    }

    pub fn get_state(&self, id: &str) -> Option<&WorkflowState> {
        self.states.get(id)
    }

    pub fn can_transition(&self, from: &str, to: &str) -> bool {
        self.transitions
            .iter()
            .any(|t| t.from == from && t.to == to)
    }
}

// Heuristic planner: simple keyword-based task decomposition.
fn heuristic_plan(task: &str, max_steps: u32) -> Vec<PlanStep> {
    let keywords = task.to_lowercase();
    let mut steps = Vec::new();

    if keywords.contains("write") || keywords.contains("create") || keywords.contains("generate") {
        steps.push(PlanStep {
            id: "draft".to_string(),
            description: "Create initial draft".to_string(),
            dependencies: vec![],
            estimated_cost_cents: 10,
        });
        steps.push(PlanStep {
            id: "review".to_string(),
            description: "Review and refine output".to_string(),
            dependencies: vec!["draft".to_string()],
            estimated_cost_cents: 5,
        });
    }

    if keywords.contains("analyze")
        || keywords.contains("research")
        || keywords.contains("investigate")
    {
        steps.push(PlanStep {
            id: "gather".to_string(),
            description: "Gather relevant information".to_string(),
            dependencies: vec![],
            estimated_cost_cents: 8,
        });
        steps.push(PlanStep {
            id: "synthesize".to_string(),
            description: "Synthesize findings".to_string(),
            dependencies: vec!["gather".to_string()],
            estimated_cost_cents: 6,
        });
    }

    if keywords.contains("compare") || keywords.contains("versus") || keywords.contains("vs") {
        steps.push(PlanStep {
            id: "evaluate_a".to_string(),
            description: "Evaluate option A".to_string(),
            dependencies: vec![],
            estimated_cost_cents: 5,
        });
        steps.push(PlanStep {
            id: "evaluate_b".to_string(),
            description: "Evaluate option B".to_string(),
            dependencies: vec![],
            estimated_cost_cents: 5,
        });
        steps.push(PlanStep {
            id: "compare".to_string(),
            description: "Compare and summarize".to_string(),
            dependencies: vec!["evaluate_a".to_string(), "evaluate_b".to_string()],
            estimated_cost_cents: 4,
        });
    }

    if steps.is_empty() {
        steps.push(PlanStep {
            id: "execute".to_string(),
            description: task.to_string(),
            dependencies: vec![],
            estimated_cost_cents: 10,
        });
    }

    steps.truncate(max_steps as usize);
    steps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fan_out_creates_correct_job_count() {
        let enriched = TaskEnriched {
            session_id: "s1".to_string(),
            correlation_id: "c1".to_string(),
            user_input: "test".to_string(),
            context: Default::default(),
            inferred_constraints: vec![],
            enriched_at: 0,
        };
        let jobs = fan_out(
            &enriched,
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
        );
        assert_eq!(jobs.len(), 3);
    }

    #[test]
    fn test_fan_in_combines_results() {
        let results = vec![
            AgentResult {
                job_id: "j1".to_string(),
                correlation_parent: "c1".to_string(),
                output: crate::message_types::AgentOutput {
                    text: Some("Part 1".to_string()),
                    structured: None,
                    files_written: vec![],
                    commands_run: vec![],
                },
                tool_invocations: vec![],
                errors: vec![],
                usage: None,
                completed_at: 0,
            },
            AgentResult {
                job_id: "j2".to_string(),
                correlation_parent: "c1".to_string(),
                output: crate::message_types::AgentOutput {
                    text: Some("Part 2".to_string()),
                    structured: None,
                    files_written: vec![],
                    commands_run: vec![],
                },
                tool_invocations: vec![],
                errors: vec![],
                usage: None,
                completed_at: 0,
            },
        ];
        let complete = fan_in("s1", "c1", &results);
        assert!(complete.payload.result.contains("Part 1"));
        assert!(complete.payload.result.contains("Part 2"));
    }

    #[test]
    fn test_heuristic_plan_write() {
        let plan = heuristic_plan("Write a report about AI", 10);
        assert!(!plan.is_empty());
        assert!(plan.iter().any(|s| s.id == "draft"));
    }

    #[test]
    fn test_state_machine_transition() {
        let mut sm = WorkflowStateMachine::new();
        sm.add_state("a", WorkflowState::Pending);
        sm.add_state("b", WorkflowState::Pending);
        sm.add_transition(StateTransition {
            from: "a".to_string(),
            to: "b".to_string(),
            condition: TransitionCondition::OnSuccess,
        });
        assert!(sm.can_transition("a", "b"));
        assert!(!sm.can_transition("b", "a"));
    }
}
