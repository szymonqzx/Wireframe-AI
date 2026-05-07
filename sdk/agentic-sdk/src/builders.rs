//! Type-safe message builders for constructing Wireframe AI envelopes and payloads.
//!
//! These builders ensure required fields are set and provide fluent APIs for
//! common message construction patterns.
//!
//! ## Example
//!
//! ```ignore
//! use agentic_sdk::builders::*;
//!
//! let envelope = TaskSubmittedBuilder::new()
//!     .session_id("session_abc123")
//!     .user_input("Hello, world!")
//!     .build_envelope();
//! ```

use crate::envelope::Envelope;
use crate::message_types::*;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// TaskSubmitted Builder
// ============================================================================

/// Builder for [`TaskSubmitted`] messages.
#[derive(Debug, Default)]
pub struct TaskSubmittedBuilder {
    session_id: Option<String>,
    user_input: Option<String>,
    submitted_at: Option<i64>,
}

impl TaskSubmittedBuilder {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = Some(id.into());
        self
    }

    #[inline]
    pub fn user_input(mut self, input: impl Into<String>) -> Self {
        self.user_input = Some(input.into());
        self
    }

    #[inline]
    pub fn submitted_at(mut self, ts: i64) -> Self {
        self.submitted_at = Some(ts);
        self
    }

    /// Build the payload.
    pub fn build(self) -> Result<TaskSubmitted, BuilderError> {
        Ok(TaskSubmitted {
            session_id: self.session_id.ok_or(BuilderError::missing("session_id"))?,
            user_input: self.user_input.ok_or(BuilderError::missing("user_input"))?,
            submitted_at: self
                .submitted_at
                .unwrap_or_else(|| chrono::Utc::now().timestamp()),
        })
    }

    /// Build and wrap in an [`Envelope`].
    pub fn build_envelope(self) -> Result<Envelope<TaskSubmitted>, BuilderError> {
        let payload = self.build()?;
        let session_id = payload.session_id.clone();
        Ok(Envelope::new("task.submitted", payload, Some(session_id)))
    }
}

// ============================================================================
// TaskComplete Builder
// ============================================================================

/// Builder for [`TaskComplete`] messages.
#[derive(Debug, Default)]
pub struct TaskCompleteBuilder {
    session_id: Option<String>,
    correlation_id: Option<String>,
    result: Option<String>,
    side_effects: Vec<SideEffect>,
    warnings: Vec<String>,
    completed_at: Option<i64>,
}

impl TaskCompleteBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = Some(id.into());
        self
    }

    pub fn correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    pub fn result(mut self, text: impl Into<String>) -> Self {
        self.result = Some(text.into());
        self
    }

    pub fn side_effect(mut self, kind: impl Into<String>, description: impl Into<String>) -> Self {
        self.side_effects.push(SideEffect {
            kind: kind.into(),
            description: description.into(),
            path: None,
        });
        self
    }

    pub fn side_effect_with_path(
        mut self,
        kind: impl Into<String>,
        description: impl Into<String>,
        path: impl Into<PathBuf>,
    ) -> Self {
        self.side_effects.push(SideEffect {
            kind: kind.into(),
            description: description.into(),
            path: Some(path.into()),
        });
        self
    }

    pub fn warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    pub fn completed_at(mut self, ts: i64) -> Self {
        self.completed_at = Some(ts);
        self
    }

    pub fn build(self) -> Result<TaskComplete, BuilderError> {
        Ok(TaskComplete {
            session_id: self.session_id.ok_or(BuilderError::missing("session_id"))?,
            correlation_id: self
                .correlation_id
                .ok_or(BuilderError::missing("correlation_id"))?,
            result: self.result.ok_or(BuilderError::missing("result"))?,
            side_effects: self.side_effects,
            warnings: self.warnings,
            completed_at: self
                .completed_at
                .unwrap_or_else(|| chrono::Utc::now().timestamp()),
        })
    }

    pub fn build_envelope(self) -> Result<Envelope<TaskComplete>, BuilderError> {
        let payload = self.build()?;
        let session_id = payload.session_id.clone();
        Ok(Envelope::new("task.complete", payload, Some(session_id)))
    }
}

