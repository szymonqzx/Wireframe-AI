//! Wireframe-AI Webhook Receiver & Dispatcher
//!
//! Provides both an HTTP ingress and a NATS module for webhook processing.
//!
//! HTTP Server:
//!   - POST /webhook/:source  — receive external webhooks
//!   - Validates HMAC-SHA256 signatures when a secret is configured
//!   - Publishes valid payloads to NATS `webhook.receive`
//!
//! NATS Module:
//!   - Subscribes to: webhook.receive, webhook.configure, webhook.status
//!   - Publishes to: webhook.processed, webhook.failed, webhook.delivered

use agentic_sdk::{Envelope, Module};
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

struct WebhookModule {
    endpoints: Arc<RwLock<HashMap<String, WebhookEndpoint>>>,
    delivery_log: Arc<RwLock<Vec<DeliveryRecord>>>,
}

#[derive(Clone, Deserialize)]
struct WebhookEndpoint {
    source: String,
    #[serde(skip)]
    secret: Option<String>,
    target_topic: String,
    enabled: bool,
}

impl std::fmt::Debug for WebhookEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebhookEndpoint")
            .field("source", &self.source)
            .field("secret", &self.secret.as_ref().map(|_| "[REDACTED]"))
            .field("target_topic", &self.target_topic)
            .field("enabled", &self.enabled)
            .finish()
    }
}

impl serde::Serialize for WebhookEndpoint {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("WebhookEndpoint", 3)?;
        state.serialize_field("source", &self.source)?;
        state.serialize_field("target_topic", &self.target_topic)?;
        state.serialize_field("enabled", &self.enabled)?;
        state.end()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct DeliveryRecord {
    source: String,
    received_at: i64,
    processed_at: i64,
    success: bool,
    error: Option<String>,
}

#[agentic_sdk::module(
    subscribes = ["webhook.receive", "webhook.configure", "webhook.status"],
    publishes  = ["webhook.processed", "webhook.failed", "webhook.delivered"],
    queue_group = "webhook_handler"
)]
impl Module for WebhookModule {
    async fn handle(&mut self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        match env.topic.as_str() {
            "webhook.receive" => self.handle_receive(env).await,
            "webhook.configure" => self.handle_configure(env).await,
            "webhook.status" => self.handle_status(env).await,
            _ => vec![],
        }
    }
}

impl WebhookModule {
    async fn handle_receive(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let payload = &env.payload;
        let source = payload
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let endpoints = self.endpoints.read().await;
        let endpoint = match endpoints.get(&source) {
            Some(ep) if ep.enabled => ep.clone(),
            _ => {
                tracing::warn!(source, "no configured webhook endpoint");
                let error = serde_json::json!({
                    "source": source,
                    "error": "endpoint_not_configured",
                    "received_at": chrono::Utc::now().timestamp(),
                });
                return vec![env.reply("webhook.failed", error)];
            }
        };
        drop(endpoints);

        // Validate signature if secret is configured.
        if let Some(secret) = &endpoint.secret {
            let signature = payload.get("signature").and_then(|v| v.as_str());
            let body = payload.get("body").and_then(|v| v.as_str());
            if signature.is_none() {
                tracing::warn!(source, "signature missing");
                let error = serde_json::json!({
                    "source": source,
                    "error": "missing_signature",
                    "received_at": chrono::Utc::now().timestamp(),
                });
                return vec![env.reply("webhook.failed", error)];
            }
            if body.is_none() {
                tracing::warn!(source, "body missing; cannot verify signature");
                let error = serde_json::json!({
                    "source": source,
                    "error": "missing_body_for_signature_verification",
                    "received_at": chrono::Utc::now().timestamp(),
                });
                return vec![env.reply("webhook.failed", error)];
            }
            if !verify_signature(body.unwrap().as_bytes(), secret, signature.unwrap()) {
                tracing::warn!(source, "signature verification failed");
                let error = serde_json::json!({
                    "source": source,
                    "error": "invalid_signature",
                    "received_at": chrono::Utc::now().timestamp(),
                });
                return vec![env.reply("webhook.failed", error)];
            }
        }

        let now = chrono::Utc::now().timestamp();
        let normalized = serde_json::json!({
            "original": payload,
            "source": source,
            "received_at": now,
            "target_topic": endpoint.target_topic,
        });

        let record = DeliveryRecord {
            source: source.clone(),
            received_at: now,
            processed_at: now,
            success: true,
            error: None,
        };

        {
            let mut log = self.delivery_log.write().await;
            log.push(record);
        }

        tracing::info!(source, topic = %endpoint.target_topic, "webhook processed");

        vec![
            env.reply("webhook.processed", normalized.clone()),
            Envelope::new(&endpoint.target_topic, normalized, Some(env.session_id)),
        ]
    }

