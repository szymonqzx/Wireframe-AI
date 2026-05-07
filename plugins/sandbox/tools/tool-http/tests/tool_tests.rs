//! Tests for the HTTP tool plugin.

use agentic_sdk::plugins::sandbox::Tool;
use agentic_sdk::Plugin;
use tool_http::HttpTool;

#[tokio::test]
async fn test_tool_http_plugin_id() {
    let tool = HttpTool::new();
    assert_eq!(tool.plugin_id(), "tool-http");
}

#[tokio::test]
async fn test_tool_http_tool_name() {
    let tool = HttpTool::new();
    assert_eq!(tool.tool_name(), "http");
}

#[tokio::test]
async fn test_tool_http_input_schema() {
    let tool = HttpTool::new();
    let schema = tool.input_schema();
    assert!(schema.is_object());
}