// ============================================================================
// AgentJob Builder
// ============================================================================

/// Builder for [`AgentJob`] messages.
#[derive(Debug, Default)]
pub struct AgentJobBuilder {
    job_id: Option<String>,
    correlation_parent: Option<String>,
    task: Option<TaskDescription>,
    context: Option<ContextPackage>,
    available_tool_capabilities: Vec<ToolCapability>,
    constraints: Option<ExecutionConstraints>,
    model_config: Option<ModelConfig>,
    metadata: Option<JobMetadata>,
    adapter_hints: Option<Value>,
}

impl AgentJobBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn job_id(mut self, id: impl Into<String>) -> Self {
        self.job_id = Some(id.into());
        self
    }

    pub fn correlation_parent(mut self, id: impl Into<String>) -> Self {
        self.correlation_parent = Some(id.into());
        self
    }

    pub fn task(mut self, task: TaskDescription) -> Self {
        self.task = Some(task);
        self
    }

    pub fn user_input(mut self, input: impl Into<String>) -> Self {
        let task = self.task.get_or_insert_with(|| TaskDescription {
            user_input: String::new(),
            sub_task: None,
            output_format: None,
            user_constraints: vec![],
        });
        task.user_input = input.into();
        self
    }

    pub fn context(mut self, ctx: ContextPackage) -> Self {
        self.context = Some(ctx);
        self
    }

    pub fn tool_capability(mut self, cap: ToolCapability) -> Self {
        self.available_tool_capabilities.push(cap);
        self
    }

    pub fn constraints(mut self, c: ExecutionConstraints) -> Self {
        self.constraints = Some(c);
        self
    }

    pub fn model_config(mut self, m: ModelConfig) -> Self {
        self.model_config = Some(m);
        self
    }

    pub fn metadata(mut self, m: JobMetadata) -> Self {
        self.metadata = Some(m);
        self
    }

    pub fn adapter_hints(mut self, hints: Value) -> Self {
        self.adapter_hints = Some(hints);
        self
    }

    pub fn build(self) -> Result<AgentJob, BuilderError> {
        Ok(AgentJob {
            job_id: self
                .job_id
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            correlation_parent: self
                .correlation_parent
                .ok_or(BuilderError::missing("correlation_parent"))?,
            task: self.task.ok_or(BuilderError::missing("task"))?,
            context: self.context.unwrap_or_else(|| ContextPackage {
                memory_chunks: vec![],
                session_history: vec![],
                readonly_files: vec![],
                safe_env: HashMap::new(),
                working_dir: PathBuf::from("."),
                max_context_tokens: 32768,
            }),
            available_tool_capabilities: self.available_tool_capabilities,
            constraints: self.constraints.unwrap_or_default(),
            model_config: self.model_config.unwrap_or_default(),
            metadata: self.metadata.unwrap_or_default(),
            adapter_hints: self.adapter_hints,
            schema_version: 1,
        })
    }

    pub fn build_envelope(self) -> Result<Envelope<AgentJob>, BuilderError> {
        let payload = self.build()?;
        Ok(Envelope::new("agent.job", payload, None))
    }
}

// ============================================================================
// AgentResult Builder
// ============================================================================

/// Builder for [`AgentResult`] messages.
#[derive(Debug, Default)]
pub struct AgentResultBuilder {
    job_id: Option<String>,
    correlation_parent: Option<String>,
    output_text: Option<String>,
    output_structured: Option<Value>,
    files_written: Vec<PathBuf>,
    commands_run: Vec<String>,
    tool_invocations: Vec<ToolInvocation>,
    errors: Vec<AdapterError>,
    usage: Option<UsageMetrics>,
}