    async fn handle_configure(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let payload = &env.payload;
        let source = payload.get("source").and_then(|v| v.as_str()).unwrap_or("");
        let target_topic = payload
            .get("target_topic")
            .and_then(|v| v.as_str())
            .unwrap_or("webhook.processed")
            .to_string();
        let secret = payload
            .get("secret")
            .and_then(|v| v.as_str())
            .map(String::from);
        let enabled = payload
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if source.is_empty() {
            return vec![env.reply(
                "webhook.failed",
                serde_json::json!({
                    "error": "missing_source"
                }),
            )];
        }

        // Prevent self-referential loops: the module must never publish to a
        // topic it also subscribes to. Otherwise a received webhook would trigger
        // this handler again, causing an infinite NATS message storm.
        const RESERVED_TOPICS: &[&str] =
            &["webhook.receive", "webhook.configure", "webhook.status"];
        if RESERVED_TOPICS.contains(&target_topic.as_str()) {
            tracing::warn!(
                source,
                target_topic,
                "rejected endpoint configuration: target topic would create a loop"
            );
            return vec![env.reply("webhook.failed", serde_json::json!({
                "error": "invalid_target_topic",
                "message": format!("target_topic '{}' is reserved by the webhook module and cannot be used as a destination", target_topic),
            }))];
        }

        let mut endpoints = self.endpoints.write().await;
        endpoints.insert(
            source.to_string(),
            WebhookEndpoint {
                source: source.to_string(),
                secret,
                target_topic,
                enabled,
            },
        );
        drop(endpoints);

        tracing::info!(source, "webhook endpoint configured");

        vec![env.reply(
            "webhook.delivered",
            serde_json::json!({
                "status": "configured",
                "source": source,
            }),
        )]
    }

    async fn handle_status(&self, _env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let endpoints = self.endpoints.read().await;
        let log = self.delivery_log.read().await;

        let status = serde_json::json!({
            "endpoints": endpoints.len(),
            "total_deliveries": log.len(),
            "successful_deliveries": log.iter().filter(|r| r.success).count(),
            "failed_deliveries": log.iter().filter(|r| !r.success).count(),
        });

        vec![Envelope::new("webhook.delivered", status, None)]
    }
}

/// Query parameters for the HTTP webhook handler.
#[derive(Deserialize)]
struct WebhookQuery {
    #[serde(default)]
    signature: String,
}

/// Shared state for the HTTP server.
struct HttpState {
    endpoints: Arc<RwLock<HashMap<String, WebhookEndpoint>>>,
    nats_client: async_nats::Client,
}

