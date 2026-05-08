//! Module trait — the interface contract every Wireframe AI module implements.
//!
//! Any module — Rust, Python, or anything else — must be wrappable in this
//! three-method interface. The SDK enforces it. This is what makes third-party
//! modules interchangeable.

use crate::envelope::Envelope;
use async_nats::Client;
use serde_json::Value;
use tracing::info;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

type MessageBufferInner = Vec<(String, Vec<u8>)>;

/// The Module trait — every Wireframe AI module implements this.
///
/// - `subscribes()` — which NATS topics this module listens to
/// - `publishes()` — which NATS topics this module may emit
/// - `handle()` — process an incoming message and return responses
#[async_trait::async_trait]
pub trait Module {
    /// Topics this module subscribes to.
    fn subscribes() -> &'static [&'static str];

    /// Topics this module publishes.
    fn publishes() -> &'static [&'static str];

    /// Process an incoming envelope and return response envelopes.
    async fn handle(&mut self, env: Envelope<Value>) -> Vec<Envelope<Value>>;
}

/// Announce this module on the NATS bus (sys.module.online).
pub async fn announce_online(
    nc: &Client,
    module_id: &str,
    version: &str,
    subscribes: &[&str],
    publishes: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    announce_online_with_selfdev(
        nc, module_id, version, subscribes, publishes, false, None, None,
    )
    .await
    .map_err(|e| e.into())
}

/// Announce this module on the NATS bus with selfdev capability flags.
#[allow(clippy::too_many_arguments)]
pub async fn announce_online_with_selfdev(
    nc: &Client,
    module_id: &str,
    version: &str,
    subscribes: &[&str],
    publishes: &[&str],
    selfdev_capable: bool,
    source_root: Option<&str>,
    binary_path: Option<&str>,
) -> anyhow::Result<()> {
    let mut payload = serde_json::json!({
        "module_id": module_id,
        "version": version,
        "subscribes": subscribes,
        "publishes": publishes,
        "selfdev_capable": selfdev_capable,
    });

    if let Some(src) = source_root {
        payload["source_root"] = serde_json::Value::String(src.to_string());
    }

    if let Some(bin) = binary_path {
        payload["binary_path"] = serde_json::Value::String(bin.to_string());
    }

    let env = Envelope::new("sys.module.online", payload, None);
    let data = env.to_bytes()?;
    nc.publish("sys.module.online", data.into()).await?;
    info!(module_id, selfdev_capable, "announced online");
    Ok(())
}

/// Start a periodic heartbeat task that publishes sys.module.heartbeat
/// every `interval_secs` seconds. Returns a handle that can be aborted.
pub fn start_heartbeat(
    nc: &Client,
    module_id: &str,
    interval_secs: u64,
) -> tokio::task::JoinHandle<()> {
    let nc = nc.clone();
    let module_id = module_id.to_string();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));
        loop {
            interval.tick().await;
            let payload = serde_json::json!({
                "module_id": module_id,
                "ts": chrono::Utc::now().timestamp(),
            });
            if let Ok(data) = serde_json::to_vec(&payload) {
                let _ = nc.publish("sys.module.heartbeat", data.into()).await;
            }
        }
    })
}

/// Announce this module is going offline (sys.module.offline).
pub async fn announce_offline(
    nc: &Client,
    module_id: &str,
    version: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = serde_json::json!({
        "module_id": module_id,
        "version": version,
    });
    let env = Envelope::new("sys.module.offline", payload, None);
    let data = env.to_bytes()?;
    nc.publish("sys.module.offline", data.into()).await?;
    info!(module_id, "announced offline");
    Ok(())
}

/// Publish an error event to sys.module.error.
/// Allows modules to report malformed payloads or other issues.
pub async fn publish_error(
    nc: &Client,
    module_id: &str,
    error_code: &str,
    error_message: &str,
    correlation_id: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let payload = serde_json::json!({
        "module_id": module_id,
        "error_code": error_code,
        "error_message": error_message,
        "correlation_id": correlation_id,
        "ts": chrono::Utc::now().timestamp(),
    });
    let env = Envelope::new("sys.module.error", payload, None);
    let data = env.to_bytes()?;
    nc.publish("sys.module.error", data.into()).await?;
    tracing::warn!(module_id, error_code, "error event published");
    Ok(())
}