impl AgentResultBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn job_id(mut self, id: impl Into<String>) -> Self {
        self.job_id = Some(id.into());
        self
    }

    pub fn correlation_parent(mut self, id: impl Into<String>) -> Self {
        self.correlation_parent = Some(id.into());
        self
    }

    pub fn output_text(mut self, text: impl Into<String>) -> Self {
        self.output_text = Some(text.into());
        self
    }

    pub fn output_structured(mut self, value: Value) -> Self {
        self.output_structured = Some(value);
        self
    }

    pub fn file_written(mut self, path: impl Into<PathBuf>) -> Self {
        self.files_written.push(path.into());
        self
    }

    pub fn command_run(mut self, cmd: impl Into<String>) -> Self {
        self.commands_run.push(cmd.into());
        self
    }

    pub fn tool_invocation(mut self, invocation: ToolInvocation) -> Self {
        self.tool_invocations.push(invocation);
        self
    }

    pub fn error(
        mut self,
        code: impl Into<String>,
        message: impl Into<String>,
        retryable: bool,
    ) -> Self {
        self.errors.push(AdapterError {
            code: code.into(),
            message: message.into(),
            retryable,
        });
        self
    }

    pub fn usage(mut self, u: UsageMetrics) -> Self {
        self.usage = Some(u);
        self
    }

    pub fn build(self) -> Result<AgentResult, BuilderError> {
        Ok(AgentResult {
            job_id: self.job_id.ok_or(BuilderError::missing("job_id"))?,
            correlation_parent: self
                .correlation_parent
                .ok_or(BuilderError::missing("correlation_parent"))?,
            output: AgentOutput {
                text: self.output_text,
                structured: self.output_structured,
                files_written: self.files_written,
                commands_run: self.commands_run,
            },
            tool_invocations: self.tool_invocations,
            errors: self.errors,
            usage: self.usage,
            completed_at: chrono::Utc::now().timestamp(),
        })
    }

    pub fn build_envelope(self) -> Result<Envelope<AgentResult>, BuilderError> {
        let payload = self.build()?;
        Ok(Envelope::new("agent.result", payload, None))
    }
}

// ============================================================================
// ContextPackage Builder
// ============================================================================

/// Builder for [`ContextPackage`] messages.
#[derive(Debug, Default)]
pub struct ContextPackageBuilder {
    memory_chunks: Vec<MemoryChunk>,
    session_history: Vec<ChatMessage>,
    readonly_files: Vec<FileSnapshot>,
    safe_env: HashMap<String, String>,
    working_dir: Option<PathBuf>,
    max_context_tokens: usize,
}

impl ContextPackageBuilder {
    pub fn new() -> Self {
        Self {
            max_context_tokens: 32768,
            ..Default::default()
        }
    }

    pub fn memory_chunk(mut self, chunk: MemoryChunk) -> Self {
        self.memory_chunks.push(chunk);
        self
    }

    pub fn session_message(mut self, role: impl Into<String>, content: impl Into<String>) -> Self {
        self.session_history.push(ChatMessage {
            role: role.into(),
            content: content.into(),
            timestamp: chrono::Utc::now().timestamp(),
        });
        self
    }

    pub fn readonly_file(mut self, snapshot: FileSnapshot) -> Self {
        self.readonly_files.push(snapshot);
        self
    }

    pub fn env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.safe_env.insert(key.into(), value.into());
        self
    }

    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    pub fn max_context_tokens(mut self, tokens: usize) -> Self {
        self.max_context_tokens = tokens;
        self
    }

    pub fn build(self) -> ContextPackage {
        ContextPackage {
            memory_chunks: self.memory_chunks,
            session_history: self.session_history,
            readonly_files: self.readonly_files,
            safe_env: self.safe_env,
            working_dir: self.working_dir.unwrap_or_else(|| PathBuf::from(".")),
            max_context_tokens: self.max_context_tokens,
        }
    }
}

