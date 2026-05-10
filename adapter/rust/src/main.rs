//! Wireframe-AI Rust Adapter
//!
//! LLM-powered reasoning adapter that processes AgentJob messages via NATS.
//! Uses the Provider trait for LLM backends with capability negotiation.
//! Supports optional sandbox mode and selfdev capability.

use agentic_sdk::{
    module::announce_online_with_selfdev, AgentJob, AgentOutput, AgentResult, Envelope,
    ToolInvocation, UsageMetrics,
};
use anyhow::Result;
use chrono::Utc;
use futures::StreamExt;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use tracing::{error, info, warn};
use wireframe_adapter::mcp_client::McpStdioClient;
use wireframe_adapter::provider_config;
use wireframe_adapter::security::{sanitize_string, validate_path, validate_path_for_write};
use wireframe_adapter::selfdev::{compile_adapter, run_safety_checks};
use wireframe_adapter::tools::{
    build_tool_definitions, execute_shell, list_directory, read_file, write_file, ToolContext,
    ToolName,
};
use wireframe_adapter::utils::{estimate_tokens, extract_side_effects};
use wireframe_provider_core::discovery::ProviderDiscoveryRegistry;
use wireframe_provider_core::{Message, Provider, SessionManager, StreamEvent};

/// Selfdev tool execution module.
mod selfdev_tools {
    use super::*;
    use std::path::PathBuf;

    const SELFDEV_NOT_ENABLED_ERROR: &str = "Selfdev not enabled or source root not configured";
    const MISSING_PATH_ERROR: &str = "Missing 'path' parameter";
    const MISSING_PATH_OR_CONTENT_ERROR: &str = "Missing 'path' or 'content' parameter";

    pub async fn execute_selfdev_tool(
        name: &str,
        params: &serde_json::Value,
        state: &AdapterState,
    ) -> Option<serde_json::Value> {
        match name {
            "read_source" => Some(execute_read_source(params, state).await),
            "write_source" => Some(execute_write_source(params, state).await),
            "compile_self" => Some(execute_compile_self(params, state).await),
            "restart_self" => Some(execute_restart_self(params, state).await),
            "switch_module" => Some(execute_switch_module(params).await),
            _ => None,
        }
    }

    async fn execute_read_source(
        params: &serde_json::Value,
        state: &AdapterState,
    ) -> serde_json::Value {
        let source_root = match &state.source_root {
            Some(root) => root,
            None => return serde_json::json!({"error": SELFDEV_NOT_ENABLED_ERROR}),
        };

        let path = match params.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => return serde_json::json!({"error": MISSING_PATH_ERROR}),
        };

        let sanitized = sanitize_string(path);
        match validate_path(sanitized.as_ref(), Some(source_root)) {
            Ok(validated_path) => match tokio::fs::read_to_string(&validated_path).await {
                Ok(content) => serde_json::json!({
                    "content": content,
                    "path": validated_path.to_string_lossy().to_string()
                }),
                Err(e) => serde_json::json!({
                    "error": format!("Failed to read source file: {}", e)
                }),
            },
            Err(e) => {
                warn!("Path validation failed: {}", e);
                serde_json::json!({
                    "error": format!("Path validation failed: {}", e)
                })
            }
        }
    }

    async fn execute_write_source(
        params: &serde_json::Value,
        state: &AdapterState,
    ) -> serde_json::Value {
        let source_root = match &state.source_root {
            Some(root) => root,
            None => return serde_json::json!({"error": SELFDEV_NOT_ENABLED_ERROR}),
        };

        let (path, content) = match (
            params.get("path").and_then(|v| v.as_str()),
            params.get("content").and_then(|v| v.as_str()),
        ) {
            (Some(p), Some(c)) => (p, c),
            _ => return serde_json::json!({"error": MISSING_PATH_OR_CONTENT_ERROR}),
        };

        let sanitized_path = sanitize_string(path);
        let sanitized_content = sanitize_string(content);
        match validate_path_for_write(sanitized_path.as_ref(), Some(source_root)) {
            Ok(validated_path) => {
                match tokio::fs::write(&validated_path, sanitized_content.as_ref()).await {
                    Ok(_) => serde_json::json!({
                        "success": true,
                        "path": validated_path.to_string_lossy().to_string()
                    }),
                    Err(e) => serde_json::json!({
                        "error": format!("Failed to write source file: {}", e)
                    }),
                }
            }
            Err(e) => {
                warn!("Path validation failed: {}", e);
                serde_json::json!({
                    "error": format!("Path validation failed: {}", e)
                })
            }
        }
    }

    async fn ensure_safety_checks_passed(source_root: &PathBuf) -> Result<(), serde_json::Value> {
        match run_safety_checks(source_root).await {
            Ok(check_result) => {
                if !check_result.all_passed {
                    Err(serde_json::json!({
                        "success": false,
                        "error": "Safety checks failed. Operation aborted.",
                        "check_results": check_result.to_json()
                    }))
                } else {
                    info!("Safety checks passed");
                    Ok(())
                }
            }
            Err(e) => {
                warn!("Safety check execution failed: {}", e);
                Err(serde_json::json!({
                    "success": false,
                    "error": format!("Safety check execution failed: {}", e)
                }))
            }
        }
    }

    async fn execute_compile_self(
        params: &serde_json::Value,
        state: &AdapterState,
    ) -> serde_json::Value {
        let source_root = match &state.source_root {
            Some(root) => root,
            None => return serde_json::json!({"error": SELFDEV_NOT_ENABLED_ERROR}),
        };

        let run_checks = params
            .get("run_checks")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        info!(
            "Starting self-compilation in {} (run_checks: {})",
            source_root.display(),
            run_checks
        );

        if run_checks {
            if let Err(e) = ensure_safety_checks_passed(source_root).await {
                return e;
            }
        }

        match compile_adapter(source_root).await {
            Ok(result) => result,
            Err(e) => serde_json::json!({
                "error": format!("Compilation failed: {}", e)
            }),
        }
    }

    async fn execute_restart_self(
        params: &serde_json::Value,
        state: &AdapterState,
    ) -> serde_json::Value {
        let source_root = match &state.source_root {
            Some(root) => root,
            None => return serde_json::json!({"error": SELFDEV_NOT_ENABLED_ERROR}),
        };

        let auto_restart = params
            .get("auto_restart")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let skip_checks = params
            .get("skip_checks")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        info!(
            "Restarting self (auto_restart: {}, skip_checks: {})",
            auto_restart, skip_checks
        );

        if !auto_restart {
            return serde_json::json!({
                "success": true,
                "message": "To restart, run: .\\scripts\\process-manager.ps1 -Module wireframe-adapter-rust -AutoRestart"
            });
        }

        if !skip_checks {
            if let Err(e) = ensure_safety_checks_passed(source_root).await {
                return e;
            }
        } else {
            warn!("Safety checks skipped for restart — this is dangerous");
        }

        let compile_result = compile_adapter(source_root).await;

        match compile_result {
            Ok(result_json) => {
                if result_json
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    info!("Compilation successful, initiating restart");
                    serde_json::json!({
                        "success": true,
                        "message": "Compilation successful. To restart, run: .\\scripts\\process-manager.ps1 -Module wireframe-adapter-rust -AutoRestart",
                        "manual_restart": true
                    })
                } else {
                    result_json
                }
            }
            Err(e) => serde_json::json!({
                "error": format!("Compilation failed: {}", e)
            }),
        }
    }

    async fn execute_switch_module(params: &serde_json::Value) -> serde_json::Value {
        let new_module = match params.get("new_module").and_then(|v| v.as_str()) {
            Some(m) => m,
            None => return serde_json::json!({"error": "Missing 'new_module' parameter"}),
        };

        let force = params
            .get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        info!("Switching to module: {} (force: {})", new_module, force);

        let force_str = if force { "true" } else { "false" };
        serde_json::json!({
            "success": true,
            "message": format!("To switch to {}, run: .\\scripts\\process-manager.ps1 -Switch -OldModule wireframe-adapter-rust -NewModule {} -Force {}", new_module, new_module, force_str),
            "manual_switch": true,
            "new_module": new_module
        })
    }
}

