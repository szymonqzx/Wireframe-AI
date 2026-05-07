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
