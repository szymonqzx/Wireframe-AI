//! All message payload types for the distributed agent system.
//! These structs are the actual data carried inside Envelope<T>.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// Task Flow: submitted → enriched → complete
// ============================================================================

/// User request as submitted by the Interface module.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaskSubmitted {
    pub session_id: String,
    pub user_input: String,
    pub submitted_at: i64,
}

/// Task after Context module enrichment — now carries memory and session history.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaskEnriched {
    pub session_id: String,
    pub correlation_id: String,
    /// Original user request (preserved)
    pub user_input: String,
    /// Enriched context package (memory, history, files)
    pub context: ContextPackage,
    /// Any constraints discovered/ inferred
    pub inferred_constraints: Vec<String>,
    pub enriched_at: i64,
}

/// Final result returned to Interface for display to user.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaskComplete {
    pub session_id: String,
    pub correlation_id: String,
    /// The final answer / output
    pub result: String,
    /// Optional side effects (files written, commands run)
    pub side_effects: Vec<SideEffect>,
    /// Any errors encountered (non-fatal)
    pub warnings: Vec<String>,
    pub completed_at: i64,
}

// ============================================================================
// Agent Job & Result
// ============================================================================

/// Self-contained unit of work dispatched to a Reasoning Adapter.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AgentJob {
    /// Unique ID for this job
    pub job_id: String,
    /// Parent correlation ID — routes result back to Orchestrator
    pub correlation_parent: String,
    /// What to do
    pub task: TaskDescription,
    /// All context needed (no external queries allowed)
    pub context: ContextPackage,
    /// Which MCP tools the adapter may invoke
    pub available_tool_capabilities: Vec<ToolCapability>,
    /// Execution constraints (timeout, network, filesystem)
    pub constraints: ExecutionConstraints,
    /// Model to use
    pub model_config: ModelConfig,
    /// Observability metadata
    pub metadata: JobMetadata,
    /// Adapter-specific hints (optional, e.g., chain config, index IDs).
    /// Passed through verbatim — adapters ignore unknown fields.
    pub adapter_hints: Option<serde_json::Value>,
    /// Schema version (default 1)
    #[serde(default = "current_schema_version")]
    pub schema_version: u32,
}

/// What an adapter sends back after finishing a job.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AgentResult {
    /// The job this completes
    pub job_id: String,
    /// Root correlation ID (echoed from AgentJob.correlation_parent)
    pub correlation_parent: String,
    /// What the adapter produced
    pub output: AgentOutput,
    /// Tool calls made (if any)
    pub tool_invocations: Vec<ToolInvocation>,
    /// Errors that occurred during execution (non-fatal)
    pub errors: Vec<AdapterError>,
    /// Token usage / cost metrics
    pub usage: Option<UsageMetrics>,
    pub completed_at: i64,
}

// ============================================================================
// Execution (Sandbox)
// ============================================================================

/// Request to execute something in the sandbox.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExecRequest {
    pub request_id: String,
    pub correlation_id: String,
    pub tool: String,
    pub parameters: serde_json::Value,
    pub timeout_seconds: Option<u32>,
}

/// Result of a sandbox execution.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExecResult {
    pub request_id: String,
    pub correlation_id: String,
    pub success: bool,
    pub output: serde_json::Value,
    pub stderr: Option<String>,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
}