// ============================================================================
// ModelConfig Builder
// ============================================================================

/// Builder for [`ModelConfig`] messages.
#[derive(Debug, Default)]
pub struct ModelConfigBuilder {
    provider: Option<String>,
    model_name: Option<String>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    extra: HashMap<String, Value>,
}

impl ModelConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn provider(mut self, p: impl Into<String>) -> Self {
        self.provider = Some(p.into());
        self
    }

    pub fn model_name(mut self, m: impl Into<String>) -> Self {
        self.model_name = Some(m.into());
        self
    }

    pub fn temperature(mut self, t: f32) -> Self {
        self.temperature = Some(t);
        self
    }

    pub fn top_p(mut self, p: f32) -> Self {
        self.top_p = Some(p);
        self
    }

    pub fn extra_param(mut self, key: impl Into<String>, value: Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }

    pub fn build(self) -> Result<ModelConfig, BuilderError> {
        Ok(ModelConfig {
            provider: self.provider.ok_or(BuilderError::missing("provider"))?,
            model_name: self.model_name.ok_or(BuilderError::missing("model_name"))?,
            temperature: self.temperature,
            top_p: self.top_p,
            extra: self.extra,
        })
    }
}

// ============================================================================
// ExecutionConstraints Builder
// ============================================================================

/// Builder for [`ExecutionConstraints`] messages.
#[derive(Debug, Default)]
pub struct ExecutionConstraintsBuilder {
    timeout_seconds: Option<u32>,
    max_completion_tokens: Option<usize>,
    network_access: NetworkPolicy,
    filesystem_policy: FilesystemPolicy,
    allow_subprocess: bool,
    execution_mode: Option<String>,
}

impl ExecutionConstraintsBuilder {
    pub fn new() -> Self {
        Self {
            network_access: NetworkPolicy::OutboundOnly,
            filesystem_policy: FilesystemPolicy::SandboxWritable,
            allow_subprocess: true,
            ..Default::default()
        }
    }

    pub fn timeout_seconds(mut self, secs: u32) -> Self {
        self.timeout_seconds = Some(secs);
        self
    }

    pub fn max_completion_tokens(mut self, tokens: usize) -> Self {
        self.max_completion_tokens = Some(tokens);
        self
    }

    pub fn network_access(mut self, policy: NetworkPolicy) -> Self {
        self.network_access = policy;
        self
    }

    pub fn filesystem_policy(mut self, policy: FilesystemPolicy) -> Self {
        self.filesystem_policy = policy;
        self
    }

    pub fn allow_subprocess(mut self, allow: bool) -> Self {
        self.allow_subprocess = allow;
        self
    }

    pub fn execution_mode(mut self, mode: impl Into<String>) -> Self {
        self.execution_mode = Some(mode.into());
        self
    }

    pub fn build(self) -> ExecutionConstraints {
        ExecutionConstraints {
            timeout_seconds: self.timeout_seconds,
            max_completion_tokens: self.max_completion_tokens,
            network_access: self.network_access,
            filesystem_policy: self.filesystem_policy,
            allow_subprocess: self.allow_subprocess,
            execution_mode: self.execution_mode,
        }
    }
}

// ============================================================================
// BuilderError
// ============================================================================

/// Error when a required field is missing during builder construction.
#[derive(Debug, Clone, thiserror::Error)]
pub enum BuilderError {
    #[error("missing required field: {0}")]
    MissingField(String),
}

impl BuilderError {
    pub fn missing(field: impl Into<String>) -> Self {
        BuilderError::MissingField(field.into())
    }
}

// ============================================================================
// Envelope convenience constructors
// ============================================================================

/// Convenience functions for constructing common envelope types.
pub mod envelope_helpers {
    use super::*;

