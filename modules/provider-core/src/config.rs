// provider-core/src/config.rs
use serde::{Deserialize, Serialize};

/// Configuration for a single provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub provider_type: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// Registry configuration with routing rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRegistryConfig {
    pub default_provider: String,
    #[serde(default)]
    pub fallback_chain: Vec<String>,
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing_strategy: Option<RoutingStrategy>,
}

/// Routing strategy for provider selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RoutingStrategy {
    /// Always use the default provider, fallback on error
    DefaultWithFallback,
    /// Round-robin across available providers
    RoundRobin,
    /// Select provider with lowest cost per token
    LowestCost,
    /// Select provider with highest availability score
    HighestAvailability,
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        RoutingStrategy::DefaultWithFallback
    }
}

impl Default for ProviderRegistryConfig {
    fn default() -> Self {
        Self {
            default_provider: "openai".to_string(),
            fallback_chain: vec![],
            providers: vec![],
            routing_strategy: Some(RoutingStrategy::default()),
        }
    }
}