/// Message building module.
mod messages {
    use super::*;

    pub fn build_message_history(job: &AgentJob) -> Vec<Message> {
        let memory_count = usize::from(!job.context.memory_chunks.is_empty());
        let capacity = 2 + job.context.session_history.len() + memory_count;
        let mut messages: Vec<Message> = Vec::with_capacity(capacity);

        // System prompt (use constant to avoid allocation)
        messages.push(Message {
            role: SYSTEM_ROLE.to_string(),
            content: SYSTEM_PROMPT.to_string(),
            tool_call_id: None,
        });

        // Add memory chunks as context (before session history)
        if !job.context.memory_chunks.is_empty() {
            // Pre-allocate capacity for memory text to avoid reallocations
            let memory_chunks = &job.context.memory_chunks;
            let estimated_size = memory_chunks
                .iter()
                .map(|c| c.source.len() + c.content.len() + 50)
                .sum::<usize>();
            let mut memory_text = String::with_capacity(estimated_size);

            for (i, chunk) in memory_chunks.iter().enumerate() {
                if i > 0 {
                    memory_text.push('\n');
                }
                memory_text.push_str(&format!(
                    "[{} (relevance: {:.2})]\n{}",
                    chunk.source,
                    chunk.relevance_score,
                    sanitize_string(&chunk.content)
                ));
            }

            messages.push(Message {
                role: SYSTEM_ROLE.to_string(),
                content: format!("Relevant context from previous sessions:\n{}", memory_text),
                tool_call_id: None,
            });
        }

        // Add session history
        for msg in &job.context.session_history {
            messages.push(Message {
                role: msg.role.clone(),
                content: sanitize_string(&msg.content).into_owned(),
                tool_call_id: None,
            });
        }

        // Add current task
        let user_input = sanitize_string(&job.task.user_input).into_owned();
        messages.push(Message {
            role: USER_ROLE.to_string(),
            content: user_input,
            tool_call_id: None,
        });

        messages
    }

    pub fn update_session(
        state: &Arc<AdapterState>,
        session_id: &str,
        user_input: String,
        final_text: String,
    ) {
        if let Some(mut session) = state.session_manager.get_session_mut(session_id) {
            session.add_message(Message {
                role: USER_ROLE.to_string(),
                content: user_input,
                tool_call_id: None,
            });
            session.add_message(Message {
                role: ASSISTANT_ROLE.to_string(),
                content: final_text,
                tool_call_id: None,
            });
        }
    }
}

/// Provider registry for managing multiple LLM providers.
type ProviderRegistry = HashMap<String, Arc<dyn Provider>>;

/// System prompt for the LLM.
const SYSTEM_PROMPT: &str = "You are a helpful AI assistant with access to a sandbox environment. You can execute shell commands, read/write files, and list directories. Use these capabilities to fulfill the user's request. When you have completed the task, provide a clear summary of what you did.";

/// NATS topic constants to avoid string allocations.
const AGENT_JOB_TOPIC: &str = "agent.job";
const AGENT_RESULT_TOPIC: &str = "agent.result";
const AGENT_WORKER_QUEUE: &str = "agent_worker";
const USER_ROLE: &str = "user";
const ASSISTANT_ROLE: &str = "assistant";
const SYSTEM_ROLE: &str = "system";

/// Get cached tool definitions (built once on first use).
fn get_tool_definitions() -> &'static Vec<wireframe_provider_core::ToolDefinition> {
    static TOOLS: OnceLock<Vec<wireframe_provider_core::ToolDefinition>> = OnceLock::new();
    TOOLS.get_or_init(build_tool_definitions)
}

/// Get cached module name.
fn get_module_name() -> &'static str {
    "wireframe-adapter-rust"
}

/// Get cached module version.
fn get_module_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Execution mode for the adapter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecutionMode {
    Sandbox,
    Direct,
    Hybrid,
}