    /// Create a `sys.module.online` envelope.
    #[inline]
    pub fn module_online(
        module_id: impl Into<String>,
        version: impl Into<String>,
        subscribes: Vec<String>,
        publishes: Vec<String>,
    ) -> Envelope<Value> {
        let payload = serde_json::json!({
            "module_id": module_id.into(),
            "version": version.into(),
            "subscribes": subscribes,
            "publishes": publishes,
        });
        Envelope::new("sys.module.online", payload, None)
    }

    /// Create a `sys.module.offline` envelope.
    #[inline]
    pub fn module_offline(
        module_id: impl Into<String>,
        version: impl Into<String>,
    ) -> Envelope<Value> {
        let payload = serde_json::json!({
            "module_id": module_id.into(),
            "version": version.into(),
        });
        Envelope::new("sys.module.offline", payload, None)
    }

    /// Create a `sys.module.error` envelope.
    #[inline]
    pub fn module_error(
        module_id: impl Into<String>,
        error_code: impl Into<String>,
        error_message: impl Into<String>,
    ) -> Envelope<Value> {
        let payload = serde_json::json!({
            "module_id": module_id.into(),
            "error_code": error_code.into(),
            "error_message": error_message.into(),
            "ts": chrono::Utc::now().timestamp(),
        });
        Envelope::new("sys.module.error", payload, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_submitted_builder() {
        let envelope = TaskSubmittedBuilder::new()
            .session_id("session_abc")
            .user_input("Hello")
            .build_envelope()
            .unwrap();

        assert_eq!(envelope.payload.session_id, "session_abc");
        assert_eq!(envelope.payload.user_input, "Hello");
        assert_eq!(envelope.topic, "task.submitted");
    }

    #[test]
    fn test_task_submitted_builder_missing_field() {
        let result = TaskSubmittedBuilder::new().user_input("Hello").build();
        assert!(result.is_err());
    }

    #[test]
    fn test_task_complete_builder() {
        let envelope = TaskCompleteBuilder::new()
            .session_id("session_abc")
            .correlation_id("corr_123")
            .result("Done!")
            .side_effect("file_written", "Created output.txt")
            .warning("Partial result")
            .build_envelope()
            .unwrap();

        assert_eq!(envelope.payload.result, "Done!");
        assert_eq!(envelope.payload.side_effects.len(), 1);
        assert_eq!(envelope.payload.warnings.len(), 1);
    }

    #[test]
    fn test_agent_job_builder() {
        let job = AgentJobBuilder::new()
            .correlation_parent("parent_123")
            .user_input("Do something")
            .build()
            .unwrap();

        assert_eq!(job.correlation_parent, "parent_123");
        assert_eq!(job.task.user_input, "Do something");
        assert!(!job.job_id.is_empty());
    }

    #[test]
    fn test_context_package_builder() {
        let ctx = ContextPackageBuilder::new()
            .session_message("user", "Hello")
            .session_message("assistant", "Hi there")
            .env_var("KEY", "value")
            .max_context_tokens(16000)
            .build();

        assert_eq!(ctx.session_history.len(), 2);
        assert_eq!(ctx.safe_env.get("KEY"), Some(&"value".to_string()));
        assert_eq!(ctx.max_context_tokens, 16000);
    }

    #[test]
    fn test_model_config_builder() {
        let config = ModelConfigBuilder::new()
            .provider("openai")
            .model_name("gpt-4o")
            .temperature(0.5)
            .build()
            .unwrap();

        assert_eq!(config.provider, "openai");
        assert_eq!(config.model_name, "gpt-4o");
        assert_eq!(config.temperature, Some(0.5));
    }

    #[test]
    fn test_execution_constraints_builder() {
        let constraints = ExecutionConstraintsBuilder::new()
            .timeout_seconds(60)
            .network_access(NetworkPolicy::None)
            .allow_subprocess(false)
            .build();

        assert_eq!(constraints.timeout_seconds, Some(60));
        assert_eq!(constraints.network_access, NetworkPolicy::None);
        assert!(!constraints.allow_subprocess);
    }
}
