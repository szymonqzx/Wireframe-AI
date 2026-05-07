use agentic_sdk::message_types::{ContextPackage, TaskSubmitted};
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::context::EnrichmentStrategy;
use enrichment_env::EnvEnrichmentPlugin;

#[tokio::test]
async fn test_enrichment_plugin_lifecycle() {
    let mut plugin = EnvEnrichmentPlugin::new();
    let config = serde_json::json!({});

    assert!(plugin.initialize(&config).await.is_ok());
    assert!(plugin.health_check().await.is_ok());
    assert!(plugin.shutdown().await.is_ok());
}

#[tokio::test]
async fn test_enrich_filters_secrets() {
    let plugin = EnvEnrichmentPlugin::new();
    std::env::set_var("API_KEY", "secret123");
    std::env::set_var("DATABASE_URL", "postgres://localhost");

    let task = TaskSubmitted {
        session_id: "test".to_string(),
        user_input: "test".to_string(),
        submitted_at: 0,
    };
    let context = ContextPackage {
        memory_chunks: vec![],
        session_history: vec![],
        readonly_files: vec![],
        safe_env: std::collections::HashMap::new(),
        working_dir: std::env::current_dir().unwrap(),
        max_context_tokens: 1000,
    };

    let enriched = plugin.enrich(&task, &context).await.unwrap();
    assert!(!enriched.safe_env.contains_key("API_KEY"));
    assert!(enriched.safe_env.contains_key("DATABASE_URL"));

    std::env::remove_var("API_KEY");
    std::env::remove_var("DATABASE_URL");
}
