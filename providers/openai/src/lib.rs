//! OpenAI-compatible provider for Wireframe-AI.
//!
//! Supports OpenAI API and OpenAI-compatible APIs (DeepSeek, OpenCode Go, etc.)

use anyhow::Result;
use async_trait::async_trait;
use futures::{stream, StreamExt};
use std::sync::Arc;
use wireframe_provider_core::{
    Availability, Diagnostic, EventStream, Message, Provider, ProviderCapabilities,
    ProviderMetadata, ProviderStatus, SetupState, StreamEvent,
    get_http_client,
};

/// OpenAI provider configuration.
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: String,
    pub stream: Option<bool>,
}

/// OpenAI provider implementation.
#[derive(Debug, Clone)]
pub struct OpenAIProvider {
    config: OpenAIConfig,
}

impl OpenAIProvider {
    pub fn new(config: OpenAIConfig) -> Self {
        Self {
            config,
        }
    }

    fn base_url(&self) -> String {
        self.config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string())
    }
    
    /// Parse OpenAI SSE line.
    fn parse_sse_line(&self, line: &str) -> Option<StreamEvent> {
        let line = line.trim();
        if line.is_empty() || line.starts_with(':') {
            return None;
        }
        
        let data = line.strip_prefix("data: ")?;
        
        if data == "[DONE]" {
            return Some(StreamEvent::Done);
        }
        
        let json: serde_json::Value = serde_json::from_str(data).ok()?;
        
        // Extract text delta
        if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
            if !content.is_empty() {
                return Some(StreamEvent::TextDelta { text: content.to_string() });
            }
        }
        
        // Extract tool calls
        if let Some(calls) = json["choices"][0]["delta"]["tool_calls"].as_array() {
            for call in calls {
                if let (Some(id), Some(name)) = (
                    call["id"].as_str(),
                    call["function"]["name"].as_str()
                ) {
                    let args = call["function"]["arguments"].as_str().unwrap_or("");
                    return Some(StreamEvent::ToolCall {
                        id: id.to_string(),
                        name: name.to_string(),
                        arguments: args.to_string(),
                    });
                }
            }
        }
        
        None
    }
}

#[async_trait]
impl Provider for OpenAIProvider {
    async fn complete(
        &self,
        messages: &[Message],
        tools: &[wireframe_provider_core::ToolDefinition],
        system: &str,
        _session_id: Option<&str>,
    ) -> Result<EventStream> {
        let use_streaming = self.config.stream.unwrap_or(true);
        
        // Build request
        let mut request_messages = Vec::new();

        // Add system message if provided
        if !system.is_empty() {
            request_messages.push(serde_json::json!({
                "role": "system",
                "content": system
            }));
        }

        // Add conversation messages
        for msg in messages {
            let json_msg = serde_json::json!({
                "role": msg.role,
                "content": msg.content
            });
            request_messages.push(json_msg);
        }

        let mut request_body = serde_json::json!({
            "model": self.config.model,
            "messages": request_messages,
            "stream": use_streaming
        });

        // Add tools if provided
        if !tools.is_empty() {
            request_body["tools"] = serde_json::to_value(tools)?;
        }

        // Make API call
        let url = format!("{}/chat/completions", self.base_url());
        let mut request = get_http_client().post(&url);

        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }

        if use_streaming {
            // For streaming, we need to read the response body as bytes and parse SSE
            let body_bytes = response.bytes().await?;
            let text = String::from_utf8_lossy(&body_bytes);
            
            // Parse SSE lines and create a stream
            let events: Vec<StreamEvent> = text
                .lines()
                .filter_map(|line| self.parse_sse_line(line))
                .collect();
            
            let stream = stream::iter(events).map(|event| Ok(event));
            Ok(Box::pin(stream) as EventStream)
        } else {
            let response_json: serde_json::Value = response.json().await?;

            // Extract content
            let content = response_json["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string();

            // Extract tool calls if present
            let tool_calls: Vec<(String, String, String)> = response_json["choices"][0]["message"]
                ["tool_calls"]
                .as_array()
                .map(|calls| {
                    calls
                        .iter()
                        .map(|tc| {
                            let id = tc["id"].as_str().unwrap_or("").to_string();
                            let name = tc["function"]["name"].as_str().unwrap_or("").to_string();
                            let arguments = tc["function"]["arguments"]
                                .as_str()
                                .unwrap_or("")
                                .to_string();
                            (id, name, arguments)
                        })
                        .collect()
                })
                .unwrap_or_default();

            // Create stream
            if tool_calls.is_empty() {
                // Only text response
                let stream = stream::once(async move { Ok(StreamEvent::TextDelta { text: content }) });
                Ok(Box::pin(stream) as EventStream)
            } else {
                // Text + tool calls
                let text_stream =
                    stream::once(async move { Ok(StreamEvent::TextDelta { text: content }) });
                let tool_stream = tool_calls.into_iter().map(|(id, name, arguments)| {
                    Ok(StreamEvent::ToolCall {
                        id,
                        name,
                        arguments,
                    })
                });
                let combined = text_stream.chain(stream::iter(tool_stream));
                Ok(Box::pin(combined) as EventStream)
            }
        }
    }

