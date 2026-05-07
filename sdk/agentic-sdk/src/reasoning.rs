//! Reasoning adapter patterns for Wireframe AI modules.
//!
//! Provides reusable patterns for tool composition, tool chaining,
//! and state management in reasoning adapters.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A composed tool that chains multiple tools together.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComposedTool {
    pub name: String,
    pub description: String,
    pub steps: Vec<ToolStep>,
}

/// A single step in a composed tool.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolStep {
    pub tool_name: String,
    pub input_mapping: HashMap<String, String>,
    pub output_key: String,
}

impl ComposedTool {
    /// Create a new composed tool.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            steps: vec![],
        }
    }

    /// Add a step to the composition.
    pub fn step(mut self, tool_name: impl Into<String>, output_key: impl Into<String>) -> Self {
        self.steps.push(ToolStep {
            tool_name: tool_name.into(),
            input_mapping: HashMap::new(),
            output_key: output_key.into(),
        });
        self
    }

    /// Map an input field for the last step.
    pub fn map_input(mut self, field: impl Into<String>, from: impl Into<String>) -> Self {
        if let Some(step) = self.steps.last_mut() {
            step.input_mapping.insert(field.into(), from.into());
        }
        self
    }
}

/// Tool chain executor: runs a sequence of tools, piping outputs to inputs.
pub struct ToolChainExecutor {
    state: HashMap<String, Value>,
}

impl Default for ToolChainExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolChainExecutor {
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
        }
    }

    /// Set initial state for the chain.
    pub fn set_initial(&mut self, key: impl Into<String>, value: Value) {
        self.state.insert(key.into(), value);
    }

    /// Execute a tool chain with the current state using a provided tool executor.
    ///
    /// `executor` is called for each step with the tool name and resolved inputs.
    /// It must return the tool output, which is stored under the step's `output_key`.
    pub fn execute_with<F>(
        &mut self,
        chain: &ComposedTool,
        mut executor: F,
    ) -> HashMap<String, Value>
    where
        F: FnMut(&str, HashMap<String, Value>) -> Value,
    {
        for step in &chain.steps {
            let mut inputs = HashMap::new();
            for (field, from_key) in &step.input_mapping {
                if let Some(value) = self.state.get(from_key) {
                    inputs.insert(field.clone(), value.clone());
                }
            }
            let result = executor(&step.tool_name, inputs);
            self.state.insert(step.output_key.clone(), result);
        }
        self.state.clone()
    }

    /// Dry-run a tool chain without invoking real tools.
    /// Returns the state populated with placeholder results for each step.
    pub fn execute_dry_run(&mut self, chain: &ComposedTool) -> HashMap<String, Value> {
        for step in &chain.steps {
            let result = serde_json::json!({
                "tool": step.tool_name,
                "inputs": step.input_mapping,
                "executed_at": chrono::Utc::now().timestamp(),
                "dry_run": true,
            });
            self.state.insert(step.output_key.clone(), result);
        }
        self.state.clone()
    }

    /// Get a value from the chain state.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.state.get(key)
    }
}

/// State management for multi-turn agent workflows.
pub struct AgentStateManager {
    states: HashMap<String, AgentState>,
}

/// Persistent state for an agent session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentState {
    pub session_id: String,
    pub variables: HashMap<String, Value>,
    pub tool_results: Vec<ToolResult>,
    pub turn_count: u32,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Result of a single tool invocation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub input: Value,
    pub output: Value,
    pub success: bool,
    pub duration_ms: u64,
}

impl Default for AgentStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentStateManager {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    /// Get or create state for a session.
    pub fn get_or_create(&mut self, session_id: impl Into<String>) -> &mut AgentState {
        let id = session_id.into();
        let now = chrono::Utc::now().timestamp();
        self.states.entry(id.clone()).or_insert_with(|| AgentState {
            session_id: id,
            variables: HashMap::new(),
            tool_results: vec![],
            turn_count: 0,
            created_at: now,
            updated_at: now,
        })
    }

    /// Record a tool result for a session.
    pub fn record_tool_result(&mut self, session_id: &str, result: ToolResult) -> Option<()> {
        let state = self.states.get_mut(session_id)?;
        state.tool_results.push(result);
        state.turn_count += 1;
        state.updated_at = chrono::Utc::now().timestamp();
        Some(())
    }

    /// Set a state variable.
    pub fn set_variable(
        &mut self,
        session_id: &str,
        key: impl Into<String>,
        value: Value,
    ) -> Option<()> {
        let state = self.states.get_mut(session_id)?;
        state.variables.insert(key.into(), value);
        state.updated_at = chrono::Utc::now().timestamp();
        Some(())
    }

    /// Get a state variable.
    pub fn get_variable(&self, session_id: &str, key: &str) -> Option<&Value> {
        self.states.get(session_id)?.variables.get(key)
    }

