//! Wireframe-AI Provider Router
//!
//! Routes LLM requests to the optimal provider based on cost, latency,
//! capability requirements, and user preferences. Supports load balancing
//! and automatic fallback.
//!
//! Subscribes to: provider.route.request, provider.route.config
//! Publishes to: provider.route.response, provider.route.recommendation

use agentic_sdk::{Envelope, Module};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use wireframe_provider_core::discovery::ProviderDiscoveryRegistry;
use wireframe_provider_core::marketplace::{built_in_index, MarketplaceIndex};
use wireframe_provider_core::{CostTracker, UsageCost};

struct ProviderRouterModule {
    registry: Arc<RwLock<ProviderDiscoveryRegistry>>,
    cost_tracker: Arc<RwLock<CostTracker>>,
    routing_table: Arc<RwLock<RoutingTable>>,
    marketplace: Arc<RwLock<MarketplaceIndex>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RoutingTable {
    default_provider: String,
    fallbacks: Vec<String>,
    rules: Vec<RoutingRule>,
    cost_optimization: bool,
    latency_optimization: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RoutingRule {
    condition: RoutingCondition,
    target_provider: String,
    priority: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum RoutingCondition {
    RequiredCapability { capability: String },
    ModelName { model: String },
    MaxCostCents { max_cost: u64 },
    PreferredTransport { transport: String },
    TenantId { tenant: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RouteRequest {
    required_capabilities: Vec<String>,
    preferred_model: Option<String>,
    max_cost_cents: Option<u64>,
    preferred_transport: Option<String>,
    tenant_id: Option<String>,
    prompt_tokens_estimate: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct RouteResponse {
    provider: String,
    model: String,
    estimated_cost_cents: u64,
    estimated_latency_ms: u64,
    fallback_chain: Vec<String>,
    reason: String,
}

#[agentic_sdk::module(
    subscribes = ["provider.route.request", "provider.route.config", "provider.usage.report"],
    publishes  = ["provider.route.response", "provider.route.recommendation"],
    queue_group = "provider_router"
)]
impl Module for ProviderRouterModule {
    async fn handle(&mut self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        match env.topic.as_str() {
            "provider.route.request" => self.handle_route_request(env).await,
            "provider.route.config" => self.handle_config(env).await,
            "provider.usage.report" => self.handle_usage_report(env).await,
            _ => vec![],
        }
    }
}

impl ProviderRouterModule {
    async fn handle_route_request(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let request: RouteRequest = match serde_json::from_value(env.payload.clone()) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(error = %e, "failed to parse route request");
                return vec![env.reply(
                    "provider.route.response",
                    serde_json::json!({
                        "error": "invalid_request",
                        "message": e.to_string(),
                    }),
                )];
            }
        };

        let table = self.routing_table.read().await;
        let registry = self.registry.read().await;
        let marketplace = self.marketplace.read().await;

        // Sort rules by priority (highest first)
        let mut sorted_rules: Vec<&RoutingRule> = table.rules.iter().collect();
        sorted_rules.sort_by_key(|r| std::cmp::Reverse(r.priority));

        // 1. Apply routing rules
        let mut selected: Option<(String, String, u64, String)> = None;
        for rule in sorted_rules {
            if Self::matches_rule(&rule.condition, &request)
                && registry.is_available(&rule.target_provider)
            {
                let cost = Self::estimate_cost(
                    &registry,
                    &rule.target_provider,
                    request.prompt_tokens_estimate,
                );
                let model = registry
                    .get(&rule.target_provider)
                    .and_then(|p| p.available_models().into_iter().next())
                    .or_else(|| {
                        marketplace
                            .get(&rule.target_provider)
                            .and_then(|pkg| pkg.supported_models.first().cloned())
                    })
                    .unwrap_or_else(|| "default".to_string());
                selected = Some((
                    rule.target_provider.clone(),
                    model,
                    cost,
                    format!(
                        "matched rule: {:?} (priority {})",
                        rule.condition, rule.priority
                    ),
                ));
                break;
            }
        }

        // 2. Fallback to default with cost optimization
        if selected.is_none() {
            let default = &table.default_provider;
            if registry.is_available(default) {
                let cost = Self::estimate_cost(&registry, default, request.prompt_tokens_estimate);
                let model = registry
                    .get(default)
                    .and_then(|p| p.available_models().into_iter().next())
                    .or_else(|| {
                        marketplace
                            .get(default)
                            .and_then(|pkg| pkg.supported_models.first().cloned())
                    })
                    .unwrap_or_else(|| "default".to_string());
                selected = Some((default.clone(), model, cost, "default provider".to_string()));
            }
        }

        // 3. Build fallback chain, optionally sorting by cost when optimization is enabled.
        let mut fallback_chain = table.fallbacks.clone();
        if table.cost_optimization {
            let tracker = self.cost_tracker.read().await;
            fallback_chain.sort_by_key(|name| {
                tracker
                    .get_provider_summary(name)
                    .map(|s| s.total_cost_cents)
                    .unwrap_or(0)
            });
            drop(tracker);
        }
        drop(table);
        drop(registry);

        let response = if let Some((provider, model, cost, reason)) = selected {
            RouteResponse {
                provider,
                model,
                estimated_cost_cents: cost,
                estimated_latency_ms: 500, // placeholder
                fallback_chain,
                reason,
            }
        } else {
            RouteResponse {
                provider: "none".to_string(),
                model: "none".to_string(),
                estimated_cost_cents: 0,
                estimated_latency_ms: 0,
                fallback_chain: vec![],
                reason: "no provider available".to_string(),
            }
        };

        let value = match serde_json::to_value(&response) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "serialization failed");
                return vec![];
            }
        };

