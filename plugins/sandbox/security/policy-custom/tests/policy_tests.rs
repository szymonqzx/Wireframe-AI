//! Tests for the custom security policy plugin.

use agentic_sdk::plugins::sandbox::SecurityPolicy;
use agentic_sdk::Plugin;
use policy_custom::CustomPolicy;

#[tokio::test]
async fn test_policy_custom_plugin_id() {
    let policy = CustomPolicy::new();
    assert_eq!(policy.plugin_id(), "policy-custom");
}

#[tokio::test]
async fn test_policy_custom_validate_command_safe() {
    let policy = CustomPolicy::new();
    let result = policy.validate_command("ls -la", "/tmp").await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_policy_custom_validate_command_dangerous() {
    let policy = CustomPolicy::new();
    let result = policy.validate_command("rm -rf /", "/tmp").await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_policy_custom_validate_network_blocked() {
    let policy = CustomPolicy::with_network_blocked(true);
    let result = policy.validate_network_access("https://example.com").await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}
