use agentic_sdk::config::PluginConfig;

#[test]
fn test_parse_simple_config() {
    let config_yaml = r#"
modules:
  context:
    enabled: true
    plugins:
      storage:
        plugin_id: "storage-sqlite"
        config:
          db_path: "./test.db"
      enrichment_pipeline: []
      memory: null
      planner: null
      execution: null
      synthesizer: null
      tools: []
      security: null
      resources: null
      input: null
      output: null
      model: null
      reasoning: null
      tool_selector: null
"#;

    let config = PluginConfig::from_yaml(config_yaml).unwrap();
    assert!(config.modules.contains_key("context"));
    assert_eq!(config.modules["context"].enabled, true);
}

#[test]
fn test_parse_json_config() {
    let config_json = r#"{
  "modules": {
    "context": {
      "enabled": true,
      "plugins": {
        "storage": {
          "plugin_id": "storage-sqlite"
        },
        "enrichment_pipeline": [],
        "memory": null,
        "planner": null,
        "execution": null,
        "synthesizer": null,
        "tools": [],
        "security": null,
        "resources": null,
        "input": null,
        "output": null,
        "model": null,
        "reasoning": null,
        "tool_selector": null
      }
    }
  }
}"#;

    let config = PluginConfig::from_json(config_json).unwrap();
    assert!(config.modules.contains_key("context"));
}
