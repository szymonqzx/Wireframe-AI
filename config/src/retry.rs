//! Retry logic for transient failures
//!
//! Provides exponential backoff retry logic for operations that may fail
//! transiently (e.g., NATS connection issues, temporary network glitches).

use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Maximum delay between retries
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            max_delay: Duration::from_secs(5),
        }
    }
}

/// Retry an operation with exponential backoff
///
/// # Arguments
///
/// * `config` - Retry configuration
/// * `operation` - Async operation to retry
///
/// # Returns
///
/// * `Ok(T)` - Operation succeeded
/// * `Err(anyhow::Error)` - Operation failed after all retries
///
/// # Example
///
/// ```rust
/// use std::io;
/// use wireframe_config::retry::{retry_with_backoff, RetryConfig};
///
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let config = RetryConfig::default();
/// let result = retry_with_backoff(config, || async {
///     // Operation that may fail transiently
///     Ok::<_, io::Error>(42)
/// }).await.unwrap();
/// assert_eq!(result, 42);
/// # });
/// ```
pub async fn retry_with_backoff<F, Fut, T, E>(config: RetryConfig, mut operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut delay = config.initial_delay;
    let mut last_error = None;

    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if attempt < config.max_attempts {
                    tracing::warn!(
                        attempt = attempt,
                        max_attempts = config.max_attempts,
                        delay_ms = delay.as_millis(),
                        "Operation failed, retrying in {}ms",
                        delay.as_millis()
                    );
                    sleep(delay).await;
                    delay = std::cmp::min(
                        Duration::from_millis(
                            (delay.as_millis() as f64 * config.backoff_multiplier) as u64,
                        ),
                        config.max_delay,
                    );
                }
            }
        }
    }

    Err(anyhow::anyhow!(
        "Operation failed after {} attempts: {}",
        config.max_attempts,
        last_error.unwrap()
    ))
}

/// Retry a NATS operation with default configuration
pub async fn retry_nats_operation<F, Fut, T, E>(operation: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::error::Error + Send + Sync + 'static,
{
    let config = RetryConfig {
        max_attempts: 5,
        initial_delay: Duration::from_millis(50),
        backoff_multiplier: 1.5,
        max_delay: Duration::from_secs(2),
    };
    retry_with_backoff(config, operation).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_with_backoff_success() {
        let config = RetryConfig::default();
        let result = retry_with_backoff(config, || async { Ok::<u32, std::io::Error>(42) }).await;

        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_retry_success() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1),
            backoff_multiplier: 1.0,
            max_delay: Duration::from_millis(5),
        };
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let result = retry_with_backoff(config, move || {
            let attempts_clone = attempts_clone.clone();
            async move {
                let current = attempts_clone.fetch_add(1, Ordering::SeqCst);
                if current < 2 {
                    Err(std::io::Error::new(std::io::ErrorKind::Other, "failure"))
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_failure() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_delay: Duration::from_millis(1),
            backoff_multiplier: 1.0,
            max_delay: Duration::from_millis(5),
        };

        let result = retry_with_backoff(config, || async {
            Err::<u32, _>(std::io::Error::new(
                std::io::ErrorKind::Other,
                "permanent failure",
            ))
        })
        .await;

        let err = result.unwrap_err().to_string();
        assert!(err.contains("Operation failed after 2 attempts"));
        assert!(err.contains("permanent failure"));
    }
}
