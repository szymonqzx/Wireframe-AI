use crate::config::RoutingStrategy;
use crate::discovery::ProviderDiscoveryRegistry;
use crate::Availability;
use anyhow::Result;
use std::sync::Arc;

/// Provider router with fallback logic.
pub struct ProviderRouter {
    registry: Arc<ProviderDiscoveryRegistry>,
    fallback_chain: Vec<String>,
    strategy: RoutingStrategy,
}

impl ProviderRouter {
    pub fn new(registry: Arc<ProviderDiscoveryRegistry>, fallback_chain: Vec<String>) -> Self {
        Self {
            registry,
            fallback_chain,
            strategy: RoutingStrategy::DefaultWithFallback,
        }
    }

    pub fn with_strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn select_provider(&self, required_features: &[String]) -> Result<String> {
        match self.strategy {
            RoutingStrategy::DefaultWithFallback => self.select_with_fallback(required_features),
            RoutingStrategy::RoundRobin => self.select_round_robin(required_features),
            RoutingStrategy::LowestCost => self.select_lowest_cost(required_features),
            RoutingStrategy::HighestAvailability => {
                self.select_highest_availability(required_features)
            }
        }
    }

    fn select_with_fallback(&self, required_features: &[String]) -> Result<String> {
        // Try fallback chain first
        for provider_name in &self.fallback_chain {
            if let Some(provider) = self.registry.get(provider_name) {
                let status = provider.status();
                if matches!(status.availability, Availability::Ready) {
                    let metadata = provider.describe();
                    if required_features.iter().all(|feat| {
                        metadata.capabilities.features.contains(feat)
                            || metadata.capabilities.core_methods.contains(feat)
                    }) {
                        return Ok(provider_name.clone());
                    }
                }
            }
        }

        // Fall back to any available provider
        for provider_name in self.registry.list() {
            if let Some(provider) = self.registry.get(&provider_name) {
                let status = provider.status();
                if matches!(status.availability, Availability::Ready) {
                    let metadata = provider.describe();
                    if required_features.iter().all(|feat| {
                        metadata.capabilities.features.contains(feat)
                            || metadata.capabilities.core_methods.contains(feat)
                    }) {
                        return Ok(provider_name.clone());
                    }
                }
            }
        }

        Err(anyhow::anyhow!(
            "No available provider matches requirements"
        ))
    }

    fn select_round_robin(&self, required_features: &[String]) -> Result<String> {
        // For simplicity, use same logic as fallback for now
        self.select_with_fallback(required_features)
    }

    fn select_lowest_cost(&self, required_features: &[String]) -> Result<String> {
        let mut candidates: Vec<(String, Option<(u64, u64)>)> = Vec::new();

        for provider_name in self.registry.list() {
            if let Some(provider) = self.registry.get(&provider_name) {
                let metadata = provider.describe();
                if required_features.iter().all(|feat| {
                    metadata.capabilities.features.contains(feat)
                        || metadata.capabilities.core_methods.contains(feat)
                }) {
                    let cost = provider.cost_per_1k_tokens();
                    candidates.push((provider_name, cost));
                }
            }
        }

        candidates.sort_by(|a, b| {
            let cost_a = a.1.unwrap_or((1000, 1000));
            let cost_b = b.1.unwrap_or((1000, 1000));
            cost_a.cmp(&cost_b)
        });

        for (name, _) in candidates {
            if let Some(provider) = self.registry.get(&name) {
                let status = provider.status();
                if matches!(status.availability, Availability::Ready) {
                    return Ok(name.clone());
                }
            }
        }

        Err(anyhow::anyhow!(
            "No available provider matches requirements"
        ))
    }

    fn select_highest_availability(&self, required_features: &[String]) -> Result<String> {
        let mut candidates: Vec<(String, i32)> = Vec::new();

        for provider_name in self.registry.list() {
            if let Some(provider) = self.registry.get(&provider_name) {
                let metadata = provider.describe();
                if required_features.iter().all(|feat| {
                    metadata.capabilities.features.contains(feat)
                        || metadata.capabilities.core_methods.contains(feat)
                }) {
                    let status = provider.status();
                    let score = match status.availability {
                        Availability::Ready => 2,
                        Availability::Degraded => 1,
                        Availability::Unavailable => 0,
                    };
                    candidates.push((provider_name, score));
                }
            }
        }

        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        if let Some((name, _)) = candidates.first() {
            if let Some(provider) = self.registry.get(name) {
                let status = provider.status();
                if matches!(status.availability, Availability::Ready) {
                    return Ok(name.clone());
                }
            }
        }

        Err(anyhow::anyhow!(
            "No available provider matches requirements"
        ))
    }
}