impl ExecutionMode {
    fn from_str(s: &str) -> Self {
        // Optimize: avoid allocation by checking case-insensitively
        match s {
            s if s.eq_ignore_ascii_case("sandbox") => ExecutionMode::Sandbox,
            s if s.eq_ignore_ascii_case("direct") => ExecutionMode::Direct,
            s if s.eq_ignore_ascii_case("hybrid") => ExecutionMode::Hybrid,
            _ => ExecutionMode::Direct, // Default to direct for development
        }
    }
}

/// Adapter state.
struct AdapterState {
    providers: ProviderRegistry,
    session_manager: SessionManager,
    execution_mode: ExecutionMode,
    selfdev_enabled: bool,
    source_root: Option<PathBuf>,
    binary_path: Option<PathBuf>,
    allowed_base_dir: Option<PathBuf>,
    /// Lazy-initialized MCP stdio client for sandbox mode.
    sandbox_client: tokio::sync::Mutex<Option<McpStdioClient>>,
}

impl AdapterState {
    fn new() -> Self {
        // Determine execution mode from environment
        let execution_mode = env::var("WIREFRAME_AI_EXECUTION_MODE")
            .map(|s| ExecutionMode::from_str(&s))
            .unwrap_or(ExecutionMode::Direct); // Default to direct for development

        // Determine selfdev mode from environment or intent detection
        let selfdev_enabled = env::var("WIREFRAME_AI_SELFDEV")
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false);

        // Get source root for selfdev
        let source_root = if selfdev_enabled {
            env::var("WIREFRAME_AI_SOURCE_ROOT")
                .ok()
                .map(PathBuf::from)
                .or_else(|| {
                    // Default to current directory if not set
                    env::current_dir().ok()
                })
        } else {
            None
        };

        // Get binary path for selfdev
        let binary_path = if selfdev_enabled {
            env::current_exe().ok()
        } else {
            None
        };

        // Get allowed base directory for file operations
        let allowed_base_dir = env::var("WIREFRAME_AI_ALLOWED_BASE_DIR")
            .ok()
            .map(PathBuf::from)
            .or_else(|| env::current_dir().ok());

        // Load provider configuration
        let config_path_buf = env::var("WIREFRAME_AI_PROVIDER_CONFIG")
            .ok()
            .map(|s| PathBuf::from(s));
        let config_path = config_path_buf.as_deref();

        let config = provider_config::load_provider_config(config_path).unwrap_or_else(|e| {
            warn!("Failed to load provider config: {}, using defaults", e);
            provider_config::default_config_from_env()
        });

        // Build provider registry from configuration (once)
        let registry = Arc::new(
            provider_config::build_registry_from_config(&config).unwrap_or_else(|e| {
                warn!("Failed to build provider registry: {}, using fallback", e);
                ProviderDiscoveryRegistry::new()
            }),
        );

        // Build backward-compatible provider map from registry
        let mut providers: ProviderRegistry = HashMap::new();
        for name in registry.list() {
            if let Some(provider) = registry.get(&name) {
                providers.insert(name, provider);
            }
        }

        Self {
            providers,
            session_manager: SessionManager::new(),
            execution_mode,
            selfdev_enabled,
            source_root,
            binary_path,
            allowed_base_dir,
            sandbox_client: tokio::sync::Mutex::new(None),
        }
    }

    fn get_provider(&self, name: &str) -> Option<Arc<dyn Provider>> {
        self.providers.get(name).cloned()
    }

    fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// Detect if the user prompt suggests selfdev intent.
    fn detect_selfdev_intent(&self, user_input: &str) -> bool {
        static KEYWORDS: OnceLock<&[&str]> = OnceLock::new();
        let selfdev_keywords = KEYWORDS.get_or_init(|| {
            &[
                "edit my code",
                "modify myself",
                "improve my implementation",
                "change my source",
                "update my code",
                "fix my own code",
                "rewrite my implementation",
                "optimize my code",
            ]
        });

        let lower_input = user_input.to_lowercase();
        selfdev_keywords
            .iter()
            .any(|keyword| lower_input.contains(keyword))
    }

    /// Determine effective execution mode for a job.
    fn get_execution_mode_for_job(&self, job: &AgentJob) -> ExecutionMode {
        // If job specifies execution mode, use it
        if let Some(mode_str) = job.constraints.execution_mode.as_deref() {
            return ExecutionMode::from_str(mode_str);
        }

        // If selfdev is enabled and intent is detected, use direct mode
        if self.selfdev_enabled && self.detect_selfdev_intent(&job.task.user_input) {
            info!("Selfdev intent detected, using direct execution mode");
            return ExecutionMode::Direct;
        }

        // Otherwise use configured mode
        self.execution_mode
    }
}

