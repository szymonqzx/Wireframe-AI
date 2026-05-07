//! Provider marketplace infrastructure for Wireframe-AI.
//!
//! Enables discovery, installation, and management of community providers.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A provider package in the marketplace.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProviderPackage {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub repository_url: Option<String>,
    pub documentation_url: Option<String>,
    pub provider_type: ProviderType,
    pub transport: String,
    pub supported_models: Vec<String>,
    pub dependencies: Vec<String>,
    pub tags: Vec<String>,
    pub downloads: u64,
    pub rating: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderType {
    CloudHttp,
    LocalHttp,
    Stdio,
    Custom,
}

/// Marketplace index of available providers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MarketplaceIndex {
    pub version: String,
    pub providers: Vec<ProviderPackage>,
    pub last_updated: i64,
}

impl MarketplaceIndex {
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            providers: vec![],
            last_updated: chrono::Utc::now().timestamp(),
        }
    }

    pub fn add(&mut self, package: ProviderPackage) {
        self.providers.push(package);
        self.last_updated = chrono::Utc::now().timestamp();
    }

    /// Search providers by query string.
    pub fn search(&self, query: &str) -> Vec<&ProviderPackage> {
        let q = query.to_lowercase();
        self.providers
            .iter()
            .filter(|p| {
                p.name.to_lowercase().contains(&q)
                    || p.description.to_lowercase().contains(&q)
                    || p.tags.iter().any(|t| t.to_lowercase().contains(&q))
            })
            .collect()
    }

    /// Find by ID.
    pub fn get(&self, id: &str) -> Option<&ProviderPackage> {
        self.providers.iter().find(|p| p.id == id)
    }

    /// List by transport type.
    pub fn by_transport(&self, transport: &str) -> Vec<&ProviderPackage> {
        self.providers
            .iter()
            .filter(|p| p.transport == transport)
            .collect()
    }

    /// List by provider type.
    pub fn by_type(&self, provider_type: ProviderType) -> Vec<&ProviderPackage> {
        self.providers
            .iter()
            .filter(|p| {
                std::mem::discriminant(&p.provider_type) == std::mem::discriminant(&provider_type)
            })
            .collect()
    }
}

impl Default for MarketplaceIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Client for fetching marketplace data.
pub struct MarketplaceClient {
    base_url: String,
    http: Option<reqwest::Client>,
    cache: HashMap<String, MarketplaceIndex>,
}

impl MarketplaceClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            http: Some(reqwest::Client::new()),
            cache: HashMap::new(),
        }
    }

    /// Create an offline client that only serves the built-in index.
    pub fn new_offline() -> Self {
        Self {
            base_url: String::new(),
            http: None,
            cache: HashMap::new(),
        }
    }

    /// Fetch the marketplace index from the configured URL.
    /// Falls back to the built-in index if the HTTP request fails or if
    /// the client was created in offline mode.
    pub async fn fetch_index(&mut self) -> anyhow::Result<MarketplaceIndex> {
        if let Some(http) = &self.http {
            let url = format!("{}/index.json", self.base_url.trim_end_matches('/'));
            match http.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    match resp.json::<MarketplaceIndex>().await {
                        Ok(index) => {
                            self.cache.insert("default".to_string(), index.clone());
                            return Ok(index);
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, url, "failed to parse marketplace index; using built-in fallback");
                        }
                    }
                }
                Ok(resp) => {
                    tracing::warn!(status = %resp.status(), url, "marketplace request failed; using built-in fallback");
                }
                Err(e) => {
                    tracing::warn!(error = %e, url, "marketplace request error; using built-in fallback");
                }
            }
        }
        let index = built_in_index();
        self.cache.insert("default".to_string(), index.clone());
        Ok(index)
    }

    /// Get cached index.
    pub fn cached_index(&self) -> Option<&MarketplaceIndex> {
        self.cache.get("default")
    }

    /// Install a provider by ID.
    ///
    /// In a full implementation this would download and build the crate.
    /// For now it returns instructions so the CLI or user can complete the step.
    pub async fn install(&self, package_id: &str) -> anyhow::Result<String> {
        if let Some(index) = self.cached_index() {
            if index.get(package_id).is_some() {
                return Ok(format!(
                    "Provider '{}' is available. To install, add the corresponding crate to your Cargo.toml and rebuild.",
                    package_id
                ));
            }
        }
        let fallback = built_in_index();
        if fallback.get(package_id).is_some() {
            Ok(format!(
                "Provider '{}' is available. To install, add the corresponding crate to your Cargo.toml and rebuild.",
                package_id
            ))
        } else {
            anyhow::bail!("Provider '{}' not found in marketplace index", package_id)
        }
    }
}

