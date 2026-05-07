use wireframe_provider_core::{router::ProviderRouter, discovery::ProviderDiscoveryRegistry};
use wireframe_provider_core::{Provider, ProviderStatus, Availability, SetupState};
use std::sync::Arc;
use async_trait::async_trait;

struct MockProvider {
    name: String,
    available: bool,
}

#[async_trait]
impl Provider for MockProvider {
    async fn complete(
        &self,
        _messages: &[wireframe_provider_core::Message],
        _tools: &[wireframe_provider_core::ToolDefinition],
        _system: &str,
        _session_id: Option<&str>,
    ) -> anyhow::Result<wireframe_provider_core::EventStream> {
        unimplemented!()
    }

    fn describe(&self) -> wireframe_provider_core::ProviderMetadata {
        wireframe_provider_core::ProviderMetadata {
            provider_id: self.name.clone(),
            provider_label: self.name.clone(),
            provider_version: "0.1.0".to_string(),
            protocol_version: "0.1.0".to_string(),
            transport: "http".to_string(),
            capabilities: wireframe_provider_core::ProviderCapabilities {
                core_methods: vec!["complete".to_string()],
                optional_methods: vec![],
                features: vec![],
                custom_methods: vec![],
            },
        }
    }

    fn status(&self) -> ProviderStatus {
        ProviderStatus {
            availability: if self.available { Availability::Ready } else { Availability::Unavailable },
            setup_state: SetupState::Complete,
            requires_manual_setup: false,
            diagnostics: vec![],
        }
    }

    fn name(&self) -> &str { &self.name }
    fn model(&self) -> String { "test".to_string() }
    fn set_model(&self, _model: &str) -> anyhow::Result<()> { Ok(()) }
    fn available_models(&self) -> Vec<String> { vec![] }
    fn supports_streaming(&self) -> bool { true }
    fn cost_per_1k_tokens(&self) -> Option<(u64, u64)> { Some((10, 20)) }
    fn fork(&self) -> Arc<dyn Provider> { unimplemented!() }
}

#[test]
fn test_router_selects_available_provider() {
    let mut registry = ProviderDiscoveryRegistry::new();
    registry.register("provider1", Arc::new(MockProvider {
        name: "provider1".to_string(),
        available: true,
    }));
    registry.register("provider2", Arc::new(MockProvider {
        name: "provider2".to_string(),
        available: false,
    }));

    let router = ProviderRouter::new(Arc::new(registry), vec!["provider1".to_string(), "provider2".to_string()]);
    let selected = router.select_provider(&[]).unwrap();
    assert_eq!(selected, "provider1");
}

#[test]
fn test_router_fallback_on_unavailable() {
    let mut registry = ProviderDiscoveryRegistry::new();
    registry.register("provider1", Arc::new(MockProvider {
        name: "provider1".to_string(),
        available: false,
    }));
    registry.register("provider2", Arc::new(MockProvider {
        name: "provider2".to_string(),
        available: true,
    }));

    let router = ProviderRouter::new(Arc::new(registry), vec!["provider1".to_string(), "provider2".to_string()]);
    let selected = router.select_provider(&[]).unwrap();
    assert_eq!(selected, "provider2");
}
