//! Provider discovery and registration system for Wireframe-AI.
//!
//! Enables automatic detection and registration of LLM providers
//! through configuration, environment variables, and dynamic discovery.

use crate::{Provider, ProviderMetadata, ProviderStatus};
use std::collections::HashMap;
use std::sync::Arc;

/// Registry of discovered providers.
pub struct ProviderDiscoveryRegistry {
    providers: HashMap<String, Arc<dyn Provider>>,
    metadata: HashMap<String, ProviderMetadata>,
    /// Providers discovered from environment variables (available but not yet instantiated).
    available_from_env: HashMap<String, ProviderMetadata>,
}

impl ProviderDiscoveryRegistry {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            metadata: HashMap::new(),
            available_from_env: HashMap::new(),
        }
    }

    /// Register a provider.
    pub fn register(&mut self, name: impl Into<String>, provider: Arc<dyn Provider>) {
        let name = name.into();
        let meta = provider.describe();
        self.providers.insert(name.clone(), provider);
        self.metadata.insert(name, meta);
    }

    /// Get a provider by name.
    pub fn get(&self, name: &str) -> Option<Arc<dyn Provider>> {
        self.providers.get(name).cloned()
    }

    /// Get metadata for a provider (registered or discovered from env).
    pub fn metadata(&self, name: &str) -> Option<&ProviderMetadata> {
        self.metadata
            .get(name)
            .or_else(|| self.available_from_env.get(name))
    }

    /// Check whether a provider is known (registered or env-discovered).
    pub fn is_available(&self, name: &str) -> bool {
        self.providers.contains_key(name) || self.available_from_env.contains_key(name)
    }

    /// List all registered provider names.
    pub fn list(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    /// List all available provider names (registered + env-discovered).
    pub fn list_available(&self) -> Vec<String> {
        let mut names: Vec<String> = self.providers.keys().cloned().collect();
        for name in self.available_from_env.keys() {
            if !names.contains(name) {
                names.push(name.clone());
            }
        }
        names
    }

    /// Get status for all providers.
    pub fn statuses(&self) -> HashMap<String, ProviderStatus> {
        self.providers
            .iter()
            .map(|(name, provider)| (name.clone(), provider.status()))
            .collect()
    }

    /// Discover providers from environment variables.
    ///
    /// Inspects well-known API key environment variables and records
    /// placeholder metadata for any providers that are available but
    /// not yet instantiated. Returns the list of newly-discovered providers.
    pub fn discover_from_env(&mut self) -> Vec<(String, ProviderMetadata)> {
        let mut discovered = Vec::new();
        for (key, _value) in std::env::vars() {
            let (name, meta) = match key.as_str() {
                "OPENAI_API_KEY" => ("openai", placeholder_metadata("openai", "OpenAI GPT")),
                "ANTHROPIC_API_KEY" => (
                    "anthropic",
                    placeholder_metadata("anthropic", "Anthropic Claude"),
                ),
                "COHERE_API_KEY" => ("cohere", placeholder_metadata("cohere", "Cohere Command")),
                "GOOGLE_API_KEY" | "GEMINI_API_KEY" => {
                    ("google", placeholder_metadata("google", "Google Gemini"))
                }
                _ => continue,
            };
            if !self.providers.contains_key(name) && !self.available_from_env.contains_key(name) {
                tracing::info!(provider = %name, "discovered provider from environment");
                self.available_from_env
                    .insert(name.to_string(), meta.clone());
                discovered.push((name.to_string(), meta));
            }
        }
        discovered
    }

    /// Get providers discovered from environment variables.
    pub fn available_from_env(&self) -> &HashMap<String, ProviderMetadata> {
        &self.available_from_env
    }

    /// Find providers that support a specific capability.
    pub fn by_capability(&self, capability: &str) -> Vec<(String, ProviderMetadata)> {
        self.metadata
            .iter()
            .filter(|(_, meta)| {
                meta.capabilities.features.contains(&capability.to_string())
                    || meta
                        .capabilities
                        .core_methods
                        .contains(&capability.to_string())
            })
            .map(|(name, meta)| (name.clone(), meta.clone()))
            .collect()
    }

    /// Find providers by transport type.
    pub fn by_transport(&self, transport: &str) -> Vec<(String, ProviderMetadata)> {
        self.metadata
            .iter()
            .filter(|(_, meta)| meta.transport == transport)
            .map(|(name, meta)| (name.clone(), meta.clone()))
            .collect()
    }
}

impl Default for ProviderDiscoveryRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Provider capability negotiation.
pub struct CapabilityNegotiator;

impl CapabilityNegotiator {
    /// Negotiate the best provider for a given set of requirements.
    pub fn negotiate(
        registry: &ProviderDiscoveryRegistry,
        required_features: &[String],
        preferred_transport: Option<&str>,
    ) -> Option<(String, ProviderMetadata)> {
        let mut candidates: Vec<_> = registry
            .metadata
            .iter()
            .filter(|(_, meta)| {
                required_features.iter().all(|feat| {
                    meta.capabilities.features.contains(feat)
                        || meta.capabilities.core_methods.contains(feat)
                })
            })
            .map(|(name, meta)| (name.clone(), meta.clone()))
            .collect();

        if let Some(transport) = preferred_transport {
            candidates.retain(|(_, meta)| meta.transport == transport);
        }

        // Sort by provider label for stable selection
        candidates.sort_by(|a, b| a.1.provider_label.cmp(&b.1.provider_label));
        candidates.into_iter().next()
    }

    /// Check if a provider can handle a specific model.
    pub fn supports_model(
        registry: &ProviderDiscoveryRegistry,
        provider: &str,
        model: &str,
    ) -> bool {
        registry
            .get(provider)
            .map(|p| {
                let models = p.available_models();
                models.is_empty() || models.contains(&model.to_string())
            })
            .unwrap_or(false)
    }
}

/// Create placeholder metadata for a provider discovered from environment variables.
fn placeholder_metadata(id: &str, label: &str) -> ProviderMetadata {
    ProviderMetadata {
        provider_id: id.to_string(),
        provider_label: label.to_string(),
        provider_version: "0.1.0".to_string(),
        protocol_version: "0.1.0".to_string(),
        transport: "http".to_string(),
        capabilities: crate::ProviderCapabilities {
            core_methods: vec!["complete".to_string()],
            optional_methods: vec![],
            features: vec![],
            custom_methods: vec![],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_list() {
        let registry = ProviderDiscoveryRegistry::new();
        assert!(registry.list().is_empty());
    }

    #[test]
    fn test_negotiate_empty() {
        let registry = ProviderDiscoveryRegistry::new();
        let result = CapabilityNegotiator::negotiate(&registry, &[], None);
        assert!(result.is_none());
    }

    #[test]
    fn test_by_capability_empty() {
        let registry = ProviderDiscoveryRegistry::new();
        let result = registry.by_capability("streaming");
        assert!(result.is_empty());
    }

    #[test]
    fn test_placeholder_metadata() {
        let meta = placeholder_metadata("test", "Test Provider");
        assert_eq!(meta.provider_id, "test");
        assert_eq!(meta.provider_label, "Test Provider");
    }
}