    /// Get all tool results for a session.
    pub fn get_tool_results(&self, session_id: &str) -> Option<&[ToolResult]> {
        Some(self.states.get(session_id)?.tool_results.as_slice())
    }

    /// Summarize state for LLM context window.
    pub fn summarize(&self, session_id: &str) -> Option<Value> {
        let state = self.states.get(session_id)?;
        Some(serde_json::json!({
            "turn_count": state.turn_count,
            "variables": state.variables,
            "recent_tools": state.tool_results.iter().rev().take(5).collect::<Vec<_>>(),
        }))
    }
}

/// Caching strategies for agent outputs.
pub struct AgentCache {
    store: HashMap<String, CacheEntry>,
    default_ttl_seconds: u64,
}

struct CacheEntry {
    value: Value,
    expires_at: i64,
    hit_count: u64,
}

impl AgentCache {
    pub fn new(default_ttl_seconds: u64) -> Self {
        Self {
            store: HashMap::new(),
            default_ttl_seconds,
        }
    }

    /// Generate a cache key from a prompt and model config.
    /// Uses SHA-256 for a stable, cross-platform hash.
    pub fn key(prompt: &str, model: &str, temperature: f32) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        hasher.update(model.as_bytes());
        hasher.update(temperature.to_bits().to_le_bytes());
        let result = hasher.finalize();
        // Manual hex encode to avoid extra dependency
        result.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Get a cached value if not expired.
    pub fn get(&mut self, key: &str) -> Option<Value> {
        let entry = self.store.get_mut(key)?;
        if entry.expires_at > chrono::Utc::now().timestamp() {
            entry.hit_count += 1;
            Some(entry.value.clone())
        } else {
            None
        }
    }

    /// Store a value in the cache.
    pub fn set(&mut self, key: impl Into<String>, value: Value, ttl_seconds: Option<u64>) {
        let ttl = ttl_seconds.unwrap_or(self.default_ttl_seconds);
        let entry = CacheEntry {
            value,
            expires_at: chrono::Utc::now().timestamp() + ttl as i64,
            hit_count: 0,
        };
        self.store.insert(key.into(), entry);
    }

    /// Evict expired entries and return stats.
    pub fn evict_expired(&mut self) -> (usize, usize) {
        let now = chrono::Utc::now().timestamp();
        let before = self.store.len();
        self.store.retain(|_, entry| entry.expires_at > now);
        let after = self.store.len();
        (before - after, after)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composed_tool_builder() {
        let tool = ComposedTool::new("search_and_summarize", "Search then summarize")
            .step("search", "search_result")
            .map_input("query", "user_input")
            .step("summarize", "summary")
            .map_input("text", "search_result");

        assert_eq!(tool.steps.len(), 2);
        assert_eq!(tool.steps[0].tool_name, "search");
    }

    #[test]
    fn test_tool_chain_execution() {
        let tool = ComposedTool::new("chain", "test chain")
            .step("step1", "out1")
            .step("step2", "out2");

        let mut executor = ToolChainExecutor::new();
        executor.set_initial("input", serde_json::json!("hello"));
        let result = executor.execute_dry_run(&tool);
        assert!(result.contains_key("out2"));

        // Verify execute_with works with a real executor
        let mut executor2 = ToolChainExecutor::new();
        executor2.set_initial("input", serde_json::json!("hello"));
        let result2 = executor2.execute_with(&tool, |tool_name, inputs| {
            serde_json::json!({ "tool": tool_name, "inputs": inputs, "mock": true })
        });
        assert!(result2.contains_key("out2"));
        assert!(result2
            .get("out2")
            .unwrap()
            .get("mock")
            .unwrap()
            .as_bool()
            .unwrap());
    }

    #[test]
    fn test_agent_state_manager() {
        let mut manager = AgentStateManager::new();
        let state = manager.get_or_create("session_1");
        assert_eq!(state.turn_count, 0);

        manager.record_tool_result(
            "session_1",
            ToolResult {
                tool_name: "test".to_string(),
                input: serde_json::json!(null),
                output: serde_json::json!("ok"),
                success: true,
                duration_ms: 100,
            },
        );

        assert_eq!(manager.get_or_create("session_1").turn_count, 1);
    }

    #[test]
    fn test_agent_cache_hit() {
        let mut cache = AgentCache::new(60);
        let key = AgentCache::key("hello", "gpt-4", 0.7);
        cache.set(&key, serde_json::json!("world"), None);
        assert_eq!(cache.get(&key), Some(serde_json::json!("world")));
    }

    #[test]
    fn test_agent_cache_expiration() {
        let mut cache = AgentCache::new(0);
        let key = "test".to_string();
        cache.set(&key, serde_json::json!("value"), Some(0));
        // Entry should be expired immediately
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert_eq!(cache.get(&key), None);
    }
}
