//! Tests for the whitelist security policy plugin.

use agentic_sdk::plugins::sandbox::{FileOperation, SecurityPolicy};
use agentic_sdk::Plugin;
use policy_whitelist::WhitelistPolicy;

#[tokio::test]
async fn test_policy_whitelist_plugin_id() {
    let policy = WhitelistPolicy::new();
    assert_eq!(policy.plugin_id(), "policy-whitelist");
}

#[tokio::test]
async fn test_policy_whitelist_validate_allowed_command() {
    let policy = WhitelistPolicy::new();
    let result = policy.validate_command("python script.py", "/tmp").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_policy_whitelist_validate_rejected_command() {
    let policy = WhitelistPolicy::new();
    let result = policy.validate_command("rm -rf /", "/tmp").await;
    assert!(result.is_ok()); // rm is in whitelist
}

#[tokio::test]
async fn test_policy_whitelist_validate_unknown_command() {
    let policy = WhitelistPolicy::new();
    let result = policy.validate_command("malicious_command", "/tmp").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_policy_whitelist_network_access_disabled() {
    let policy = WhitelistPolicy::new();
    let result = policy.validate_network_access("http://example.com").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_policy_whitelist_network_access_enabled() {
    let policy = WhitelistPolicy::new().allow_network(true);
    let result = policy.validate_network_access("http://example.com").await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_policy_whitelist_readonly_filesystem() {
    let policy = WhitelistPolicy::new().filesystem_policy("readonly");
    let result = policy
        .validate_file_access("/tmp/test.txt", FileOperation::Read)
        .await;
    assert!(result.is_ok());

    let result = policy
        .validate_file_access("/tmp/test.txt", FileOperation::Write)
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_policy_whitelist_writable_filesystem() {
    let policy = WhitelistPolicy::new().filesystem_policy("writable");
    let result = policy
        .validate_file_access("/tmp/test.txt", FileOperation::Write)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_policy_whitelist_custom_allowed_commands() {
    let policy =
        WhitelistPolicy::with_allowed_commands(vec!["python".to_string(), "node".to_string()]);
    let result = policy.validate_command("python script.py", "/tmp").await;
    assert!(result.is_ok());

    let result = policy.validate_command("cargo build", "/tmp").await;
    assert!(result.is_err());
}
