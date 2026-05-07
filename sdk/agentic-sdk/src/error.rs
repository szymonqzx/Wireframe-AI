//! Error handling patterns for Wireframe AI modules.
//!
//! Provides standardized error types, retry logic with exponential backoff,
//! and conversion helpers for common failure modes in distributed agent systems.
//!
//! ## Example
//!
//! ```ignore
//! use agentic_sdk::error::{SdkError, SdkResult, retry_with_backoff};
//!
//! async fn fetch_data() -> SdkResult<String> {
//!     retry_with_backoff(3, || async {
//!         // operation that might fail
//!         Ok("data".to_string())
//!     }).await
//! }
//! ```

use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

/// Unified error type for SDK operations.
#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    #[error("NATS error: {0}")]
    Nats(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),

    #[error("Module not found: {0}")]
    ModuleNotFound(String),

    #[error("Incompatible module interface: {0}")]
    IncompatibleInterface(String),

    #[error("Timeout after {0}s")]
    Timeout(u64),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Retry exhausted after {attempts} attempts: {last_error}")]
    RetryExhausted { attempts: u32, last_error: String },

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Shorthand result type using [`SdkError`].
pub type SdkResult<T> = Result<T, SdkError>;

impl SdkError {
    /// Returns true if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        match self {
            SdkError::Nats(_) => true,
            SdkError::Timeout(_) => true,
            SdkError::RetryExhausted { .. } => false,
            SdkError::Io(e) => {
                e.kind() == std::io::ErrorKind::TimedOut
                    || e.kind() == std::io::ErrorKind::ConnectionRefused
                    || e.kind() == std::io::ErrorKind::ConnectionReset
            }
            _ => false,
        }
    }

    /// Convert from [`anyhow::Error`].
    pub fn from_anyhow(e: anyhow::Error) -> Self {
        SdkError::Unknown(e.to_string())
    }

    /// Convert from a NATS error string.
    pub fn from_nats(e: impl Into<String>) -> Self {
        SdkError::Nats(e.into())
    }

    /// Convert from serde_json error.
    pub fn from_serde(e: serde_json::Error) -> Self {
        if e.is_data() {
            SdkError::Deserialization(e.to_string())
        } else {
            SdkError::Serialization(e.to_string())
        }
    }
}

/// Retry an async operation with exponential backoff.
///
/// # Arguments
/// * `max_attempts` - Maximum number of retry attempts (including initial)
/// * `operation` - The async operation to retry
///
/// # Returns
/// * `Ok(T)` if the operation succeeds within max_attempts
/// * `Err(SdkError::RetryExhausted)` if all attempts fail
///
/// Backoff schedule: 100ms, 200ms, 400ms, 800ms, ... (capped at 5s)
pub async fn retry_with_backoff<F, Fut, T>(max_attempts: u32, operation: F) -> SdkResult<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = SdkResult<T>>,
{
    let mut last_error = None;

    for attempt in 0..max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if !e.is_retryable() || attempt == max_attempts - 1 {
                    last_error = Some(e);
                    break;
                }
                let delay_ms = (100u64 * 2u64.pow(attempt)).min(5000);
                sleep(Duration::from_millis(delay_ms)).await;
            }
        }
    }

    Err(SdkError::RetryExhausted {
        attempts: max_attempts,
        last_error: last_error.map(|e| e.to_string()).unwrap_or_default(),
    })
}

/// Retry an async operation with a fixed delay between attempts.
///
/// # Arguments
/// * `max_attempts` - Maximum number of retry attempts
/// * `delay_ms` - Fixed delay in milliseconds between attempts
/// * `operation` - The async operation to retry
pub async fn retry_with_fixed_delay<F, Fut, T>(
    max_attempts: u32,
    delay_ms: u64,
    operation: F,
) -> SdkResult<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = SdkResult<T>>,
{
    let mut last_error = None;

    for attempt in 0..max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if !e.is_retryable() || attempt == max_attempts - 1 {
                    last_error = Some(e);
                    break;
                }
                sleep(Duration::from_millis(delay_ms)).await;
            }
        }
    }

    Err(SdkError::RetryExhausted {
        attempts: max_attempts,
        last_error: last_error.map(|e| e.to_string()).unwrap_or_default(),
    })
}

