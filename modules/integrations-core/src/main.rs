//! Wireframe-AI Integration Hub
//!
//! Bridges Wireframe-AI with external services: GitHub, Slack, databases.
//! Normalizes external APIs into Wireframe-AI message patterns.
//!
//! Subscribes to: integration.request, integration.github.>, integration.slack.>, integration.db.>
//! Publishes to: integration.response, integration.result

use agentic_sdk::{Envelope, Module};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

struct IntegrationModule {
    configs: Arc<RwLock<HashMap<String, IntegrationConfig>>>,
    http_client: reqwest::Client,
}

#[derive(Clone)]
struct IntegrationConfig {
    api_key: Option<String>,
    base_url: String,
    enabled: bool,
}

impl std::fmt::Debug for IntegrationConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IntegrationConfig")
            .field("api_key", &self.api_key.as_ref().map(|_| "[REDACTED]"))
            .field("base_url", &self.base_url)
            .field("enabled", &self.enabled)
            .finish()
    }
}

#[agentic_sdk::module(
    subscribes = [
        "integration.request",
        "integration.github.>",
        "integration.slack.>",
        "integration.db.>",
        "integration.config.set"
    ],
    publishes  = ["integration.response", "integration.result"],
    queue_group = "integration_handler"
)]
impl Module for IntegrationModule {
    async fn handle(&mut self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        match env.topic.as_str() {
            "integration.request" => self.handle_generic_request(env).await,
            t if t.starts_with("integration.github.") => self.handle_github(env).await,
            t if t.starts_with("integration.slack.") => self.handle_slack(env).await,
            t if t.starts_with("integration.db.") => self.handle_database(env).await,
            "integration.config.set" => self.handle_config_set(env).await,
            _ => vec![],
        }
    }
}

impl IntegrationModule {
    async fn handle_generic_request(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let payload = &env.payload;
        let service = payload
            .get("service")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        tracing::info!(service, "handling integration request");

        let response = serde_json::json!({
            "service": service,
            "status": "pending",
            "message": "Integration request received. Use specific topics for actions.",
            "available": ["github", "slack", "db"],
            "handled_at": chrono::Utc::now().timestamp(),
        });

        vec![env.reply("integration.response", response)]
    }

    async fn handle_github(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let payload = &env.payload;
        let action = env.topic.strip_prefix("integration.github.").unwrap_or("");

        tracing::info!(action, "github integration");

        let configs = self.configs.read().await;
        let config = match configs.get("github") {
            Some(c) if c.enabled => c.clone(),
            _ => {
                return vec![env.reply(
                    "integration.response",
                    serde_json::json!({
                        "error": "github_not_configured",
                        "action": action,
                    }),
                )];
            }
        };
        drop(configs);

        let result = match action {
            "issues.list" => {
                let owner = payload.get("owner").and_then(|v| v.as_str()).unwrap_or("");
                let repo = payload.get("repo").and_then(|v| v.as_str()).unwrap_or("");
                if owner.is_empty() || repo.is_empty() {
                    serde_json::json!({
                        "action": "issues.list",
                        "error": "missing required fields: owner and repo",
                        "status": "failed",
                    })
                } else {
                    match Self::build_github_url(
                        &config.base_url,
                        &["repos", owner, repo, "issues"],
                    ) {
                        Ok(url) => match self.github_get(&url, &config.api_key).await {
                            Ok(body) => serde_json::json!({
                                "action": "issues.list",
                                "owner": owner,
                                "repo": repo,
                                "status": "fetched",
                                "data": body,
                            }),
                            Err(e) => serde_json::json!({
                                "action": "issues.list",
                                "error": e,
                                "status": "failed",
                            }),
                        },
                        Err(e) => serde_json::json!({
                            "action": "issues.list",
                            "error": e,
                            "status": "failed",
                        }),
                    }
                }
            }
            "issues.create" => {
                let owner = payload.get("owner").and_then(|v| v.as_str()).unwrap_or("");
                let repo = payload.get("repo").and_then(|v| v.as_str()).unwrap_or("");
                let title = payload.get("title").and_then(|v| v.as_str()).unwrap_or("");
                let body_text = payload.get("body").and_then(|v| v.as_str()).unwrap_or("");
                if owner.is_empty() || repo.is_empty() || title.is_empty() {
                    serde_json::json!({
                        "action": "issues.create",
                        "error": "missing required fields: owner, repo, and title",
                        "status": "failed",
                    })
                } else {
                    match Self::build_github_url(
                        &config.base_url,
                        &["repos", owner, repo, "issues"],
                    ) {
                        Ok(url) => {
                            let req_body = serde_json::json!({ "title": title, "body": body_text });
                            match self.github_post(&url, &config.api_key, &req_body).await {
                                Ok(body) => serde_json::json!({
                                    "action": "issues.create",
                                    "status": "created",
                                    "data": body,
                                }),
                                Err(e) => serde_json::json!({
                                    "action": "issues.create",
                                    "error": e,
                                    "status": "failed",
                                }),
                            }
                        }
                        Err(e) => serde_json::json!({
                            "action": "issues.create",
                            "error": e,
                            "status": "failed",
                        }),
                    }
                }
            }
            "pr.list" => {
                let owner = payload.get("owner").and_then(|v| v.as_str()).unwrap_or("");
                let repo = payload.get("repo").and_then(|v| v.as_str()).unwrap_or("");
                if owner.is_empty() || repo.is_empty() {
                    serde_json::json!({
                        "action": "pr.list",
                        "error": "missing required fields: owner and repo",
                        "status": "failed",
                    })
                } else {
                    match Self::build_github_url(&config.base_url, &["repos", owner, repo, "pulls"])
                    {
                        Ok(url) => match self.github_get(&url, &config.api_key).await {
                            Ok(body) => serde_json::json!({
                                "action": "pr.list",
                                "status": "fetched",
                                "data": body,
                            }),
                            Err(e) => serde_json::json!({
                                "action": "pr.list",
                                "error": e,
                                "status": "failed",
                            }),
                        },
                        Err(e) => serde_json::json!({
                            "action": "pr.list",
                            "error": e,
                            "status": "failed",
                        }),
                    }
                }
            }
            _ => serde_json::json!({
                "error": "unknown_action",
                "action": action,
            }),
        };

        vec![env.reply("integration.result", result)]
    }

