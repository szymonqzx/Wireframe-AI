//! Tests for the shell tool plugin.

use agentic_sdk::plugins::sandbox::{SandboxContext, Tool};
use agentic_sdk::Plugin;
use serde_json::json;
use tool_shell::ShellTool;

#[tokio::test]
async fn test_tool_shell_input_schema() {
    let tool = ShellTool::new();
    let schema = tool.input_schema();

    assert!(schema.is_object());
    let props = schema.get("properties").unwrap().as_object().unwrap();
    assert!(props.contains_key("command"));
    assert!(props.contains_key("working_dir"));
    assert!(props.contains_key("timeout_secs"));
}

#[tokio::test]
async fn test_tool_shell_plugin_id() {
    let tool = ShellTool::new();
    assert_eq!(tool.plugin_id(), "tool-shell");
}

#[tokio::test]
async fn test_tool_shell_tool_name() {
    let tool = ShellTool::new();
    assert_eq!(tool.tool_name(), "shell");
}

#[tokio::test]
async fn test_tool_shell_invalid_command() {
    let tool = ShellTool::new();
    let context = SandboxContext {
        working_dir: "/tmp".to_string(),
        environment: vec![],
        allowed_paths: vec![],
    };

    let params = json!({
        "command": ""
    });

    let result = tool.execute(params, &context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tool_shell_command_too_long() {
    let tool = ShellTool::new();
    let context = SandboxContext {
        working_dir: "/tmp".to_string(),
        environment: vec![],
        allowed_paths: vec![],
    };

    let long_command = "a".repeat(2000);
    let params = json!({
        "command": long_command
    });

    let result = tool.execute(params, &context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tool_shell_shell_metacharacters() {
    let tool = ShellTool::new();
    let context = SandboxContext {
        working_dir: "/tmp".to_string(),
        environment: vec![],
        allowed_paths: vec![],
    };

    let params = json!({
        "command": "echo hello; rm -rf /"
    });

    let result = tool.execute(params, &context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tool_shell_with_timeout() {
    let tool = ShellTool::with_timeout(60);
    // Verify tool was created successfully
    assert_eq!(tool.tool_name(), "shell");
}