/// Built-in marketplace index with known providers.
pub fn built_in_index() -> MarketplaceIndex {
    let mut index = MarketplaceIndex::new();

    index.add(ProviderPackage {
        id: "openai".to_string(),
        name: "OpenAI".to_string(),
        version: "0.1.0".to_string(),
        description: "OpenAI GPT-4, GPT-4o, GPT-3.5 provider".to_string(),
        author: "Wireframe-AI Team".to_string(),
        license: "MIT".to_string(),
        repository_url: Some("https://github.com/wireframe-ai/providers/openai".to_string()),
        documentation_url: None,
        provider_type: ProviderType::CloudHttp,
        transport: "http".to_string(),
        supported_models: vec![
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-3.5-turbo".to_string(),
        ],
        dependencies: vec!["reqwest".to_string()],
        tags: vec![
            "cloud".to_string(),
            "openai".to_string(),
            "popular".to_string(),
        ],
        downloads: 10000,
        rating: 4.8,
    });

    index.add(ProviderPackage {
        id: "anthropic".to_string(),
        name: "Anthropic Claude".to_string(),
        version: "0.1.0".to_string(),
        description: "Anthropic Claude 3.5 Sonnet, Opus, Haiku provider".to_string(),
        author: "Wireframe-AI Team".to_string(),
        license: "MIT".to_string(),
        repository_url: Some("https://github.com/wireframe-ai/providers/anthropic".to_string()),
        documentation_url: None,
        provider_type: ProviderType::CloudHttp,
        transport: "http".to_string(),
        supported_models: vec![
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-5-haiku-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
        ],
        dependencies: vec!["reqwest".to_string()],
        tags: vec![
            "cloud".to_string(),
            "anthropic".to_string(),
            "tools".to_string(),
        ],
        downloads: 8500,
        rating: 4.9,
    });

    index.add(ProviderPackage {
        id: "ollama".to_string(),
        name: "Ollama".to_string(),
        version: "0.1.0".to_string(),
        description: "Local model provider via Ollama".to_string(),
        author: "Wireframe-AI Team".to_string(),
        license: "MIT".to_string(),
        repository_url: Some("https://github.com/wireframe-ai/providers/ollama".to_string()),
        documentation_url: None,
        provider_type: ProviderType::LocalHttp,
        transport: "http".to_string(),
        supported_models: vec![
            "llama3.2".to_string(),
            "llama3.1".to_string(),
            "mistral".to_string(),
            "qwen2.5".to_string(),
            "phi4".to_string(),
        ],
        dependencies: vec!["reqwest".to_string()],
        tags: vec![
            "local".to_string(),
            "ollama".to_string(),
            "free".to_string(),
        ],
        downloads: 12000,
        rating: 4.5,
    });

    index.add(ProviderPackage {
        id: "google".to_string(),
        name: "Google Gemini".to_string(),
        version: "0.1.0".to_string(),
        description: "Google Gemini provider".to_string(),
        author: "Wireframe-AI Team".to_string(),
        license: "MIT".to_string(),
        repository_url: Some("https://github.com/wireframe-ai/providers/google".to_string()),
        documentation_url: None,
        provider_type: ProviderType::CloudHttp,
        transport: "http".to_string(),
        supported_models: vec![
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
            "gemini-1.0-pro".to_string(),
        ],
        dependencies: vec!["reqwest".to_string()],
        tags: vec!["cloud".to_string(), "google".to_string()],
        downloads: 5000,
        rating: 4.6,
    });

    index.add(ProviderPackage {
        id: "cohere".to_string(),
        name: "Cohere".to_string(),
        version: "0.1.0".to_string(),
        description: "Cohere Command and Embed provider".to_string(),
        author: "Wireframe-AI Team".to_string(),
        license: "MIT".to_string(),
        repository_url: Some("https://github.com/wireframe-ai/providers/cohere".to_string()),
        documentation_url: None,
        provider_type: ProviderType::CloudHttp,
        transport: "http".to_string(),
        supported_models: vec![
            "command-r".to_string(),
            "command-r-plus".to_string(),
            "command".to_string(),
        ],
        dependencies: vec!["reqwest".to_string()],
        tags: vec!["cloud".to_string(), "cohere".to_string()],
        downloads: 3000,
        rating: 4.4,
    });

    index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_marketplace_search() {
        let index = built_in_index();
        let results = index.search("local");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_marketplace_get() {
        let index = built_in_index();
        let pkg = index.get("openai");
        assert!(pkg.is_some());
        assert_eq!(pkg.unwrap().name, "OpenAI");
    }

    #[test]
    fn test_marketplace_by_transport() {
        let index = built_in_index();
        let http = index.by_transport("http");
        assert_eq!(http.len(), 5);
    }
}
