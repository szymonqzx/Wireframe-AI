//! Tests for the markdown output plugin.

use agentic_sdk::message_types::TaskComplete;
use agentic_sdk::plugins::interface::OutputFormatter;
use agentic_sdk::Plugin;
use output_markdown::MarkdownOutput;

#[tokio::test]
async fn test_output_markdown_plugin_id() {
    let output = MarkdownOutput::new();
    assert_eq!(output.plugin_id(), "output-markdown");
}

#[tokio::test]
async fn test_output_markdown_format_result() {
    let output = MarkdownOutput::new();
    let task_complete = TaskComplete {
        session_id: "test-session".to_string(),
        correlation_id: "test-correlation".to_string(),
        result: "test result".to_string(),
        side_effects: vec![],
        warnings: vec![],
        completed_at: 0,
    };
    let result = output.format_result(&task_complete).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test result");
}
