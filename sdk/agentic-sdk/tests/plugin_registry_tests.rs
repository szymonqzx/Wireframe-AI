use agentic_sdk::plugin::{Plugin, PluginError};
use agentic_sdk::plugin_registry::PluginRegistry;
use serde_json::Value;

#[derive(Clone)]
struct MockPlugin {
    id: &'static str,
    initialized: bool,
}

#[async_trait::async_trait]
impl Plugin for MockPlugin {
    fn plugin_id(&self) -> &'static str {
        self.id
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Mock plugin"
    }

    async fn initialize(&mut self, _config: &Value) -> Result<(), PluginError> {
        self.initialized = true;
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(self.initialized)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        self.initialized = false;
        Ok(())
    }
}

#[tokio::test]
async fn test_register_and_get_plugin() {
    let registry = PluginRegistry::new();
    let plugin = Box::new(MockPlugin {
        id: "test-plugin",
        initialized: false,
    });

    registry.register(plugin).unwrap();

    let retrieved = registry.get::<MockPlugin>("test-plugin");
    assert!(retrieved.is_ok());
}

#[tokio::test]
async fn test_plugin_not_found() {
    let registry = PluginRegistry::new();
    let result = registry.get::<MockPlugin>("nonexistent");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_is_registered() {
    let registry = PluginRegistry::new();
    let plugin = Box::new(MockPlugin {
        id: "test-plugin",
        initialized: false,
    });

    assert!(!registry.is_registered("test-plugin"));
    registry.register(plugin).unwrap();
    assert!(registry.is_registered("test-plugin"));
}

#[tokio::test]
async fn test_register_multiple_plugins() {
    let registry = PluginRegistry::new();

    registry
        .register(Box::new(MockPlugin {
            id: "plugin1",
            initialized: false,
        }))
        .unwrap();

    registry
        .register(Box::new(MockPlugin {
            id: "plugin2",
            initialized: false,
        }))
        .unwrap();

    registry
        .register(Box::new(MockPlugin {
            id: "plugin3",
            initialized: false,
        }))
        .unwrap();

    assert_eq!(registry.count(), 3);
    assert!(registry.is_registered("plugin1"));
    assert!(registry.is_registered("plugin2"));
    assert!(registry.is_registered("plugin3"));
}

#[tokio::test]
async fn test_unregister_plugin() {
    let registry = PluginRegistry::new();
    registry
        .register(Box::new(MockPlugin {
            id: "test-plugin",
            initialized: false,
        }))
        .unwrap();

    assert!(registry.is_registered("test-plugin"));
    registry.unregister("test-plugin").unwrap();
    assert!(!registry.is_registered("test-plugin"));
}

#[tokio::test]
async fn test_unregister_nonexistent_plugin() {
    let registry = PluginRegistry::new();
    let result = registry.unregister("nonexistent");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_plugins() {
    let registry = PluginRegistry::new();

    registry
        .register(Box::new(MockPlugin {
            id: "plugin1",
            initialized: false,
        }))
        .unwrap();

    registry
        .register(Box::new(MockPlugin {
            id: "plugin2",
            initialized: false,
        }))
        .unwrap();

    let plugins = registry.list_plugins();
    assert_eq!(plugins.len(), 2);
    assert!(plugins.contains(&"plugin1".to_string()));
    assert!(plugins.contains(&"plugin2".to_string()));
}

#[tokio::test]
async fn test_clear_plugins() {
    let registry = PluginRegistry::new();

    registry
        .register(Box::new(MockPlugin {
            id: "plugin1",
            initialized: false,
        }))
        .unwrap();

    registry
        .register(Box::new(MockPlugin {
            id: "plugin2",
            initialized: false,
        }))
        .unwrap();

    assert_eq!(registry.count(), 2);
    registry.clear();
    assert_eq!(registry.count(), 0);
}

#[tokio::test]
async fn test_plugin_registry_default() {
    let registry = PluginRegistry::default();
    assert_eq!(registry.count(), 0);
}

#[tokio::test]
async fn test_register_duplicate_plugin() {
    let registry = PluginRegistry::new();
    registry
        .register(Box::new(MockPlugin {
            id: "test-plugin",
            initialized: false,
        }))
        .unwrap();

    // Registering a plugin with the same ID should replace it
    registry
        .register(Box::new(MockPlugin {
            id: "test-plugin",
            initialized: false,
        }))
        .unwrap();

    assert_eq!(registry.count(), 1);
}
