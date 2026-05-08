use anyhow::Result;
use std::env;
use std::path::Path;
use std::sync::Arc;
use wireframe_provider_anthropic::{AnthropicConfig, AnthropicProvider};
use wireframe_provider_cohere::{CohereConfig, CohereProvider};
use wireframe_provider_core::config::{ProviderConfig, ProviderRegistryConfig};
use wireframe_provider_core::discovery::ProviderDiscoveryRegistry;
use wireframe_provider_core::Provider;
use wireframe_provider_google::{GeminiConfig, GeminiProvider};
use wireframe_provider_ollama::{OllamaConfig, OllamaProvider};
use wireframe_provider_openai::{OpenAIConfig, OpenAIProvider};

pub fn load_provider_config(config_path: Option<&Path>) -> Result<ProviderRegistryConfig> {
    if let Some(path) = config_path {
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                return Ok(toml::from_str(&content)?);
            } else {
                return Ok(serde_json::from_str(&content)?);
            }
        }
    }

    Ok(default_config_from_env())
}

pub fn default_config_from_env() -> ProviderRegistryConfig {
    let mut providers = Vec::new();

    if env::var("OPENAI_API_KEY").is_ok() {
        providers.push(ProviderConfig {
            name: "openai".to_string(),
            provider_type: "openai".to_string(),
            model: env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string()),
            api_key: env::var("OPENAI_API_KEY").ok(),
            base_url: env::var("OPENAI_BASE_URL").ok(),
            enabled: Some(true),
            priority: Some(100),
            max_tokens: None,
            temperature: None,
            stream: Some(true),
        });
    }

    if env::var("ANTHROPIC_API_KEY").is_ok() {
        providers.push(ProviderConfig {
            name: "anthropic".to_string(),
            provider_type: "anthropic".to_string(),
            model: env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string()),
            api_key: env::var("ANTHROPIC_API_KEY").ok(),
            base_url: env::var("ANTHROPIC_BASE_URL").ok(),
            enabled: Some(true),
            priority: Some(90),
            max_tokens: None,
            temperature: None,
            stream: Some(true),
        });
    }

    if env::var("GOOGLE_API_KEY").is_ok() || env::var("GEMINI_API_KEY").is_ok() {
        providers.push(ProviderConfig {
            name: "google".to_string(),
            provider_type: "google".to_string(),
            model: env::var("GOOGLE_MODEL")
                .or(env::var("GEMINI_MODEL"))
                .unwrap_or_else(|_| "gemini-1.5-pro".to_string()),
            api_key: env::var("GOOGLE_API_KEY")
                .or(env::var("GEMINI_API_KEY"))
                .ok(),
            base_url: env::var("GOOGLE_BASE_URL").ok(),
            enabled: Some(true),
            priority: Some(80),
            max_tokens: None,
            temperature: None,
            stream: Some(true),
        });
    }

    if env::var("COHERE_API_KEY").is_ok() {
        providers.push(ProviderConfig {
            name: "cohere".to_string(),
            provider_type: "cohere".to_string(),
            model: env::var("COHERE_MODEL").unwrap_or_else(|_| "command-r".to_string()),
            api_key: env::var("COHERE_API_KEY").ok(),
            base_url: env::var("COHERE_BASE_URL").ok(),
            enabled: Some(true),
            priority: Some(70),
            max_tokens: None,
            temperature: None,
            stream: Some(true),
        });
    }

    // Ollama is always available (local)
    providers.push(ProviderConfig {
        name: "ollama".to_string(),
        provider_type: "ollama".to_string(),
        model: env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string()),
        api_key: None,
        base_url: env::var("OLLAMA_BASE_URL").ok(),
        enabled: Some(true),
        priority: Some(50),
        max_tokens: None,
        temperature: None,
        stream: Some(true),
    });

    // DeepSeek (OpenAI-compatible)
    if env::var("DEEPSEEK_API_KEY").is_ok() {
        providers.push(ProviderConfig {
            name: "deepseek".to_string(),
            provider_type: "openai".to_string(),
            model: "deepseek-chat".to_string(),
            api_key: env::var("DEEPSEEK_API_KEY").ok(),
            base_url: Some("https://api.deepseek.com".to_string()),
            enabled: Some(true),
            priority: Some(85),
            max_tokens: None,
            temperature: None,
            stream: Some(true),
        });
    }

    // OpenCode Go (OpenAI-compatible)
    if env::var("OPENCODE_GO_API_KEY").is_ok() {
        providers.push(ProviderConfig {
            name: "opencode-go".to_string(),
            provider_type: "openai".to_string(),
            model: "deepseek-v4-flash".to_string(),
            api_key: env::var("OPENCODE_GO_API_KEY").ok(),
            base_url: Some("https://opencode.ai/zen/go/v1".to_string()),
            enabled: Some(true),
            priority: Some(75),
            max_tokens: None,
            temperature: None,
            stream: Some(true),
        });
    }

    ProviderRegistryConfig {
        default_provider: providers
            .first()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "openai".to_string()),
        fallback_chain: providers
            .iter()
            .filter(|p| p.enabled.unwrap_or(true))
            .map(|p| p.name.clone())
            .collect(),
        providers,
        routing_strategy: None,
    }
}

pub fn build_registry_from_config(
    config: &ProviderRegistryConfig,
) -> Result<ProviderDiscoveryRegistry> {
    let mut registry = ProviderDiscoveryRegistry::new();

    for provider_config in &config.providers {
        if !provider_config.enabled.unwrap_or(true) {
            continue;
        }

        let provider: Arc<dyn Provider> = match provider_config.provider_type.as_str() {
            "openai" => {
                let openai_config = OpenAIConfig {
                    api_key: provider_config.api_key.clone(),
                    base_url: provider_config.base_url.clone(),
                    model: provider_config.model.clone(),
                    stream: provider_config.stream,
                };
                Arc::new(OpenAIProvider::new(openai_config))
            }
            "anthropic" => {
                let anthropic_config = AnthropicConfig {
                    api_key: provider_config.api_key.clone(),
                    model: provider_config.model.clone(),
                    base_url: provider_config.base_url.clone(),
                    stream: provider_config.stream,
                };
                Arc::new(AnthropicProvider::new(anthropic_config))
            }
            "google" => {
                let gemini_config = GeminiConfig {
                    api_key: provider_config.api_key.clone(),
                    model: provider_config.model.clone(),
                    base_url: provider_config.base_url.clone(),
                    stream: provider_config.stream,
                };
                Arc::new(GeminiProvider::new(gemini_config))
            }
            "cohere" => {
                let cohere_config = CohereConfig {
                    api_key: provider_config.api_key.clone(),
                    model: provider_config.model.clone(),
                    base_url: provider_config.base_url.clone(),
                    stream: provider_config.stream,
                };
                Arc::new(CohereProvider::new(cohere_config))
            }
            "ollama" => {
                let ollama_config = OllamaConfig {
                    model: provider_config.model.clone(),
                    base_url: provider_config.base_url.clone(),
                    stream: provider_config.stream,
                };
                Arc::new(OllamaProvider::new(ollama_config))
            }
            _ => {
                tracing::warn!(
                    provider_type = %provider_config.provider_type,
                    "Unknown provider type, skipping"
                );
                continue;
            }
        };

        registry.register(&provider_config.name, provider);
    }

    Ok(registry)
}
