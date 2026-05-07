// provider-core/tests/config_test.rs
use wireframe_provider_core::config::{ProviderConfig, ProviderRegistryConfig};
use serde_json;

#[test]
fn test_provider_config_deserialization() {
    let json = r#"{
        "name": "openai",
        "provider_type": "openai",
        "model": "gpt-4o",
        "api_key": "sk-test",
        "base_url": "https://api.openai.com/v1"
    }"#;
    
    let config: ProviderConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.name, "openai");
    assert_eq!(config.model, "gpt-4o");
}

#[test]
fn test_registry_config_with_fallback() {
    let json = r#"{
        "default_provider": "openai",
        "fallback_chain": ["anthropic", "google"],
        "providers": [
            {
                "name": "openai",
                "provider_type": "openai",
                "model": "gpt-4o"
            }
        ]
    }"#;
    
    let config: ProviderRegistryConfig = serde_json::from_str(json).unwrap();
    assert_eq!(config.default_provider, "openai");
    assert_eq!(config.fallback_chain.len(), 2);
}