/// Receive an external webhook via HTTP.
/// Validates the signature if the source endpoint has a secret configured,
/// then publishes the payload to NATS `webhook.receive` for downstream processing.
async fn http_webhook_handler(
    Path(source): Path<String>,
    Query(query): Query<WebhookQuery>,
    State(state): State<Arc<HttpState>>,
    body: Bytes,
) -> impl IntoResponse {
    let body_str = String::from_utf8_lossy(&body);
    let now = chrono::Utc::now().timestamp();

    // Look up endpoint configuration.
    let endpoint = {
        let eps = state.endpoints.read().await;
        eps.get(&source).cloned()
    };

    if let Some(ep) = &endpoint {
        if !ep.enabled {
            tracing::warn!(source, "webhook rejected: endpoint disabled");
            return (StatusCode::FORBIDDEN, "Endpoint disabled");
        }
        if let Some(secret) = &ep.secret {
            if query.signature.is_empty() {
                tracing::warn!(
                    source,
                    "webhook rejected: missing signature query parameter"
                );
                return (StatusCode::BAD_REQUEST, "Missing signature query parameter");
            }
            if !verify_signature(&body, secret, &query.signature) {
                tracing::warn!(source, "webhook rejected: invalid signature");
                return (StatusCode::UNAUTHORIZED, "Invalid signature");
            }
        }
    }

    // Build normalized payload — never include the endpoint secret.
    let sanitized_endpoint = endpoint.as_ref().map(|ep| {
        serde_json::json!({
            "source": &ep.source,
            "target_topic": &ep.target_topic,
            "enabled": ep.enabled,
        })
    });
    let payload = serde_json::json!({
        "source": source,
        "body": body_str.as_ref(),
        "received_at": now,
        "endpoint": sanitized_endpoint,
    });

    let payload_vec = match serde_json::to_vec(&payload) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "failed to serialize webhook payload");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Serialization error");
        }
    };

    if let Err(e) = state
        .nats_client
        .publish("webhook.receive".to_string(), payload_vec.into())
        .await
    {
        tracing::error!(error = %e, "failed to publish webhook to NATS");
        return (StatusCode::BAD_GATEWAY, "Failed to publish to NATS");
    }

    tracing::info!(source, bytes = body.len(), "webhook ingested via HTTP");
    (StatusCode::ACCEPTED, "Accepted")
}

/// Verify an HMAC-SHA256 hex signature over raw body bytes.
fn verify_signature(body: &[u8], secret: &str, signature: &str) -> bool {
    if secret.is_empty() || signature.is_empty() {
        tracing::warn!("webhook signature or secret is empty; rejecting");
        return false;
    }

    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body);

    let decoded = match hex::decode(signature) {
        Ok(d) => d,
        Err(_) => {
            tracing::warn!("webhook signature is not valid hex");
            return false;
        }
    };

    match mac.verify_slice(&decoded) {
        Ok(_) => true,
        Err(_) => {
            tracing::warn!("webhook signature mismatch");
            false
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let nats_url = std::env::var("WIREFRAME_AI_NATS_URL")
        .unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let http_port: u16 = std::env::var("WIREFRAME_AI_WEBHOOK_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);

    let nats_client = async_nats::connect(&nats_url).await?;
    let endpoints: Arc<RwLock<HashMap<String, WebhookEndpoint>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let delivery_log: Arc<RwLock<Vec<DeliveryRecord>>> = Arc::new(RwLock::new(vec![]));

    // Clone for HTTP server.
    let http_state = Arc::new(HttpState {
        endpoints: Arc::clone(&endpoints),
        nats_client: nats_client.clone(),
    });

    // Start HTTP server.
    let app = Router::new()
        .route("/webhook/{source}", post(http_webhook_handler))
        .with_state(http_state);
    let addr: SocketAddr = ([0, 0, 0, 0], http_port).into();
    let server = tokio::spawn(async move {
        tracing::info!("HTTP webhook server listening on {}", addr);
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    // Run NATS module.
    let module = WebhookModule {
        endpoints,
        delivery_log,
    };

    let module_fut = module.run(&nats_url);

    tokio::select! {
        _ = server => {},
        res = module_fut => {
            if let Err(e) = res {
                tracing::error!("Module exited with error: {}", e);
            }
        }
    }

    Ok(())
}
