use agentic_sdk::config::PluginConfig;
use serde_json::json;
use std::fs;
use std::env;

#[test]
fn test_expand_env_vars() {
    env::set_var("TEST_VAR", "test_value");
    env::set_var("ANOTHER_VAR", "another_value");

    let yaml = r#"
modules:
  test_module:
    enabled: true
    plugins:
      storage:
        plugin_id: "storage-${TEST_VAR}"
        config:
          path: "/path/${ANOTHER_VAR}"
      enrichment_pipeline: []
      tools: []
"#;

    let mut config = PluginConfig::from_yaml(yaml).unwrap();
    config.expand_env_vars().unwrap();

    let storage_plugin = &config.modules["test_module"].plugins.storage.as_ref().unwrap();
    assert_eq!(storage_plugin.plugin_id, "storage-test_value");
    assert_eq!(storage_plugin.config["path"], "/path/another_value");

    env::remove_var("TEST_VAR");
    env::remove_var("ANOTHER_VAR");
}

#[test]
fn test_expand_env_vars_missing() {
    env::remove_var("NONEXISTENT_VAR");

    let yaml = r#"
modules:
  test_module:
    enabled: true
    plugins:
      storage:
        plugin_id: "storage-${NONEXISTENT_VAR}"
      enrichment_pipeline: []
      tools: []
"#;

    let mut config = PluginConfig::from_yaml(yaml).unwrap();
    config.expand_env_vars().unwrap();

    let storage_plugin = &config.modules["test_module"].plugins.storage.as_ref().unwrap();
    // Missing vars should be replaced with empty string
    assert_eq!(storage_plugin.plugin_id, "storage-");
}

#[test]
fn test_expand_env_vars_nested() {
    env::set_var("VAR1", "value1");
    env::set_var("VAR2", "value2");

    let yaml = r#"
modules:
  test_module:
    enabled: true
    plugins:
      storage:
        plugin_id: "storage"
        config:
          nested:
            key1: "${VAR1}"
            key2: "${VAR2}"
            array:
              - "${VAR1}"
              - "${VAR2}"
      enrichment_pipeline: []
      tools: []
"#;

    let mut config = PluginConfig::from_yaml(yaml).unwrap();
    config.expand_env_vars().unwrap();

    let storage_plugin = &config.modules["test_module"].plugins.storage.as_ref().unwrap();
    assert_eq!(storage_plugin.config["nested"]["key1"], "value1");
    assert_eq!(storage_plugin.config["nested"]["key2"], "value2");
    assert_eq!(storage_plugin.config["nested"]["array"][0], "value1");
    assert_eq!(storage_plugin.config["nested"]["array"][1], "value2");

    env::remove_var("VAR1");
    env::remove_var("VAR2");
}

#[test]
fn test_expand_env_vars_no_vars() {
    let yaml = r#"
modules:
  test_module:
    enabled: true
    plugins:
      storage:
        plugin_id: "storage"
      enrichment_pipeline: []
      tools: []
"#;

    let mut config = PluginConfig::from_yaml(yaml).unwrap();
    config.expand_env_vars().unwrap();

    let storage_plugin = &config.modules["test_module"].plugins.storage.as_ref().unwrap();
    assert_eq!(storage_plugin.plugin_id, "storage");
}

#[test]
fn test_validate_config_without_feature() {
    let yaml = r#"
modules:
  test_module:
    enabled: true
    plugins:
      storage:
        plugin_id: "storage"
      enrichment_pipeline: []
      tools: []
"#;

    let config = PluginConfig::from_yaml(yaml).unwrap();
    let schema = json!({
        "type": "object",
        "properties": {
            "modules": {
                "type": "object"
            }
        }
    });

    // Should succeed without schema-validation feature (no-op)
    let result = config.validate(&schema);
    assert!(result.is_ok());
}

#[cfg(feature = "schema-validation")]
#[test]
fn test_validate_config_with_feature() {
    let yaml = r#"
modules:
  test_module:
    enabled: true
    plugins:
      storage:
        plugin_id: "storage"
      enrichment_pipeline: []
      tools: []
"#;

    let config = PluginConfig::from_yaml(yaml).unwrap();
    let schema = json!({
        "type": "object",
        "properties": {
            "modules": {
                "type": "object"
            }
        },
        "required": ["modules"]
    });

    let result = config.validate(&schema);
    assert!(result.is_ok());
}

#[cfg(feature = "schema-validation")]
#[test]
fn test_validate_config_invalid() {
    let yaml = r#"
modules:
  test_module:
    enabled: true
    plugins:
      storage:
        plugin_id: "storage"
      enrichment_pipeline: []
      tools: []
"#;

    let config = PluginConfig::from_yaml(yaml).unwrap();
    let schema = json!({
        "type": "object",
        "properties": {
            "modules": {
                "type": "object",
                "properties": {
                    "test_module": {
                        "type": "object",
                        "properties": {
                            "enabled": {
                                "type": "boolean"
                            },
                            "plugins": {
                                "type": "object"
                            }
                        },
                        "required": ["enabled", "plugins", "invalid_field"]
                    }
                }
            }
        }
    });

    let result = config.validate(&schema);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_config_watcher() {
    let temp_dir = std::env::temp_dir();
    let config_path = temp_dir.join("test_config.yaml");
    
    let yaml = r#"
modules:
  test_module:
    enabled: true
    plugins:
      storage:
        plugin_id: "storage"
      enrichment_pipeline: []
      tools: []
"#;

    fs::write(&config_path, yaml).unwrap();

    let watcher = agentic_sdk::config::ConfigWatcher::new(config_path.clone()).unwrap();
    
    // Test that we can load the config
    let config = watcher.get_config().await;
    assert!(config.modules.contains_key("test_module"));

    // Clean up
    fs::remove_file(&config_path).unwrap();
}
