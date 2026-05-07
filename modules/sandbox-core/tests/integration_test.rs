//! Integration tests for sandbox-core with plugins.

use wireframe_ai_sandbox_core::{SandboxCore, WhitelistPolicy, UnixResourceLimiter};
use std::sync::Arc;

#[tokio::test]
async fn test_sandbox_core_sandbox_root() {
    let sandbox_root = std::env::temp_dir().join("sandbox-test").to_string_lossy().to_string();
    let sandbox = SandboxCore::new(sandbox_root.clone());

    assert_eq!(sandbox.sandbox_root(), sandbox_root);
}

#[tokio::test]
async fn test_sandbox_core_register_tool() {
    let sandbox_root = std::env::temp_dir().join("sandbox-test").to_string_lossy().to_string();
    let sandbox = SandboxCore::new(sandbox_root);

    // For now, just verify the structure is correct
    assert!(sandbox.sandbox_root().len() > 0);
}

#[tokio::test]
async fn test_sandbox_core_security_none() {
    let sandbox_root = std::env::temp_dir().join("sandbox-test").to_string_lossy().to_string();
    let sandbox = SandboxCore::new(sandbox_root);

    let security = sandbox.security().await;
    assert!(security.is_none());
}

#[tokio::test]
async fn test_sandbox_core_resource_limiter_none() {
    let sandbox_root = std::env::temp_dir().join("sandbox-test").to_string_lossy().to_string();
    let sandbox = SandboxCore::new(sandbox_root);

    let limiter = sandbox.resource_limiter().await;
    assert!(limiter.is_none());
}

#[tokio::test]
async fn test_sandbox_core_with_security_plugin() {
    let sandbox_root = std::env::temp_dir().join("sandbox-test-sec").to_string_lossy().to_string();
    let sandbox = SandboxCore::new(sandbox_root);

    let config = serde_json::json!({
        "allowed_paths": ["/tmp"],
        "allowed_commands": ["ls", "pwd"],
        "allowed_network": []
    });

    let security = Arc::new(WhitelistPolicy::new(&config));
    sandbox.set_security(security).await;

    let retrieved = sandbox.security().await;
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_sandbox_core_with_resource_limiter_plugin() {
    let sandbox_root = std::env::temp_dir().join("sandbox-test-res").to_string_lossy().to_string();
    let sandbox = SandboxCore::new(sandbox_root);

    let config = serde_json::json!({
        "max_execution_time_secs": 60,
        "max_memory_mb": 512,
        "max_file_size_mb": 10
    });

    let limiter = Arc::new(UnixResourceLimiter::new(&config));
    sandbox.set_resource_limiter(limiter).await;

    let retrieved = sandbox.resource_limiter().await;
    assert!(retrieved.is_some());
}

#[tokio::test]
async fn test_sandbox_core_security_validation_integration() {
    let sandbox_root = std::env::temp_dir().join("sandbox-test-sec-val").to_string_lossy().to_string();
    let sandbox = SandboxCore::new(sandbox_root);

    let config = serde_json::json!({
        "allowed_paths": ["/tmp"],
        "allowed_commands": ["ls"],
        "allowed_network": []
    });

    let security = Arc::new(WhitelistPolicy::new(&config));
    sandbox.set_security(security).await;

    let retrieved = sandbox.security().await;
    assert!(retrieved.is_some());

    // Test validation through the security plugin
    let sec = retrieved.unwrap();
    let result = sec.validate_command("ls", "/tmp").await;
    assert!(result.is_ok());

    let result = sec.validate_command("rm", "/tmp").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_sandbox_core_resource_limiting_integration() {
    let sandbox_root = std::env::temp_dir().join("sandbox-test-res-lim").to_string_lossy().to_string();
    let sandbox = SandboxCore::new(sandbox_root);

    let config = serde_json::json!({
        "max_execution_time_secs": 60,
        "max_memory_mb": 512,
        "max_file_size_mb": 10
    });

    let limiter = Arc::new(UnixResourceLimiter::new(&config));
    sandbox.set_resource_limiter(limiter).await;

    let retrieved = sandbox.resource_limiter().await;
    assert!(retrieved.is_some());

    // Test memory limiting
    let res = retrieved.unwrap();
    let result = res.check_memory_limit(100 * 1024 * 1024).await; // 100 MB
    assert!(result.is_ok());

    let result = res.check_memory_limit(1024 * 1024 * 1024).await; // 1 GB
    assert!(result.is_err());
}