/// Publish multiple error events in batch for better performance.
/// Uses a single timestamp for all errors to reduce system calls.
#[inline]
pub async fn publish_errors_batch(
    nc: &Client,
    errors: Vec<(String, String, String)>,
) -> Result<(), Box<dyn std::error::Error>> {
    if errors.is_empty() {
        return Ok(());
    }

    let timestamp = chrono::Utc::now().timestamp();

    // Publish all errors concurrently using join_all for better efficiency
    let mut futures = Vec::new();
    let mut serialization_errors = Vec::new();

    for (module_id, error_code, error_message) in errors {
        let payload = serde_json::json!({
            "module_id": module_id,
            "error_code": error_code,
            "error_message": error_message,
            "ts": timestamp,
        });
        let env = Envelope::new("sys.module.error", payload, None);
        match env.to_bytes() {
            Ok(data) => {
                futures.push(nc.publish("sys.module.error", data.into()));
            }
            Err(e) => {
                tracing::error!("Failed to serialize error for module {}: {}", module_id, e);
                serialization_errors.push(e);
            }
        }
    }

    // Wait for all publishes to complete and log any errors
    let results = futures::future::join_all(futures).await;
    let mut publish_errors = Vec::new();
    for result in results {
        if let Err(e) = result {
            tracing::error!("Failed to publish error message: {}", e);
            publish_errors.push(e);
        }
    }

    // Return error if any serialization or publish errors occurred
    if !serialization_errors.is_empty() || !publish_errors.is_empty() {
        return Err(format!(
            "Failed to publish errors: {} serialization errors, {} publish errors",
            serialization_errors.len(),
            publish_errors.len()
        ).into());
    }

    Ok(())
}

/// Publish multiple envelopes to NATS in batch for better performance.
/// Optimized to reduce overhead of multiple publish calls.
#[inline]
pub async fn publish_envelopes_batch(
    nc: &Client,
    envelopes: Vec<(String, Vec<u8>)>,
) -> Result<(), Box<dyn std::error::Error>> {
    if envelopes.is_empty() {
        return Ok(());
    }

    let futures: Vec<_> = envelopes
        .into_iter()
        .map(|(subject, data)| {
            nc.publish(subject, data.into())
        })
        .collect();

    // Wait for all publishes to complete and log any errors
    let results = futures::future::join_all(futures).await;
    let mut publish_errors = Vec::new();
    for result in results {
        if let Err(e) = result {
            tracing::error!("Failed to publish envelope: {}", e);
            publish_errors.push(e);
        }
    }

    // Return error if any publish errors occurred
    if !publish_errors.is_empty() {
        return Err(format!(
            "Failed to publish {} envelopes",
            publish_errors.len()
        ).into());
    }

    Ok(())
}

/// Message buffer for batching NATS publishes with automatic flushing.
/// Reduces network round trips and improves throughput.
pub struct MessageBuffer {
    buffer: Arc<Mutex<MessageBufferInner>>,
    max_size: usize,
    #[allow(dead_code)]
    max_age: Duration,
    nc: Arc<Client>,
    flush_task: Option<tokio::task::JoinHandle<()>>,
}