        tracing::info!(
            provider = %response.provider,
            cost = response.estimated_cost_cents,
            "route selected"
        );

        vec![env.reply("provider.route.response", value)]
    }

    fn matches_rule(condition: &RoutingCondition, request: &RouteRequest) -> bool {
        match condition {
            RoutingCondition::RequiredCapability { capability } => {
                request.required_capabilities.contains(capability)
            }
            RoutingCondition::ModelName { model } => {
                request.preferred_model.as_ref() == Some(model)
            }
            RoutingCondition::MaxCostCents { max_cost } => match request.max_cost_cents {
                Some(c) => c <= *max_cost,
                None => true,
            },
            RoutingCondition::PreferredTransport { transport } => {
                request.preferred_transport.as_ref() == Some(transport)
            }
            RoutingCondition::TenantId { tenant } => request.tenant_id.as_ref() == Some(tenant),
        }
    }

    fn estimate_cost(
        registry: &ProviderDiscoveryRegistry,
        provider_name: &str,
        prompt_tokens: Option<usize>,
    ) -> u64 {
        let tokens = prompt_tokens.unwrap_or(1000) as u64;
        let cost_per_1k = registry
            .get(provider_name)
            .and_then(|p| p.cost_per_1k_tokens())
            .map(|(prompt, _completion)| prompt)
            .unwrap_or_else(|| {
                // Fallback heuristic based on metadata when provider instance
                // does not report cost (e.g., Ollama local models).
                registry
                    .metadata(provider_name)
                    .map(|m| match m.provider_id.as_str() {
                        "openai" => 50,
                        "anthropic" => 30,
                        "ollama" => 0,
                        "google" => 25,
                        "cohere" => 20,
                        _ => 50,
                    })
                    .unwrap_or(50)
            });
        cost_per_1k * tokens / 1000
    }

    async fn handle_config(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let payload = &env.payload;

        let default_provider = payload
            .get("default_provider")
            .and_then(|v| v.as_str())
            .unwrap_or("openai")
            .to_string();
        let fallbacks: Vec<String> = payload
            .get("fallbacks")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();
        let cost_optimization = payload
            .get("cost_optimization")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let latency_optimization = payload
            .get("latency_optimization")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let mut table = self.routing_table.write().await;
        table.default_provider = default_provider;
        table.fallbacks = fallbacks;
        table.cost_optimization = cost_optimization;
        table.latency_optimization = latency_optimization;
        drop(table);

        tracing::info!("routing table updated");

        vec![env.reply(
            "provider.route.recommendation",
            serde_json::json!({
                "status": "configured",
            }),
        )]
    }

    async fn handle_usage_report(&self, env: Envelope<Value>) -> Vec<Envelope<Value>> {
        let payload = &env.payload;
        let provider = payload
            .get("provider")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let cost_cents = payload
            .get("cost_cents")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let prompt_tokens = payload
            .get("prompt_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let completion_tokens = payload
            .get("completion_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let mut tracker = self.cost_tracker.write().await;
        tracker.record(
            provider,
            UsageCost {
                prompt_tokens,
                completion_tokens,
                total_tokens: prompt_tokens + completion_tokens,
                cost_cents: Some(cost_cents),
                metadata: serde_json::Map::new().into_iter().collect(),
            },
        );
        let total = tracker.total_cost_cents();
        drop(tracker);

        tracing::info!(provider, cost_cents, total, "usage reported and recorded");

        vec![]
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let mut registry = ProviderDiscoveryRegistry::new();
    let discovered = registry.discover_from_env();
    if !discovered.is_empty() {
        tracing::info!(count = discovered.len(), providers = ?discovered.iter().map(|(n, _)| n).collect::<Vec<_>>(), "discovered providers from environment");
    } else {
        tracing::warn!("no provider API keys found in environment; router will default to marketplace metadata");
    }

    let module = ProviderRouterModule {
        registry: Arc::new(RwLock::new(registry)),
        cost_tracker: Arc::new(RwLock::new(CostTracker::new())),
        routing_table: Arc::new(RwLock::new(RoutingTable {
            default_provider: "openai".to_string(),
            fallbacks: vec!["anthropic".to_string(), "ollama".to_string()],
            rules: vec![],
            cost_optimization: true,
            latency_optimization: false,
        })),
        marketplace: Arc::new(RwLock::new(built_in_index())),
    };

    module.run("nats://localhost:4222").await
}