/// Process an AgentJob and return AgentResult.
async fn process_job(state: &Arc<AdapterState>, job: AgentJob) -> Result<AgentResult> {
    info!(
        "Processing job {} with correlation {}",
        job.job_id, job.correlation_parent
    );

    // Determine execution mode for this job
    let execution_mode = state.get_execution_mode_for_job(&job);
    info!("Execution mode: {:?}", execution_mode);

    // Get provider from job config or default to openai
    let provider_name = if job.model_config.provider.is_empty() {
        "openai".to_string()
    } else {
        sanitize_string(&job.model_config.provider).into_owned()
    };
    let provider = state.get_provider(&provider_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Provider not found: {}. Available providers: {:?}",
            provider_name,
            state.list_providers()
        )
    })?;

    // Check provider status
    let status = provider.status();
    info!(
        "Provider {} status: {:?}",
        provider_name, status.availability
    );

    // Get model from job config or provider default
    let model = if job.model_config.model_name.is_empty() {
        provider.model()
    } else {
        sanitize_string(&job.model_config.model_name).into_owned()
    };

    // Ensure session (no session_id in AgentJob, use correlation_parent as identifier)
    let session_id = {
        state.session_manager.ensure_session(
            Some(&job.correlation_parent),
            &provider_name,
            model.as_str(),
        )
    };

    // Build message history
    let messages = messages::build_message_history(&job);
    // Get user_input reference instead of cloning (we clone later only if needed)
    let user_input = messages
        .last()
        .map(|m| m.content.as_str())
        .unwrap_or_default();

    // Use cached tool definitions (built once)
    let tools = get_tool_definitions();

    // Call LLM
    let _max_tokens = job.constraints.max_completion_tokens.unwrap_or(4096);
    let _temperature = job.model_config.temperature.unwrap_or(0.7);

    let mut event_stream = provider
        .complete(&messages, &tools, "", Some(&session_id))
        .await?;

    // Collect response
    let mut final_text = String::new();
    let mut tool_invocations: Vec<ToolInvocation> = Vec::new();

    while let Some(event) = event_stream.next().await {
        match event? {
            StreamEvent::TextDelta { text } => {
                final_text.push_str(&text);
            }
            StreamEvent::ToolCall {
                id: _id,
                name,
                arguments,
            } => {
                // Parse arguments (reuse the parsed value)
                let params: serde_json::Value =
                    serde_json::from_str(&arguments).unwrap_or(serde_json::Value::Null);

                // Execute tool based on execution mode and track duration
                let tool_start = tokio::time::Instant::now();
                let result = execute_tool(&name, &params, execution_mode, state).await;
                let duration_ms = tool_start.elapsed().as_millis() as u64;

                tool_invocations.push(ToolInvocation {
                    tool_name: name,
                    parameters: params, // Move instead of clone
                    result,
                    duration_ms,
                });
            }
            StreamEvent::Done => break,
        }
    }

    // Extract side effects
    let (files_written, commands_run) = extract_side_effects(&tool_invocations);

    // Update session (clone user_input and final_text since we need owned Strings)
    messages::update_session(
        state,
        &session_id,
        user_input.to_string(),
        final_text.clone(),
    );

    // Estimate tokens
    let prompt_tokens = estimate_tokens(&messages);
    let completion_tokens = estimate_tokens(&[Message {
        role: ASSISTANT_ROLE.to_string(),
        content: final_text.clone(),
        tool_call_id: None,
    }]);

    Ok(AgentResult {
        job_id: job.job_id,
        correlation_parent: job.correlation_parent,
        output: AgentOutput {
            text: Some(final_text),
            structured: None,
            files_written: files_written.into_iter().map(|p| p.into()).collect(),
            commands_run,
        },
        tool_invocations,
        errors: vec![],
        usage: Some(UsageMetrics {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            cost_cents: None,
        }),
        completed_at: Utc::now().timestamp(),
    })
}

/// Execute a tool based on execution mode.
async fn execute_tool(
    name: &str,
    params: &serde_json::Value,
    execution_mode: ExecutionMode,
    state: &AdapterState,
) -> serde_json::Value {
    match execution_mode {
        ExecutionMode::Sandbox => execute_tool_sandbox(name, params, state).await,
        ExecutionMode::Direct => {
            // Execute directly on host PC
            execute_tool_direct(name, params, state).await
        }
        ExecutionMode::Hybrid => {
            // Use sandbox for normal tools, direct for selfdev tools
            let tool_name = ToolName::from_str(name);
            if matches!(
                tool_name,
                Some(ToolName::ReadSource) | Some(ToolName::WriteSource)
            ) {
                execute_tool_direct(name, params, state).await
            } else {
                execute_tool_sandbox(name, params, state).await
            }
        }
    }
}

/// Execute tool via MCP stdio sandbox.
///
/// Lazy-initializes the MCP client on first use. Tool calls are serialized
/// through a tokio Mutex since the stdio transport is not concurrent-safe.
async fn execute_tool_sandbox(
    name: &str,
    params: &serde_json::Value,
    state: &AdapterState,
) -> serde_json::Value {
    let mut guard = state.sandbox_client.lock().await;

    // Lazy initialization: spawn sandbox on first use
    if guard.is_none() {
        match McpStdioClient::spawn_sandbox(None).await {
            Ok(client) => {
                info!("MCP sandbox client spawned successfully");
                *guard = Some(client);
            }
            Err(e) => {
                warn!("Failed to spawn MCP sandbox client: {}", e);
                return serde_json::json!({
                    "error": format!("Failed to spawn sandbox: {}", e),
                    "params": params
                });
            }
        }
    }

    let client = match guard.as_mut() {
        Some(c) => c,
        None => {
            return serde_json::json!({
                "error": "Sandbox client unavailable",
                "params": params
            });
        }
    };

    // Map tool name and parameters to MCP format (optimized to reduce allocations)
    let (mcp_tool_name, mcp_arguments) = match name {
        "shell_exec" => {
            let command = params.get("command").and_then(|v| v.as_str()).unwrap_or("");
            let working_dir = params.get("working_dir").and_then(|v| v.as_str());
            if let Some(dir) = working_dir {
                (
                    "shell_exec",
                    serde_json::json!({"command": command, "working_dir": dir}),
                )
            } else {
                ("shell_exec", serde_json::json!({"command": command}))
            }
        }
        "file_read" => {
            let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");
            ("file_read", serde_json::json!({"path": path}))
        }
        "file_write" => {
            let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let content = params.get("content").and_then(|v| v.as_str()).unwrap_or("");
            (
                "file_write",
                serde_json::json!({"path": path, "content": content}),
            )
        }
        "file_list" => {
            let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");
            ("file_list", serde_json::json!({"path": path}))
        }
        other => {
            return serde_json::json!({
                "error": format!("Tool '{}' not supported in sandbox mode", other),
                "params": params
            });
        }
    };

    match client.call_tool(mcp_tool_name, mcp_arguments).await {
        Ok(result) => {
            // Translate MCP CallToolResult to our JSON format
            if let Some(content) = result.get("content") {
                // Extract the first text content item
                if let Some(items) = content.as_array() {
                    if let Some(first) = items.first() {
                        if let Some(text) = first.get("text").and_then(|v| v.as_str()) {
                            // Try to parse as JSON (sandbox returns JSON results)
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                                return parsed;
                            }
                            return serde_json::json!({ "result": text });
                        }
                    }
                }
            }
            result
        }
        Err(e) => {
            warn!("MCP tool call failed: {}", e);
            serde_json::json!({
                "error": format!("Sandbox tool execution failed: {}", e),
                "params": params
            })
        }
    }
}