impl MessageBuffer {
    /// Create a new message buffer with automatic flushing.
    pub fn new(nc: Arc<Client>, max_size: usize, max_age: Duration) -> Self {
        let buffer = Arc::new(Mutex::new(Vec::with_capacity(max_size)));
        let buffer_clone = buffer.clone();
        let nc_clone = nc.clone();

        // Start background flush task
        let flush_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(max_age);
            loop {
                interval.tick().await;
                Self::flush_buffer(&buffer_clone, &nc_clone).await;
            }
        });

        Self {
            buffer,
            max_size,
            max_age,
            nc,
            flush_task: Some(flush_task),
        }
    }

    /// Add a message to the buffer. Flushes if buffer is full.
    #[inline]
    pub async fn publish(&self, subject: String, data: Vec<u8>) {
        let mut buffer = self.buffer.lock().await;
        buffer.push((subject, data));

        if buffer.len() >= self.max_size {
            let messages = std::mem::take(&mut *buffer);
            // Release lock before publishing
            let _ = buffer;

            let nc_clone = self.nc.clone();
            tokio::spawn(async move {
                let futures: Vec<_> = messages
                    .into_iter()
                    .map(|(subject, data)| {
                        nc_clone.publish(subject, data.into())
                    })
                    .collect();

                let results = futures::future::join_all(futures).await;
                for result in results {
                    if let Err(e) = result {
                        tracing::error!("Failed to publish message from buffer: {}", e);
                    }
                }
            });
        }
    }

    /// Manually flush the buffer.
    #[inline]
    pub async fn flush(&self) {
        Self::flush_buffer(&self.buffer, &self.nc).await;
    }

    /// Flush the buffer (internal).
    async fn flush_buffer(
        buffer: &Arc<Mutex<MessageBufferInner>>,
        nc: &Arc<Client>,
    ) {
        let mut buffer = buffer.lock().await;
        Self::flush_buffer_unlocked(&mut buffer, nc).await;
    }

    /// Flush the buffer with lock already held (internal).
    async fn flush_buffer_unlocked(
        buffer: &mut MessageBufferInner,
        nc: &Arc<Client>,
    ) {
        if buffer.is_empty() {
            return;
        }

        let messages = std::mem::take(buffer);
        let _ = buffer; // Release lock before publishing

        let nc_clone = nc.clone();
        tokio::spawn(async move {
            let futures: Vec<_> = messages
                .into_iter()
                .map(|(subject, data)| {
                    nc_clone.publish(subject, data.into())
                })
                .collect();

            let results = futures::future::join_all(futures).await;
            for result in results {
                if let Err(e) = result {
                    tracing::error!("Failed to publish message from buffer: {}", e);
                }
            }
        });
    }

    /// Get current buffer size.
    #[inline]
    pub async fn size(&self) -> usize {
        self.buffer.lock().await.len()
    }

    /// Check if buffer is empty.
    #[inline]
    pub async fn is_empty(&self) -> bool {
        self.buffer.lock().await.is_empty()
    }
}

impl Drop for MessageBuffer {
    fn drop(&mut self) {
        // Flush remaining messages on drop
        let buffer = self.buffer.clone();
        let nc = self.nc.clone();
        tokio::spawn(async move {
            Self::flush_buffer(&buffer, &nc).await;
        });

        // Abort the flush task
        if let Some(task) = self.flush_task.take() {
            task.abort();
        }
    }
}

/// Optimized envelope publisher with connection reuse and batching.
pub struct EnvelopePublisher {
    nc: Arc<Client>,
    buffer: Option<MessageBuffer>,
    enable_buffering: bool,
}

impl EnvelopePublisher {
    /// Create a new envelope publisher.
    pub fn new(nc: Arc<Client>) -> Self {
        Self {
            nc,
            buffer: None,
            enable_buffering: false,
        }
    }

    /// Enable message buffering with specified parameters.
    pub fn with_buffering(mut self, max_size: usize, max_age: Duration) -> Self {
        self.buffer = Some(MessageBuffer::new(self.nc.clone(), max_size, max_age));
        self.enable_buffering = true;
        self
    }

    /// Publish an envelope immediately (bypasses buffer).
    #[inline]
    pub async fn publish_immediate<T>(&self, subject: String, envelope: &Envelope<T>) -> Result<(), Box<dyn std::error::Error>>
    where
        T: serde::Serialize,
    {
        let data = envelope.to_bytes()?;
        self.nc.publish(subject, data.into()).await?;
        Ok(())
    }

    /// Publish an envelope (uses buffer if enabled).
    #[inline]
    pub async fn publish<T>(&self, subject: String, envelope: &Envelope<T>) -> Result<(), Box<dyn std::error::Error>>
    where
        T: serde::Serialize,
    {
        let data = envelope.to_bytes()?;

        if let Some(buffer) = &self.buffer {
            buffer.publish(subject, data).await;
        } else {
            self.nc.publish(subject, data.into()).await?;
        }

        Ok(())
    }

    /// Flush the buffer if enabled.
    #[inline]
    pub async fn flush(&self) {
        if let Some(buffer) = &self.buffer {
            buffer.flush().await;
        }
    }

    /// Get buffer size if buffering is enabled.
    #[inline]
    pub async fn buffer_size(&self) -> Option<usize> {
        if let Some(buffer) = &self.buffer {
            Some(buffer.size().await)
        } else {
            None
        }
    }
}