/// Execute an operation with a timeout.
///
/// # Arguments
/// * `timeout_secs` - Maximum time to wait in seconds
/// * `operation` - The async operation to execute
///
/// # Returns
/// * `Ok(T)` if the operation completes within the timeout
/// * `Err(SdkError::Timeout)` if the operation exceeds the timeout
pub async fn with_timeout<F, Fut, T>(timeout_secs: u64, operation: F) -> SdkResult<T>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = SdkResult<T>>,
{
    match tokio::time::timeout(Duration::from_secs(timeout_secs), operation()).await {
        Ok(result) => result,
        Err(_) => Err(SdkError::Timeout(timeout_secs)),
    }
}

/// Convert a `Result<T, E>` to `SdkResult<T>` where `E: ToString`.
pub fn map_err<T, E>(result: Result<T, E>) -> SdkResult<T>
where
    E: ToString,
{
    result.map_err(|e| SdkError::Unknown(e.to_string()))
}

/// A wrapper that automatically retries NATS publish operations.
pub async fn nats_publish_with_retry(
    client: &async_nats::Client,
    subject: &str,
    payload: bytes::Bytes,
    max_attempts: u32,
) -> SdkResult<()> {
    retry_with_backoff(max_attempts, || async {
        client
            .publish(subject.to_string(), payload.clone())
            .await
            .map_err(|e| SdkError::from_nats(e.to_string()))?;
        Ok(())
    })
    .await
}

/// Helper for graceful error recovery in module handlers.
/// Logs the error and returns a default value.
#[macro_export]
macro_rules! try_or_default {
    ($expr:expr, $default:expr, $ctx:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                tracing::warn!(error = %e, context = $ctx, "operation failed, using default");
                $default
            }
        }
    };
}

/// Helper for graceful error recovery in module handlers.
/// Logs the error and returns an empty vector.
#[macro_export]
macro_rules! try_or_empty {
    ($expr:expr, $ctx:expr) => {
        $crate::try_or_default!($expr, Vec::new(), $ctx)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdk_error_retryable() {
        assert!(SdkError::Nats("conn refused".to_string()).is_retryable());
        assert!(SdkError::Timeout(30).is_retryable());
        assert!(!SdkError::ModuleNotFound("foo".to_string()).is_retryable());
        assert!(!SdkError::Config("bad".to_string()).is_retryable());
    }

    #[tokio::test]
    async fn test_retry_with_backoff_succeeds_first_try() {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let c = counter.clone();

        let result = retry_with_backoff(3, || async {
            c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(42)
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_retries_then_succeeds() {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let c = counter.clone();

        let result = retry_with_backoff(3, || async {
            let attempt = c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if attempt < 2 {
                Err(SdkError::Nats("transient".to_string()))
            } else {
                Ok(42)
            }
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_exhausted() {
        let result = retry_with_backoff(2, || async {
            Err::<(), _>(SdkError::Nats("persistent".to_string()))
        })
        .await;

        assert!(matches!(result, Err(SdkError::RetryExhausted { .. })));
    }

    #[tokio::test]
    async fn test_retry_non_retryable_fails_immediately() {
        let counter = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let c = counter.clone();

        let result = retry_with_backoff(5, || async {
            c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err::<(), _>(SdkError::Config("bad".to_string()))
        })
        .await;

        assert!(matches!(result, Err(SdkError::RetryExhausted { .. })));
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_with_timeout_succeeds() {
        let result = with_timeout(1, || async { Ok(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_with_timeout_fires() {
        let result = with_timeout(0, || async {
            sleep(Duration::from_secs(10)).await;
            Ok(42)
        })
        .await;

        assert!(matches!(result, Err(SdkError::Timeout(0))));
    }

    #[test]
    fn test_map_err_ok() {
        let result: Result<i32, &str> = Ok(42);
        assert_eq!(map_err(result).unwrap(), 42);
    }

    #[test]
    fn test_map_err_err() {
        let result: Result<i32, &str> = Err("bad");
        assert!(matches!(map_err(result), Err(SdkError::Unknown(_))));
    }
}