/// Execute tool directly on host PC.
async fn execute_tool_direct(
    name: &str,
    params: &serde_json::Value,
    state: &AdapterState,
) -> serde_json::Value {
    // Try selfdev tools first
    if let Some(result) = selfdev_tools::execute_selfdev_tool(name, params, state).await {
        return result;
    }

    let ctx = ToolContext {
        allowed_base_dir: state.allowed_base_dir.as_ref(),
        execution_mode: wireframe_adapter::tools::ExecutionMode::Direct,
    };

    match ToolName::from_str(name) {
        Some(ToolName::ShellExec) => {
            let command = params.get("command").and_then(|v| v.as_str()).unwrap_or("");
            let working_dir = params.get("working_dir").and_then(|v| v.as_str());
            execute_shell(command, working_dir, &ctx).await
        }
        Some(ToolName::FileRead) => {
            let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");
            read_file(path, &ctx).await
        }
        Some(ToolName::FileWrite) => {
            let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let content = params.get("content").and_then(|v| v.as_str()).unwrap_or("");
            write_file(path, content, &ctx).await
        }
        Some(ToolName::FileList) => {
            let path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");
            list_directory(path, &ctx).await
        }
        None => {
            serde_json::json!({
                "error": format!("Unknown tool: {}", name)
            })
        }
        _ => {
            serde_json::json!({
                "error": format!("Unknown tool: {}", name)
            })
        }
    }
}

/// Initialize tracing with environment variable support.
fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG")
                .unwrap_or_else(|_| "wireframe_adapter_rust=info,async_nats=warn".to_string()),
        )
        .init();
}

/// Connect to NATS with optimized settings.
async fn connect_nats() -> Result<async_nats::Client> {
    let nats_url =
        env::var("WIREFRAME_AI_NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());

    info!("Connecting to NATS at {}", nats_url);

    let client = async_nats::connect_with_options(
        &nats_url,
        async_nats::ConnectOptions::new()
            .retry_on_initial_connect()
            .max_reconnects(100)
            .reconnect_delay_callback(|_| std::time::Duration::from_millis(500))
            .ping_interval(std::time::Duration::from_secs(20)),
    )
    .await?;

    info!("Connected to NATS");
    Ok(client)
}

/// Log adapter configuration.
fn log_adapter_config(state: &AdapterState) {
    let providers = state.list_providers();
    info!("Registered providers: {:?}", providers);
    info!("Execution mode: {:?}", state.execution_mode);
    info!("Selfdev enabled: {}", state.selfdev_enabled);
    if let Some(ref root) = state.source_root {
        info!("Source root: {}", root.display());
    }
    if let Some(ref bin) = state.binary_path {
        info!("Binary path: {}", bin.display());
    }
}

/// Announce module online with selfdev capability.
async fn announce_module_online(client: &async_nats::Client, state: &AdapterState) -> Result<()> {
    let source_root_str = state
        .source_root
        .as_ref()
        .map(|p| p.to_string_lossy().to_string());
    let binary_path_str = state
        .binary_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string());

    announce_online_with_selfdev(
        client,
        get_module_name(),
        get_module_version(),
        &[AGENT_JOB_TOPIC],
        &[AGENT_RESULT_TOPIC],
        state.selfdev_enabled,
        source_root_str.as_deref(),
        binary_path_str.as_deref(),
    )
    .await?;

    Ok(())
}

/// Process a single NATS message.
async fn process_message(
    message: async_nats::Message,
    state: &Arc<AdapterState>,
    client: &async_nats::Client,
) -> Result<()> {
    let envelope: Envelope<serde_json::Value> = match serde_json::from_slice(&message.payload) {
        Ok(env) => env,
        Err(e) => {
            error!("Failed to deserialize envelope: {}", e);
            return Ok(());
        }
    };

    if envelope.topic != AGENT_JOB_TOPIC {
        error!("Unexpected topic: {}", envelope.topic);
        return Ok(());
    }

    let job: AgentJob = match serde_json::from_value(envelope.payload) {
        Ok(job) => job,
        Err(e) => {
            error!("Failed to deserialize job: {}", e);
            return Ok(());
        }
    };

    let session_id = envelope.session_id.clone();

    // Process job
    let state_clone = Arc::clone(state);
    let result = tokio::spawn(async move { process_job(&state_clone, job).await }).await??;

    // Publish result (reuse topic strings to avoid allocation)
    let result_envelope = Envelope::new(
        AGENT_RESULT_TOPIC.to_string(),
        serde_json::to_value(&result)?,
        Some(session_id),
    );
    let result_payload = serde_json::to_vec(&result_envelope)?;

    client
        .publish(AGENT_RESULT_TOPIC, result_payload.into())
        .await?;
    info!("Published result for job {}", result.job_id);

    Ok(())
}