    async fn handle_slack(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let payload = &env.payload;
        let action = env.topic.strip_prefix("integration.slack.").unwrap_or("");

        tracing::info!(action, "slack integration");

        let configs = self.configs.read().await;
        let config = match configs.get("slack") {
            Some(c) if c.enabled => c.clone(),
            _ => {
                return vec![env.reply(
                    "integration.result",
                    serde_json::json!({
                        "error": "slack_not_configured",
                        "action": action,
                    }),
                )];
            }
        };
        drop(configs);

        let base = config.base_url.clone();
        let result = match action {
            "message.send" => {
                let channel = payload
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let text = payload.get("text").and_then(|v| v.as_str()).unwrap_or("");
                if channel.is_empty() || text.is_empty() {
                    serde_json::json!({
                        "action": "message.send",
                        "error": "missing required fields: channel and text",
                        "status": "failed",
                    })
                } else {
                    let req_body = serde_json::json!({ "channel": channel, "text": text });
                    let url = format!("{}/chat.postMessage", base);
                    match self.slack_post(&url, &config.api_key, &req_body).await {
                        Ok(body) => serde_json::json!({
                            "action": "message.send",
                            "channel": channel,
                            "status": "sent",
                            "data": body,
                        }),
                        Err(e) => serde_json::json!({
                            "action": "message.send",
                            "error": e,
                            "status": "failed",
                        }),
                    }
                }
            }
            "channels.list" => {
                let url = format!("{}/conversations.list", base);
                match self.slack_get(&url, &config.api_key).await {
                    Ok(body) => serde_json::json!({
                        "action": "channels.list",
                        "status": "fetched",
                        "data": body,
                    }),
                    Err(e) => serde_json::json!({
                        "action": "channels.list",
                        "error": e,
                        "status": "failed",
                    }),
                }
            }
            _ => serde_json::json!({
                "error": "unknown_action",
                "action": action,
            }),
        };

        vec![env.reply("integration.result", result)]
    }

    async fn handle_database(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let payload = &env.payload;
        let action = env.topic.strip_prefix("integration.db.").unwrap_or("");

        tracing::info!(action, "database integration");

        let configs = self.configs.read().await;
        let config = match configs.get("db") {
            Some(c) if c.enabled => c.clone(),
            _ => {
                return vec![env.reply(
                    "integration.result",
                    serde_json::json!({
                        "error": "db_not_configured",
                        "action": action,
                    }),
                )];
            }
        };
        drop(configs);

        let result = match action {
            "query" => {
                let sql = payload.get("sql").and_then(|v| v.as_str()).unwrap_or("");
                if sql.is_empty() {
                    serde_json::json!({
                        "action": "query",
                        "error": "missing required field: sql",
                        "status": "failed",
                    })
                } else {
                    let req_body = serde_json::json!({ "query": sql });
                    let url = format!("{}/query", config.base_url);
                    match self.db_post(&url, &config.api_key, &req_body).await {
                        Ok(body) => serde_json::json!({
                            "action": "query",
                            "status": "executed",
                            "data": body,
                        }),
                        Err(e) => serde_json::json!({
                            "action": "query",
                            "error": e,
                            "status": "failed",
                        }),
                    }
                }
            }
            "migrate" => {
                serde_json::json!({
                    "action": "migrate",
                    "status": "not_implemented",
                    "message": "Database migration requires a schema migration tool. Use integration.db.query with DDL statements instead.",
                })
            }
            _ => serde_json::json!({
                "error": "unknown_action",
                "action": action,
            }),
        };

        vec![env.reply("integration.result", result)]
    }

