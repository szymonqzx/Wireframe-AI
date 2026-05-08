//! Utility functions for the adapter.
//!
//! Provides token estimation and side effect extraction utilities.

use crate::ToolName;
use std::sync::OnceLock;
use tiktoken_rs::cl100k_base;
use wireframe_provider_core::Message;

/// Get cached tokenizer (built once on first use).
pub fn get_tokenizer() -> &'static tiktoken_rs::CoreBPE {
    static TOKENIZER: OnceLock<tiktoken_rs::CoreBPE> = OnceLock::new();
    TOKENIZER.get_or_init(|| cl100k_base().expect("Failed to load tokenizer"))
}

/// Estimate token count using proper tokenizer (cl100k_base for GPT-4).
pub fn estimate_tokens(messages: &[Message]) -> usize {
    let bpe = get_tokenizer();

    // Pre-allocate capacity to avoid reallocations
    let mut text = String::with_capacity(
        messages
            .iter()
            .map(|m| m.role.len() + m.content.len() + 3)
            .sum::<usize>(),
    );

    for (i, m) in messages.iter().enumerate() {
        if i > 0 {
            text.push('\n');
        }
        text.push_str(&m.role);
        text.push_str(": ");
        text.push_str(&m.content);
    }

    bpe.encode_with_special_tokens(&text).len()
}

/// Extract side effects from tool invocations.
pub fn extract_side_effects(
    tool_invocations: &[agentic_sdk::ToolInvocation],
) -> (Vec<String>, Vec<String>) {
    let files_written: Vec<String> = tool_invocations
        .iter()
        .filter(|ti| ti.tool_name == ToolName::FileWrite.as_str())
        .filter_map(|ti| ti.parameters.get("path"))
        .filter_map(|p| p.as_str())
        .map(String::from)
        .collect();

    let commands_run: Vec<String> = tool_invocations
        .iter()
        .filter(|ti| ti.tool_name == ToolName::ShellExec.as_str())
        .filter_map(|ti| ti.parameters.get("command"))
        .filter_map(|c| c.as_str())
        .map(String::from)
        .collect();

    (files_written, commands_run)
}
