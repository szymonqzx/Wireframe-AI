//! Tests for the file tool plugin.

use tool_file::FileTool;
use agentic_sdk::Plugin;
use agentic_sdk::plugins::sandbox::{Tool, SandboxContext};
use serde_json::json;

#[tokio::test]
async fn test_tool_file_input_schema() {
    let tool = FileTool::new();
    let schema = tool.input_schema();

    assert!(schema.is_object());
    let props = schema.get("properties").unwrap().as_object().unwrap();
    assert!(props.contains_key("operation"));
    assert!(props.contains_key("path"));
    assert!(props.contains_key("content"));
}

#[tokio::test]
async fn test_tool_file_plugin_id() {
    let tool = FileTool::new();
    assert_eq!(tool.plugin_id(), "tool-file");
}

#[tokio::test]
async fn test_tool_file_tool_name() {
    let tool = FileTool::new();
    assert_eq!(tool.tool_name(), "file");
}

#[tokio::test]
async fn test_tool_file_path_traversal() {
    let tool = FileTool::new();
    let context = SandboxContext {
        working_dir: "/tmp".to_string(),
        environment: vec![],
        allowed_paths: vec![],
    };

    let params = json!({
        "operation": "read",
        "path": "../../../etc/passwd"
    });

    let result = tool.execute(params, &context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tool_file_absolute_path() {
    let tool = FileTool::new();
    let context = SandboxContext {
        working_dir: "/tmp".to_string(),
        environment: vec![],
        allowed_paths: vec![],
    };

    let params = json!({
        "operation": "read",
        "path": "/etc/passwd"
    });

    let result = tool.execute(params, &context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tool_file_path_too_long() {
    let tool = FileTool::new();
    let context = SandboxContext {
        working_dir: "/tmp".to_string(),
        environment: vec![],
        allowed_paths: vec![],
    };

    let long_path = "a".repeat(5000);
    let params = json!({
        "operation": "read",
        "path": long_path
    });

    let result = tool.execute(params, &context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tool_file_unknown_operation() {
    let tool = FileTool::new();
    let context = SandboxContext {
        working_dir: "/tmp".to_string(),
        environment: vec![],
        allowed_paths: vec![],
    };

    let params = json!({
        "operation": "delete",
        "path": "test.txt"
    });

    let result = tool.execute(params, &context).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_tool_file_with_max_size() {
    let tool = FileTool::with_max_file_size(1024);
    // Verify tool was created successfully
    assert_eq!(tool.tool_name(), "file");
}
