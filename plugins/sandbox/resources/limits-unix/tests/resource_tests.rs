//! Tests for the Unix resource limiter plugin.

use agentic_sdk::plugins::sandbox::ResourceLimiter;
use agentic_sdk::Plugin;
use limits_unix::UnixResourceLimiter;
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_limits_unix_plugin_id() {
    let limiter = UnixResourceLimiter::new();
    assert_eq!(limiter.plugin_id(), "limits-unix");
}

#[tokio::test]
async fn test_limits_unix_check_cpu_limit() {
    let limiter = UnixResourceLimiter::new().with_cpu_limit(60);
    let usage = Duration::from_secs(30);
    let result = limiter.check_cpu_limit(usage).await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_limits_unix_check_cpu_limit_exceeded() {
    let limiter = UnixResourceLimiter::new().with_cpu_limit(60);
    let usage = Duration::from_secs(90);
    let result = limiter.check_cpu_limit(usage).await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_limits_unix_check_memory_limit() {
    let limiter = UnixResourceLimiter::new().with_memory_limit(1024);
    let usage = 512 * 1024 * 1024; // 512MB
    let result = limiter.check_memory_limit(usage).await;
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[tokio::test]
async fn test_limits_unix_check_memory_limit_exceeded() {
    let limiter = UnixResourceLimiter::new().with_memory_limit(1024);
    let usage = 2 * 1024 * 1024 * 1024; // 2GB
    let result = limiter.check_memory_limit(usage).await;
    assert!(result.is_ok());
    assert!(!result.unwrap());
}

#[tokio::test]
async fn test_limits_unix_enforce_timeout() {
    let limiter = UnixResourceLimiter::new().with_timeout(30);
    let started_at = Instant::now();
    let timeout = Duration::from_secs(60);
    let result = limiter.enforce_timeout(started_at, timeout).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_limits_unix_enforce_timeout_exceeded() {
    let limiter = UnixResourceLimiter::new().with_timeout(30);
    let started_at = Instant::now() - Duration::from_secs(60);
    let timeout = Duration::from_secs(30);
    let result = limiter.enforce_timeout(started_at, timeout).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_limits_unix_apply_rlimits() {
    let limiter = UnixResourceLimiter::new();
    // This should not panic on Unix, no-op on other platforms
    let result = limiter.apply_rlimits();
    #[cfg(unix)]
    assert!(result.is_ok());
    #[cfg(not(unix))]
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_limits_unix_custom_limits() {
    let limiter = UnixResourceLimiter::new()
        .with_cpu_limit(120)
        .with_memory_limit(2048)
        .with_timeout(60);

    let cpu_result = limiter.check_cpu_limit(Duration::from_secs(90)).await;
    assert!(cpu_result.unwrap());

    let mem_result = limiter.check_memory_limit(1024 * 1024 * 1024).await;
    assert!(mem_result.unwrap());
}
