//! Tests for the HTTP tool plugin.

use agentic_sdk::plugins::sandbox::Tool;
use agentic_sdk::Plugin;
use tool_http::HttpTool;

#[tokio::test]
async fn test_tool_http_plugin_id() {
    let tool = HttpTool::new().unwrap();
    assert_eq!(tool.plugin_id(), "tool-http");
}

#[tokio::test]
async fn test_tool_http_tool_name() {
    let tool = HttpTool::new().unwrap();
    assert_eq!(tool.tool_name(), "http");
}

#[tokio::test]
async fn test_tool_http_input_schema() {
    let tool = HttpTool::new().unwrap();
    let schema = tool.input_schema();
    assert!(schema.is_object());
    assert!(schema["properties"].get("url").is_some());
    assert!(schema["properties"].get("headers").is_some());
    assert!(schema["properties"].get("body").is_some());
}

#[tokio::test]
async fn test_tool_http_execute_missing_url() {
    let tool = HttpTool::new().unwrap();
    let ctx = agentic_sdk::plugins::sandbox::SandboxContext {
        working_dir: "/tmp".to_string(),
        environment: vec![],
        allowed_paths: vec![],
    };
    let params = serde_json::json!({});
    let result = tool.execute(params, &ctx).await;
    assert!(result.is_err());
}
