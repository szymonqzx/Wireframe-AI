use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::context::MemoryBackend;
use memory_fts5::FTS5MemoryPlugin;
use tempfile::NamedTempFile;

#[tokio::test]
async fn test_memory_plugin_lifecycle() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap().to_string();

    let mut plugin = FTS5MemoryPlugin::new(db_path.clone());
    let config = serde_json::json!({ "db_path": db_path });

    assert!(plugin.initialize(&config).await.is_ok());
    assert!(plugin.health_check().await.is_ok());
    assert!(plugin.shutdown().await.is_ok());
}

#[tokio::test]
async fn test_persist_and_load_chunks() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap().to_string();

    let mut plugin = FTS5MemoryPlugin::new(db_path.clone());
    let config = serde_json::json!({ "db_path": db_path });
    plugin.initialize(&config).await.unwrap();

    plugin
        .persist_chunk("test_session", "User prefers Python", "user_preference")
        .await
        .unwrap();
    plugin
        .persist_chunk(
            "test_session",
            "Architecture uses microservices",
            "decision",
        )
        .await
        .unwrap();

    let chunks = plugin.load_chunks("test_session", 10).await.unwrap();
    assert_eq!(chunks.len(), 2);
}

#[tokio::test]
async fn test_search_memory() {
    let temp_file = NamedTempFile::new().unwrap();
    let db_path = temp_file.path().to_str().unwrap().to_string();

    let mut plugin = FTS5MemoryPlugin::new(db_path.clone());
    let config = serde_json::json!({ "db_path": db_path });
    plugin.initialize(&config).await.unwrap();

    // First, we need to insert into FTS5 table directly for search to work
    let conn = rusqlite::Connection::open(db_path).unwrap();
    conn.execute(
        "INSERT INTO memory_fts (rowid, session_id, content, role) VALUES (1, 'test_session', 'authentication JWT tokens', 'user')",
        [],
    ).unwrap();

    let results = plugin
        .search("authentication", "test_session", 10)
        .await
        .unwrap();
    assert!(!results.is_empty());
}