    fn describe(&self) -> ProviderMetadata {
        ProviderMetadata {
            provider_id: "openai".to_string(),
            provider_label: "OpenAI".to_string(),
            provider_version: "0.1.0".to_string(),
            protocol_version: "0.1.0".to_string(),
            transport: "http".to_string(),
            capabilities: ProviderCapabilities {
                core_methods: vec![
                    "complete".to_string(),
                    "describe".to_string(),
                    "status".to_string(),
                ],
                optional_methods: vec!["set_model".to_string(), "available_models".to_string()],
                features: vec!["streaming".to_string(), "tools".to_string()],
                custom_methods: vec![],
            },
        }
    }

    fn status(&self) -> ProviderStatus {
        let diagnostics = if self.config.api_key.is_some() {
            vec![]
        } else {
            vec![Diagnostic {
                level: "warning".to_string(),
                code: "missing_api_key".to_string(),
                message: "API key not set".to_string(),
            }]
        };

        ProviderStatus {
            availability: if self.config.api_key.is_some() {
                Availability::Ready
            } else {
                Availability::Degraded
            },
            setup_state: if self.config.api_key.is_some() {
                SetupState::Complete
            } else {
                SetupState::Required
            },
            requires_manual_setup: self.config.api_key.is_none(),
            diagnostics,
        }
    }

    fn name(&self) -> &str {
        "openai"
    }

    fn model(&self) -> String {
        self.config.model.clone()
    }

    fn set_model(&self, model: &str) -> Result<()> {
        if model.is_empty() {
            return Err(anyhow::anyhow!("Model name cannot be empty"));
        }
        Ok(())
    }

    fn available_models(&self) -> Vec<String> {
        vec![
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-3.5-turbo".to_string(),
        ]
    }

    fn transport(&self) -> String {
        "http".to_string()
    }

    fn fork(&self) -> Arc<dyn Provider> {
        Arc::new(Self::new(self.config.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let config = OpenAIConfig {
            api_key: Some("test-key".to_string()),
            base_url: None,
            model: "gpt-4o".to_string(),
            stream: Some(true),
        };
        let provider = OpenAIProvider::new(config);
        assert_eq!(provider.name(), "openai");
        assert_eq!(provider.model(), "gpt-4o");
    }

    #[test]
    fn test_describe() {
        let config = OpenAIConfig {
            api_key: None,
            base_url: None,
            model: "gpt-4o".to_string(),
            stream: Some(true),
        };
        let provider = OpenAIProvider::new(config);
        let metadata = provider.describe();
        assert_eq!(metadata.provider_id, "openai");
        assert_eq!(metadata.transport, "http");
    }

    #[test]
    fn test_status_with_key() {
        let config = OpenAIConfig {
            api_key: Some("test-key".to_string()),
            base_url: None,
            model: "gpt-4o".to_string(),
            stream: Some(true),
        };
        let provider = OpenAIProvider::new(config);
        let status = provider.status();
        assert!(matches!(status.availability, Availability::Ready));
    }

    #[test]
    fn test_status_without_key() {
        let config = OpenAIConfig {
            api_key: None,
            base_url: None,
            model: "gpt-4o".to_string(),
            stream: Some(true),
        };
        let provider = OpenAIProvider::new(config);
        let status = provider.status();
        assert!(matches!(status.availability, Availability::Degraded));
    }
}
