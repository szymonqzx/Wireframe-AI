use agentic_sdk::plugin::{Plugin, PluginError};
use serde_json::Value;

struct TestPlugin {
    initialized: bool,
    should_fail_init: bool,
    should_fail_health: bool,
    should_fail_shutdown: bool,
}

#[async_trait::async_trait]
impl Plugin for TestPlugin {
    fn plugin_id(&self) -> &'static str {
        "test-plugin"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "A test plugin"
    }

    async fn initialize(&mut self, _config: &Value) -> Result<(), PluginError> {
        if self.should_fail_init {
            return Err(PluginError::InitializationFailed("Simulated failure".to_string()));
        }
        self.initialized = true;
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        if self.should_fail_health {
            return Err(PluginError::HealthCheckFailed("Simulated failure".to_string()));
        }
        Ok(self.initialized)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        if self.should_fail_shutdown {
            return Err(PluginError::ShutdownFailed("Simulated failure".to_string()));
        }
        self.initialized = false;
        Ok(())
    }
}

#[tokio::test]
async fn test_plugin_lifecycle() {
    let mut plugin = TestPlugin {
        initialized: false,
        should_fail_init: false,
        should_fail_health: false,
        should_fail_shutdown: false,
    };
    assert!(!plugin.initialized);

    let config = serde_json::json!({});
    plugin.initialize(&config).await.unwrap();
    assert!(plugin.initialized);

    let healthy = plugin.health_check().await.unwrap();
    assert!(healthy);

    plugin.shutdown().await.unwrap();
    assert!(!plugin.initialized);
}

#[tokio::test]
async fn test_plugin_initialization_failure() {
    let mut plugin = TestPlugin {
        initialized: false,
        should_fail_init: true,
        should_fail_health: false,
        should_fail_shutdown: false,
    };

    let config = serde_json::json!({});
    let result = plugin.initialize(&config).await;
    assert!(result.is_err());
    assert!(!plugin.initialized);
}

#[tokio::test]
async fn test_plugin_health_check_failure() {
    let mut plugin = TestPlugin {
        initialized: false,
        should_fail_init: false,
        should_fail_health: true,
        should_fail_shutdown: false,
    };

    let config = serde_json::json!({});
    plugin.initialize(&config).await.unwrap();
    assert!(plugin.initialized);

    let result = plugin.health_check().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_plugin_shutdown_failure() {
    let mut plugin = TestPlugin {
        initialized: false,
        should_fail_init: false,
        should_fail_health: false,
        should_fail_shutdown: true,
    };

    let config = serde_json::json!({});
    plugin.initialize(&config).await.unwrap();
    assert!(plugin.initialized);

    let result = plugin.shutdown().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_plugin_health_check_before_initialization() {
    let plugin = TestPlugin {
        initialized: false,
        should_fail_init: false,
        should_fail_health: false,
        should_fail_shutdown: false,
    };

    let healthy = plugin.health_check().await.unwrap();
    assert!(!healthy);
}

#[tokio::test]
async fn test_plugin_multiple_initialize_calls() {
    let mut plugin = TestPlugin {
        initialized: false,
        should_fail_init: false,
        should_fail_health: false,
        should_fail_shutdown: false,
    };

    let config = serde_json::json!({});
    plugin.initialize(&config).await.unwrap();
    assert!(plugin.initialized);

    // Second initialize should succeed
    plugin.initialize(&config).await.unwrap();
    assert!(plugin.initialized);
}

#[tokio::test]
async fn test_plugin_multiple_shutdown_calls() {
    let mut plugin = TestPlugin {
        initialized: false,
        should_fail_init: false,
        should_fail_health: false,
        should_fail_shutdown: false,
    };

    let config = serde_json::json!({});
    plugin.initialize(&config).await.unwrap();
    plugin.shutdown().await.unwrap();
    assert!(!plugin.initialized);

    // Second shutdown should succeed
    plugin.shutdown().await.unwrap();
    assert!(!plugin.initialized);
}

#[tokio::test]
async fn test_plugin_with_config() {
    let mut plugin = TestPlugin {
        initialized: false,
        should_fail_init: false,
        should_fail_health: false,
        should_fail_shutdown: false,
    };

    let config = serde_json::json!({
        "setting1": "value1",
        "setting2": 42,
        "nested": {
            "key": "value"
        }
    });

    plugin.initialize(&config).await.unwrap();
    assert!(plugin.initialized);
}

#[tokio::test]
async fn test_plugin_metadata() {
    let plugin = TestPlugin {
        initialized: false,
        should_fail_init: false,
        should_fail_health: false,
        should_fail_shutdown: false,
    };

    assert_eq!(plugin.plugin_id(), "test-plugin");
    assert_eq!(plugin.version(), "1.0.0");
    assert_eq!(plugin.description(), "A test plugin");
}