// ============================================================================
// Sub-structures
// ============================================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TaskDescription {
    pub user_input: String,
    pub sub_task: Option<SubTask>,
    pub output_format: Option<OutputFormat>,
    pub user_constraints: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubTask {
    /// Short title
    pub title: String,
    /// Detailed instructions
    pub description: String,
    /// Expected artifact types (e.g. ["file", "code", "sql"])
    pub expected_artifacts: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OutputFormat {
    /// e.g. "markdown", "json", "plain text"
    pub format: String,
    /// Optional template string
    pub template: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ContextPackage {
    pub memory_chunks: Vec<MemoryChunk>,
    pub session_history: Vec<ChatMessage>,
    pub readonly_files: Vec<FileSnapshot>,
    pub safe_env: HashMap<String, String>,
    pub working_dir: PathBuf,
    pub max_context_tokens: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MemoryChunk {
    pub id: String,
    pub content: String,
    pub source: String,
    pub relevance_score: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub role: String, // "user", "assistant", "system"
    pub content: String,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileSnapshot {
    pub path: PathBuf,
    pub content: String,
    pub size_bytes: usize,
    pub last_modified: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToolCapability {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub required_credentials: Vec<CredentialRef>,
    pub rate_limit: Option<RateLimit>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CredentialRef {
    /// Opaque reference to a credential managed by the orchestrator
    pub credential_id: String,
    /// What scope it grants (e.g., "file_read", "http_outbound")
    pub scope: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub burst: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExecutionConstraints {
    pub timeout_seconds: Option<u32>,
    pub max_completion_tokens: Option<usize>,
    pub network_access: NetworkPolicy,
    pub filesystem_policy: FilesystemPolicy,
    pub allow_subprocess: bool,
    pub execution_mode: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "PascalCase")]
pub enum NetworkPolicy {
    #[default]
    None,
    OutboundOnly,
    Full,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "PascalCase")]
pub enum FilesystemPolicy {
    #[default]
    Readonly,
    SandboxWritable,
    IsolatedVM,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModelConfig {
    pub provider: String,
    pub model_name: String,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobMetadata {
    pub submitter: String,
    #[serde(default = "default_priority")]
    pub priority: u8,
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AgentOutput {
    /// Final text result
    pub text: Option<String>,
    /// Structured output (JSON, if requested)
    pub structured: Option<serde_json::Value>,
    /// Files written (relative to working_dir)
    pub files_written: Vec<PathBuf>,
    /// Commands executed
    pub commands_run: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToolInvocation {
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub result: serde_json::Value,
    pub duration_ms: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AdapterError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UsageMetrics {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    pub cost_cents: Option<f64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SideEffect {
    pub kind: String,
    pub description: String,
    pub path: Option<PathBuf>,
}

// Defaults

fn default_priority() -> u8 {
    1
}
fn current_schema_version() -> u32 {
    1
}

impl Default for ExecutionConstraints {
    fn default() -> Self {
        Self {
            timeout_seconds: Some(300),
            max_completion_tokens: Some(32768),
            network_access: NetworkPolicy::OutboundOnly,
            filesystem_policy: FilesystemPolicy::SandboxWritable,
            allow_subprocess: true,
            execution_mode: None,
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: "openai".into(),
            model_name: "gpt-4o".into(),
            temperature: Some(0.7),
            top_p: None,
            extra: HashMap::new(),
        }
    }
}

impl Default for JobMetadata {
    fn default() -> Self {
        Self {
            submitter: "system".into(),
            priority: 1,
            tags: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn agent_job_serializes() {
        let job = AgentJob {
            job_id: Uuid::new_v4().to_string(),
            correlation_parent: "root-correlation".into(),
            task: TaskDescription {
                user_input: "Hello".into(),
                sub_task: None,
                output_format: None,
                user_constraints: vec![],
            },
            context: ContextPackage {
                memory_chunks: vec![],
                session_history: vec![],
                readonly_files: vec![],
                safe_env: HashMap::new(),
                working_dir: PathBuf::from("/tmp"),
                max_context_tokens: 32768,
            },
            available_tool_capabilities: vec![],
            constraints: ExecutionConstraints::default(),
            model_config: ModelConfig::default(),
            metadata: JobMetadata::default(),
            adapter_hints: None,
            schema_version: 1,
        };

        let json = serde_json::to_string_pretty(&job).unwrap();
        assert!(json.contains("\"job_id\""));
        assert!(json.contains("\"correlation_parent\""));
        let deserialized: AgentJob = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.job_id, job.job_id);
        assert_eq!(deserialized.correlation_parent, "root-correlation");
    }

    #[test]
    fn execution_constraints_default() {
        let c = ExecutionConstraints::default();
        assert_eq!(c.timeout_seconds, Some(300));
        assert_eq!(c.network_access, NetworkPolicy::OutboundOnly);
    }
}
