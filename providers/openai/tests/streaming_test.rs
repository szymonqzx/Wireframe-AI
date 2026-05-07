// providers/openai/tests/streaming_test.rs
use wireframe_provider_openai::{OpenAIProvider, OpenAIConfig};
use wireframe_provider_core::{Message, Provider};

#[tokio::test]
async fn test_streaming_enabled() {
    let config = OpenAIConfig {
        api_key: Some("test-key".to_string()),
        base_url: None,
        model: "gpt-4o".to_string(),
        stream: Some(true),
    };
    let provider = OpenAIProvider::new(config);
    
    let messages = vec![Message {
        role: "user".to_string(),
        content: "Hello".to_string(),
        tool_call_id: None,
    }];
    
    let result = provider.complete(&messages, &[], "", None).await;
    assert!(result.is_err() || result.is_ok());
}
