use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::context::StorageBackend;
use storage_sqlite::SQLiteStoragePlugin;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_storage_plugin_lifecycle() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap().to_string();

    let mut plugin = SQLiteStoragePlugin::new(db_path.clone());
    let config = serde_json::json!({ "db_path": db_path });

    assert!(plugin.initialize(&config).await.is_ok());
    assert!(plugin.health_check().await.is_ok());
    assert!(plugin.shutdown().await.is_ok());
}

#[tokio::test]
async fn test_ensure_session() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap().to_string();

    let mut plugin = SQLiteStoragePlugin::new(db_path.clone());
    let config = serde_json::json!({ "db_path": db_path });
    plugin.initialize(&config).await.unwrap();

    assert!(plugin.ensure_session("test_session").await.is_ok());
    assert!(plugin.ensure_session("test_session").await.is_ok()); // Should not error on duplicate
}

#[tokio::test]
async fn test_store_and_load_messages() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap().to_string();

    let mut plugin = SQLiteStoragePlugin::new(db_path.clone());
    let config = serde_json::json!({ "db_path": db_path });
    plugin.initialize(&config).await.unwrap();

    plugin.ensure_session("test_session").await.unwrap();
    plugin
        .store_message("test_session", "user", "Hello")
        .await
        .unwrap();
    plugin
        .store_message("test_session", "assistant", "Hi there!")
        .await
        .unwrap();

    let history = plugin
        .load_session_history("test_session", 10)
        .await
        .unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].content, "Hello");
    assert_eq!(history[1].content, "Hi there!");
}