/// Run the main message processing loop.
async fn run_message_loop(client: async_nats::Client, state: Arc<AdapterState>) -> Result<()> {
    let mut subscriber = client
        .queue_subscribe(AGENT_JOB_TOPIC, AGENT_WORKER_QUEUE.to_string())
        .await?;

    info!(
        "Subscribed to {} with queue group {}",
        AGENT_JOB_TOPIC, AGENT_WORKER_QUEUE
    );

    while let Some(message) = subscriber.next().await {
        if let Err(e) = process_message(message, &state, &client).await {
            error!("Failed to process message: {}", e);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let client = connect_nats().await?;
    let state = Arc::new(AdapterState::new());

    log_adapter_config(&state);
    announce_module_online(&client, &state).await?;

    run_message_loop(client, state).await?;

    Ok(())
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // -------------------------------------------------------------------------
    // ExecutionMode
    // -------------------------------------------------------------------------

    #[test]
    fn test_execution_mode_from_str() {
        assert_eq!(ExecutionMode::from_str("sandbox"), ExecutionMode::Sandbox);
        assert_eq!(ExecutionMode::from_str("direct"), ExecutionMode::Direct);
        assert_eq!(ExecutionMode::from_str("hybrid"), ExecutionMode::Hybrid);
        assert_eq!(ExecutionMode::from_str("unknown"), ExecutionMode::Direct);
        assert_eq!(ExecutionMode::from_str("SANDBOX"), ExecutionMode::Sandbox);
        assert_eq!(ExecutionMode::from_str("Direct"), ExecutionMode::Direct);
    }

    // -------------------------------------------------------------------------
    // ToolName
    // -------------------------------------------------------------------------

    #[test]
    fn test_tool_name_roundtrip() {
        let names = [
            ToolName::ShellExec,
            ToolName::FileRead,
            ToolName::FileWrite,
            ToolName::FileList,
            ToolName::ReadSource,
            ToolName::WriteSource,
            ToolName::CompileSelf,
            ToolName::RestartSelf,
            ToolName::SwitchModule,
        ];
        for name in &names {
            let s = name.as_str();
            let parsed = ToolName::from_str(s);
            assert_eq!(
                parsed.as_ref(),
                Some(name),
                "Roundtrip failed for {:?} -> {} -> {:?}",
                name,
                s,
                parsed
            );
        }
    }

    #[test]
    fn test_tool_name_from_str_unknown() {
        assert_eq!(ToolName::from_str("unknown_tool"), None);
        assert_eq!(ToolName::from_str(""), None);
        assert_eq!(ToolName::from_str("shellExec"), None); // case sensitive
    }

    #[test]
    fn test_tool_name_as_str_values() {
        assert_eq!(ToolName::ShellExec.as_str(), "shell_exec");
        assert_eq!(ToolName::FileRead.as_str(), "file_read");
        assert_eq!(ToolName::FileWrite.as_str(), "file_write");
        assert_eq!(ToolName::CompileSelf.as_str(), "compile_self");
        assert_eq!(ToolName::RestartSelf.as_str(), "restart_self");
        assert_eq!(ToolName::SwitchModule.as_str(), "switch_module");
    }

    // -------------------------------------------------------------------------
    // AdapterState::detect_selfdev_intent
    // -------------------------------------------------------------------------

    #[test]
    fn test_detect_selfdev_intent_positive() {
        let state = AdapterState::new();
        assert!(state.detect_selfdev_intent("Please edit my code to fix this bug"));
        assert!(state.detect_selfdev_intent("I want to modify myself"));
        assert!(state.detect_selfdev_intent("Improve my implementation please"));
        assert!(state.detect_selfdev_intent("Can you change my source?"));
        assert!(state.detect_selfdev_intent("Update my code"));
        assert!(state.detect_selfdev_intent("Fix my own code now"));
        assert!(state.detect_selfdev_intent("Rewrite my implementation"));
        assert!(state.detect_selfdev_intent("OPTIMIZE MY CODE")); // case insensitive
    }

    #[test]
    fn test_detect_selfdev_intent_negative() {
        let state = AdapterState::new();
        assert!(!state.detect_selfdev_intent("Hello, how are you?"));
        assert!(!state.detect_selfdev_intent("Write a poem about code"));
        assert!(!state.detect_selfdev_intent("Edit the user profile"));
        assert!(!state.detect_selfdev_intent("Modify the database schema"));
        assert!(!state.detect_selfdev_intent(""));
    }

    // -------------------------------------------------------------------------
    // extract_side_effects
    // -------------------------------------------------------------------------

    #[test]
    fn test_extract_side_effects_empty() {
        let (files, commands) = extract_side_effects(&[]);
        assert!(files.is_empty());
        assert!(commands.is_empty());
    }

    #[test]
    fn test_extract_side_effects_files_written() {
        let invocations = vec![
            ToolInvocation {
                tool_name: "file_write".to_string(),
                parameters: serde_json::json!({"path": "/tmp/test.txt"}),
                result: serde_json::json!({"success": true}),
                duration_ms: 0,
            },
            ToolInvocation {
                tool_name: "file_read".to_string(),
                parameters: serde_json::json!({"path": "/tmp/other.txt"}),
                result: serde_json::json!({"content": "hello"}),
                duration_ms: 0,
            },
        ];
        let (files, commands) = extract_side_effects(&invocations);
        assert_eq!(files, vec!["/tmp/test.txt"]);
        assert!(commands.is_empty());
    }

    #[test]
    fn test_extract_side_effects_commands_run() {
        let invocations = vec![
            ToolInvocation {
                tool_name: "shell_exec".to_string(),
                parameters: serde_json::json!({"command": "ls -la"}),
                result: serde_json::json!({"success": true}),
                duration_ms: 0,
            },
            ToolInvocation {
                tool_name: "shell_exec".to_string(),
                parameters: serde_json::json!({"command": "cargo build"}),
                result: serde_json::json!({"success": true}),
                duration_ms: 0,
            },
        ];
        let (files, commands) = extract_side_effects(&invocations);
        assert!(files.is_empty());
        assert_eq!(commands, vec!["ls -la", "cargo build"]);
    }

    #[test]
    fn test_extract_side_effects_mixed() {
        let invocations = vec![
            ToolInvocation {
                tool_name: "file_write".to_string(),
                parameters: serde_json::json!({"path": "output.txt"}),
                result: serde_json::json!({"success": true}),
                duration_ms: 0,
            },
            ToolInvocation {
                tool_name: "shell_exec".to_string(),
                parameters: serde_json::json!({"command": "echo hello"}),
                result: serde_json::json!({"success": true}),
                duration_ms: 0,
            },
        ];
        let (files, commands) = extract_side_effects(&invocations);
        assert_eq!(files, vec!["output.txt"]);
        assert_eq!(commands, vec!["echo hello"]);
    }

    // -------------------------------------------------------------------------
    // build_tool_definitions
    // -------------------------------------------------------------------------

    #[test]
    fn test_build_tool_definitions_count() {
        let tools = build_tool_definitions();
        assert_eq!(tools.len(), 9);
    }

    #[test]
    fn test_build_tool_definitions_names() {
        let tools = build_tool_definitions();
        let names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
        assert!(names.contains(&"shell_exec".to_string()));
        assert!(names.contains(&"file_read".to_string()));
        assert!(names.contains(&"file_write".to_string()));
        assert!(names.contains(&"file_list".to_string()));
        assert!(names.contains(&"read_source".to_string()));
        assert!(names.contains(&"write_source".to_string()));
        assert!(names.contains(&"compile_self".to_string()));
        assert!(names.contains(&"restart_self".to_string()));
        assert!(names.contains(&"switch_module".to_string()));
    }

    #[test]
    fn test_build_tool_definitions_shell_exec_structure() {
        let tools = build_tool_definitions();
        let shell = tools.iter().find(|t| t.name == "shell_exec").unwrap();
        assert!(shell.description.contains("sandbox"));
        let params = shell.parameters.as_object().unwrap();
        assert!(params.contains_key("properties"));
        assert!(params.contains_key("required"));
    }

    // -------------------------------------------------------------------------
    // validate_path
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_path_traversal_blocked() {
        assert!(validate_path("../etc/passwd", None).is_err());
        assert!(validate_path("foo/../../bar", None).is_err());
        assert!(validate_path("~/.ssh/id_rsa", None).is_err());
    }

    #[test]
    fn test_validate_path_nonexistent() {
        // Nonexistent path should fail at canonicalize stage
        assert!(validate_path("/nonexistent/path/12345", None).is_err());
    }

    #[tokio::test]
    async fn test_validate_path_with_base_dir() {
        use tokio::fs;
        let temp_dir = std::env::temp_dir().join("wireframe_test_validate_path");
        let _ = fs::remove_dir_all(&temp_dir).await;
        fs::create_dir_all(&temp_dir).await.unwrap();
        fs::create_dir_all(temp_dir.join("subdir")).await.unwrap();

        let base = Some(temp_dir.as_path());

        // Path within base should succeed
        let result = validate_path(&temp_dir.join("subdir").to_string_lossy(), base);
        assert!(result.is_ok());

        // Path outside base should fail
        let result = validate_path("/", base);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&temp_dir).await;
    }

    // -------------------------------------------------------------------------
    // build_message_history
    // -------------------------------------------------------------------------

    #[test]
    fn test_build_message_history_basic() {
        let job = AgentJob {
            job_id: "test_job".to_string(),
            correlation_parent: "corr_123".to_string(),
            task: agentic_sdk::message_types::TaskDescription {
                user_input: "Hello, world!".to_string(),
                sub_task: None,
                output_format: None,
                user_constraints: vec![],
            },
            context: agentic_sdk::message_types::ContextPackage {
                memory_chunks: vec![],
                session_history: vec![],
                readonly_files: vec![],
                safe_env: HashMap::new(),
                working_dir: std::path::PathBuf::from("."),
                max_context_tokens: 32768,
            },
            available_tool_capabilities: vec![],
            constraints: agentic_sdk::message_types::ExecutionConstraints::default(),
            model_config: agentic_sdk::message_types::ModelConfig::default(),
            metadata: agentic_sdk::message_types::JobMetadata::default(),
            adapter_hints: None,
            schema_version: 1,
        };

        let messages = messages::build_message_history(&job);
        assert!(!messages.is_empty());
        assert_eq!(messages[messages.len() - 1].role, "user");
        assert_eq!(messages[messages.len() - 1].content, "Hello, world!");
    }

    #[test]
    fn test_build_message_history_sanitizes_input() {
        let job = AgentJob {
            job_id: "test_job".to_string(),
            correlation_parent: "corr_123".to_string(),
            task: agentic_sdk::message_types::TaskDescription {
                user_input: "Hello\x00world".to_string(),
                sub_task: None,
                output_format: None,
                user_constraints: vec![],
            },
            context: agentic_sdk::message_types::ContextPackage {
                memory_chunks: vec![],
                session_history: vec![],
                readonly_files: vec![],
                safe_env: HashMap::new(),
                working_dir: std::path::PathBuf::from("."),
                max_context_tokens: 32768,
            },
            available_tool_capabilities: vec![],
            constraints: agentic_sdk::message_types::ExecutionConstraints::default(),
            model_config: agentic_sdk::message_types::ModelConfig::default(),
            metadata: agentic_sdk::message_types::JobMetadata::default(),
            adapter_hints: None,
            schema_version: 1,
        };

        let messages = messages::build_message_history(&job);
        let last = messages.last().unwrap();
        assert!(!last.content.contains('\0'));
        assert_eq!(last.content, "Helloworld");
    }

    // -------------------------------------------------------------------------
    // Integration tests for execute_tool_direct
    // -------------------------------------------------------------------------

    // TODO: Re-enable when execute_tool_direct is made accessible for testing
    /*
     /// Helper to create a minimal AdapterState for file operation tests.
     fn test_adapter_state(allowed_base_dir: Option<PathBuf>) -> AdapterState {
         AdapterState {
             providers: HashMap::new(),
             session_manager: SessionManager::new(),
             execution_mode: ExecutionMode::Direct,
             selfdev_enabled: false,
             source_root: None,
             binary_path: None,
             allowed_base_dir,
             sandbox_client: tokio::sync::Mutex::new(None),
         }
     }

     // // #[tokio::test]
     async fn test_execute_tool_direct_file_write_and_read() {
         let temp_dir = std::env::temp_dir().join("wireframe_test_file_ops");
         let _ = tokio::fs::remove_dir_all(&temp_dir).await;
         tokio::fs::create_dir_all(&temp_dir).await.unwrap();

         let state = test_adapter_state(Some(temp_dir.clone()));
         let file_path = temp_dir.join("test.txt");
         let file_path_str = file_path.to_string_lossy().to_string();

         // Write file
         let write_params = serde_json::json!({
             "path": file_path_str,
             "content": "Hello, Wireframe AI!"
         });
         let write_result = execute_tool_direct("file_write", &write_params, &state).await;
         assert_eq!(
             write_result.get("success").and_then(|v| v.as_bool()),
             Some(true),
             "file_write failed: {:?}",
             write_result
         );

         // Read file
         let read_params = serde_json::json!({"path": file_path_str});
         let read_result = execute_tool_direct("file_read", &read_params, &state).await;
         assert_eq!(
             read_result.get("content").and_then(|v| v.as_str()),
             Some("Hello, Wireframe AI!"),
             "file_read failed: {:?}",
             read_result
         );

         // Cleanup
         let _ = tokio::fs::remove_dir_all(&temp_dir).await;
     }

     // // #[t#[tokio::test]
     async fn test_execute_tool_direct_file_list() {
         let temp_dir = std::env::temp_dir().join("wireframe_test_list");
         let _ = tokio::fs::remove_dir_all(&temp_dir).await;
         tokio::fs::create_dir_all(&temp_dir).await.unwrap();
         tokio::fs::write(temp_dir.join("a.txt"), "a").await.unwrap();
         tokio::fs::write(temp_dir.join("b.txt"), "b").await.unwrap();

         let state = test_adapter_state(Some(temp_dir.clone()));
         let params = serde_json::json!({"path": temp_dir.to_string_lossy().to_string()});
         let result = execute_tool_direct("file_list", &params, &state).await;

         let files = result
             .get("files")
             .and_then(|v| v.as_array())
             .expect("Expected files array");
         let file_names: Vec<String> = files
             .iter()
             .filter_map(|v| v.as_str().map(|s| s.to_string()))
             .collect();
         assert!(file_names.contains(&"a.txt".to_string()));
         assert!(file_names.contains(&"b.txt".to_string()));

         let _ = tokio::fs::remove_dir_all(&temp_dir).await;
     }

     #[t#[tokio::test]
     async fn test_execute_tool_direct_file_read_path_traversal() {
         let safe_dir = std::env::temp_dir().join("wireframe_test_safe");
         let _ = tokio::fs::remove_dir_all(&safe_dir).await;
         tokio::fs::create_dir_all(&safe_dir).await.unwrap();

         let state = test_adapter_state(Some(safe_dir.clone()));
         let params = serde_json::json!({"path": "/etc/passwd"});
         let result = execute_tool_direct("file_read", &params, &state).await;
         assert!(
             result.get("error").is_some(),
             "Expected error for path traversal, got: {:?}",
             result
         );

         let _ = tokio::fs::remove_dir_all(&safe_dir).await;
     }

     // -------------------------------------------------------------------------
     // validate_path_for_write
     // -------------------------------------------------------------------------

     #[test]
     fn test_validate_path_for_write_traversal_blocked() {
         assert!(validate_path_for_write("../etc/passwd", None).is_err());
         assert!(validate_path_for_write("foo/../../bar", None).is_err());
         assert!(validate_path_for_write("~/.ssh/id_rsa", None).is_err());
     }

     #[tokio::test]
     async fn test_validate_path_for_write_nonexistent_file_ok() {
         let temp_dir = std::env::temp_dir().join("wireframe_test_write");
         let _ = tokio::fs::remove_dir_all(&temp_dir).await;
         tokio::fs::create_dir_all(&temp_dir).await.unwrap();

         let result = validate_path_for_write(
             &temp_dir.join("new_file.txt").to_string_lossy(),
             Some(temp_dir.as_path()),
         );
         assert!(result.is_ok());

         let _ = tokio::fs::remove_dir_all(&temp_dir).await;
     }

     #[tokio::test]
     async fn test_validate_path_for_write_outside_base_blocked() {
         let safe_dir = std::env::temp_dir().join("wireframe_test_write_safe");
         let _ = tokio::fs::remove_dir_all(&safe_dir).await;
         tokio::fs::create_dir_all(&safe_dir).await.unwrap();

         let result = validate_path_for_write("/etc/passwd", Some(safe_dir.as_path()));
         assert!(result.is_err());

         let _ = tokio::fs::remove_dir_all(&safe_dir).await;
     }

     #[tokio::test]
     async fn test_execute_tool_direct_shell_exec_echo() {
         let state = test_adapter_state(None);
         let params = serde_json::json!({"command": "echo hello"});
         let result = execute_tool_direct("shell_exec", &params, &state).await;
         assert_eq!(
             result.get("success").and_then(|v| v.as_bool()),
             Some(true),
             "shell_exec failed: {:?}",
             result
         );
         let stdout = result.get("stdout").and_then(|v| v.as_str()).unwrap_or("");
         assert!(
             stdout.contains("hello"),
             "Expected 'hello' in stdout: {}",
             stdout
         );
     }

    tokio::test]
     async fn test_execute_tool_direct_shell_exec_blocked() {
         let state = test_adapter_state(None);
         let params = serde_json::json!({"command": "bash -c echo hello"});
         let result = execute_tool_direct("shell_exec", &params, &state).await;
         assert!(
             result.get("error").is_some(),
             "Expected error for blocked command, got: {:?}",
             result
         );
     }

     okio::test]
     async fn test_execute_tool_direct_unknown_tool() {
         let state = test_adapter_state(None);
         let params = serde_json::json!({"foo": "bar"});
         let result = execute_tool_direct("unknown_tool", &params, &state).await;
         assert!(
             result.get("error").is_some(),
             "Expected error for unknown tool, got: {:?}",
             result
         );
     }

     #[tokio::test]
     async fn test_execute_tool_direct_missing_params() {
         let state = test_adapter_state(None);
         let result = execute_tool_direct("file_read", &serde_json::json!({}), &state).await;
         assert!(
             result.get("error").is_some(),
             "Expected error for missing params, got: {:?}",
             result
         );
     }

     // -------------------------------------------------------------------------
     // detect_platform_shell
     // -------------------------------------------------------------------------

     // #[test]
     fn test_detect_platform_shell_returns_nonempty() {
         let (shell, flag) = detect_platform_shell();
         assert!(!shell.is_empty(), "Shell should not be empty");
         assert!(!flag.is_empty(), "Flag should not be empty");
     }

     // #[test]
     fn test_detect_platform_shell_explicit_override() {
         // Set explicit override and verify it's used
         env::set_var("WIREFRAME_AI_SHELL", "custom_shell");
         let (shell, flag) = detect_platform_shell();
         assert_eq!(shell, "custom_shell");
         assert_eq!(flag, "-c");
         env::remove_var("WIREFRAME_AI_SHELL");
     }
     */
}
