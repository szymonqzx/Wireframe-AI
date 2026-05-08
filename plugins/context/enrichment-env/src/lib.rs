use agentic_sdk::message_types::{ContextPackage, TaskComplete, TaskSubmitted};
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::context::{EnrichmentError, EnrichmentStrategy};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

pub struct EnvEnrichmentPlugin;

impl Default for EnvEnrichmentPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvEnrichmentPlugin {
    pub fn new() -> Self {
        Self
    }

    fn filter_safe_env_vars() -> HashMap<String, String> {
        std::env::vars()
            .filter(|(k, _)| {
                let key_lower = k.to_lowercase();
                let secret_suffixes = [
                    "_key",
                    "_secret",
                    "_password",
                    "_token",
                    "_api_key",
                    "_api_secret",
                    "_auth",
                    "_credential",
                    "_private",
                ];
                let secret_prefixes = [
                    "key_",
                    "secret_",
                    "password_",
                    "token_",
                    "api_key_",
                    "api_secret_",
                    "auth_",
                    "credential_",
                    "private_",
                ];
                let has_secret_suffix = secret_suffixes
                    .iter()
                    .any(|suffix| key_lower.len() > suffix.len() && key_lower.ends_with(suffix));
                let has_secret_prefix = secret_prefixes
                    .iter()
                    .any(|prefix| key_lower.len() > prefix.len() && key_lower.starts_with(prefix));
                let exact_secret = matches!(
                    key_lower.as_str(),
                    "api_key" | "secret" | "password" | "token" | "auth" | "credential" | "private"
                );
                !has_secret_suffix && !has_secret_prefix && !exact_secret
            })
            .collect()
    }
}

#[async_trait]
impl Plugin for EnvEnrichmentPlugin {
    fn plugin_id(&self) -> &'static str {
        "enrichment-env"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Environment variable enrichment strategy"
    }

    async fn initialize(
        &mut self,
        _config: &Value,
    ) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, agentic_sdk::plugin::PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }
}

#[async_trait]
impl EnrichmentStrategy for EnvEnrichmentPlugin {
    async fn enrich<'a>(
        &'a self,
        _task: &'a TaskSubmitted,
        context: &'a ContextPackage,
    ) -> Result<ContextPackage, EnrichmentError> {
        let safe_env = Self::filter_safe_env_vars();
        let mut enriched = context.clone();
        enriched.safe_env = safe_env;
        Ok(enriched)
    }

    async fn on_complete<'a>(
        &'a self,
        _session_id: &'a str,
        _result: &'a TaskComplete,
    ) -> Result<(), EnrichmentError> {
        Ok(())
    }
}