    async fn handle_config_set(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let payload = &env.payload;
        let service = payload
            .get("service")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let api_key = payload
            .get("api_key")
            .and_then(|v| v.as_str())
            .map(String::from);
        let base_url = payload
            .get("base_url")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let enabled = payload
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        if service.is_empty() {
            return vec![env.reply(
                "integration.response",
                serde_json::json!({
                    "error": "missing_service"
                }),
            )];
        }

        let mut configs = self.configs.write().await;
        configs.insert(
            service.to_string(),
            IntegrationConfig {
                api_key,
                base_url,
                enabled,
            },
        );
        drop(configs);

        tracing::info!(service, "integration configured");

        vec![env.reply(
            "integration.response",
            serde_json::json!({
                "status": "configured",
                "service": service,
            }),
        )]
    }

    // ── URL helpers ──────────────────────────────────────────────────────────

    /// Build a GitHub API URL with safe path-segment encoding.
    /// Prevents path-traversal injection from untrusted payload fields.
    fn build_github_url(base: &str, segments: &[&str]) -> Result<String, String> {
        let mut url = reqwest::Url::parse(base).map_err(|e| format!("invalid base_url: {}", e))?;
        {
            let mut path = url
                .path_segments_mut()
                .map_err(|_| "invalid url for path segments".to_string())?;
            for seg in segments {
                path.push(seg);
            }
        }
        Ok(url.to_string())
    }

    // ── HTTP helpers ─────────────────────────────────────────────────────────

    async fn github_get(&self, url: &str, token: &Option<String>) -> Result<Value, String> {
        let mut req = self.http_client.get(url);
        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
            req = req.header("Accept", "application/vnd.github+json");
            req = req.header("X-GitHub-Api-Version", "2022-11-28");
        }
        match req.send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                match resp.json::<Value>().await {
                    Ok(body) => Ok(serde_json::json!({ "status": status, "body": body })),
                    Err(_) => Ok(serde_json::json!({ "status": status, "body": null })),
                }
            }
            Err(e) => Err(format!("request failed: {}", e)),
        }
    }

    async fn github_post(
        &self,
        url: &str,
        token: &Option<String>,
        body: &Value,
    ) -> Result<Value, String> {
        let mut req = self.http_client.post(url).json(body);
        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
            req = req.header("Accept", "application/vnd.github+json");
            req = req.header("X-GitHub-Api-Version", "2022-11-28");
        }
        match req.send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                match resp.json::<Value>().await {
                    Ok(body) => Ok(serde_json::json!({ "status": status, "body": body })),
                    Err(_) => Ok(serde_json::json!({ "status": status, "body": null })),
                }
            }
            Err(e) => Err(format!("request failed: {}", e)),
        }
    }

    async fn slack_get(&self, url: &str, token: &Option<String>) -> Result<Value, String> {
        let mut req = self.http_client.get(url);
        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }
        match req.send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                match resp.json::<Value>().await {
                    Ok(body) => Ok(serde_json::json!({ "status": status, "body": body })),
                    Err(_) => Ok(serde_json::json!({ "status": status, "body": null })),
                }
            }
            Err(e) => Err(format!("request failed: {}", e)),
        }
    }

    async fn slack_post(
        &self,
        url: &str,
        token: &Option<String>,
        body: &Value,
    ) -> Result<Value, String> {
        let mut req = self.http_client.post(url).json(body);
        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }
        match req.send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                match resp.json::<Value>().await {
                    Ok(body) => Ok(serde_json::json!({ "status": status, "body": body })),
                    Err(_) => Ok(serde_json::json!({ "status": status, "body": null })),
                }
            }
            Err(e) => Err(format!("request failed: {}", e)),
        }
    }

    async fn db_post(
        &self,
        url: &str,
        token: &Option<String>,
        body: &Value,
    ) -> Result<Value, String> {
        let mut req = self.http_client.post(url).json(body);
        if let Some(t) = token {
            req = req.header("Authorization", format!("Bearer {}", t));
        }
        match req.send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                match resp.json::<Value>().await {
                    Ok(body) => Ok(serde_json::json!({ "status": status, "body": body })),
                    Err(_) => Ok(serde_json::json!({ "status": status, "body": null })),
                }
            }
            Err(e) => Err(format!("request failed: {}", e)),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;

    let module = IntegrationModule {
        configs: Arc::new(RwLock::new(HashMap::new())),
        http_client,
    };

    module.run("nats://localhost:4222").await
}
