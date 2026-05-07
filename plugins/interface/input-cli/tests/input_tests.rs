//! Tests for the CLI input plugin.

use agentic_sdk::Plugin;
use input_cli::CliInput;

#[tokio::test]
async fn test_input_cli_plugin_id() {
    let input = CliInput::new();
    assert_eq!(input.plugin_id(), "input-cli");
}

#[tokio::test]
async fn test_input_cli_health_check() {
    let input = CliInput::new();
    let result = input.health_check().await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}
