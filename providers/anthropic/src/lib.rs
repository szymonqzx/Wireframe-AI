//! Anthropic Claude provider for Wireframe-AI.
//!
//! Uses the Anthropic Messages API with streaming support.

use anyhow::Result;
use async_trait::async_trait;
use futures::{stream, StreamExt};
use reqwest::Client;
use std::sync::Arc;
use wireframe_provider_core::{
    Availability, Diagnostic, EventStream, Message, Provider, ProviderCapabilities,
    ProviderMetadata, ProviderStatus, SetupState, StreamEvent, ToolDefinition,
};

/// Anthropic provider configuration.
#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
    pub stream: Option<bool>,
}

/// Anthropic provider implementation.
#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    config: AnthropicConfig,
    client: Client,
}

impl AnthropicProvider {
    pub fn new(config: AnthropicConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    fn base_url(&self) -> String {
        self.config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.anthropic.com".to_string())
    }

    fn api_version(&self) -> &str {
        "2023-06-01"
    }

    fn should_stream(&self) -> bool {
        self.config.stream.unwrap_or(true)
    }

    /// Convert wireframe messages to Anthropic format.
    /// Anthropic uses a different message format with explicit system prompt support.
    fn build_messages(
        &self,
        messages: &[Message],
        system: &str,
    ) -> (Option<String>, Vec<serde_json::Value>) {
        let system_prompt = if system.is_empty() {
            None
        } else {
            Some(system.to_string())
        };

        let anthropic_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|msg| {
                let role = match msg.role.as_str() {
                    "system" => "assistant",
                    r => r,
                };
                serde_json::json!({
                    "role": role,
                    "content": msg.content
                })
            })
            .collect();

        (system_prompt, anthropic_messages)
    }

    /// Build Anthropic tool definitions.
    fn build_tools(&self, tools: &[ToolDefinition]) -> Vec<serde_json::Value> {
        tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.parameters
                })
            })
            .collect()
    }

    /// Extract text from Anthropic non-streaming response.
    fn extract_text(&self, response: &serde_json::Value) -> String {
        response
            .get("content")
            .and_then(|c| c.as_array())
            .map(|blocks| {
                blocks
                    .iter()
                    .filter_map(|block| {
                        let block_type = block.get("type").and_then(|t| t.as_str());
                        match block_type {
                            Some("text") => block.get("text").and_then(|t| t.as_str()),
                            _ => None,
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default()
    }

    /// Extract tool calls from Anthropic response.
    fn extract_tool_calls(&self, response: &serde_json::Value) -> Vec<(String, String, String)> {
        response
            .get("content")
            .and_then(|c| c.as_array())
            .map(|blocks| {
                blocks
                    .iter()
                    .filter_map(|block| {
                        let block_type = block.get("type").and_then(|t| t.as_str());
                        match block_type {
                            Some("tool_use") => {
                                let id = block.get("id").and_then(|v| v.as_str())?.to_string();
                                let name = block.get("name").and_then(|v| v.as_str())?.to_string();
                                let args = serde_json::to_string(
                                    block.get("input").unwrap_or(&serde_json::Value::Null),
                                )
                                .unwrap_or_default();
                                Some((id, name, args))
                            }
                            _ => None,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Parse SSE lines from Anthropic streaming response.
    pub fn parse_sse_line(&self, line: &str) -> Option<StreamEvent> {
        if let Some(data) = line.strip_prefix("data: ") {
            if data == "[DONE]" {
                return Some(StreamEvent::Done);
            }

            if let Ok(event) = serde_json::from_str::<serde_json::Value>(data) {
                let event_type = event.get("type").and_then(|v| v.as_str());
                match event_type {
                    Some("content_block_delta") => {
                        if let Some(text) = event
                            .get("delta")
                            .and_then(|d| d.get("text"))
                            .and_then(|t| t.as_str())
                        {
                            return Some(StreamEvent::TextDelta {
                                text: text.to_string(),
                            });
                        }
                        // Tool use delta
                        if let Some(partial_json) = event
                            .get("delta")
                            .and_then(|d| d.get("partial_json"))
                            .and_then(|p| p.as_str())
                        {
                            return Some(StreamEvent::TextDelta {
                                text: partial_json.to_string(),
                            });
                        }
                    }
                    Some("content_block_start") => {
                        if let Some(text) = event
                            .get("content_block")
                            .and_then(|b| b.get("text"))
                            .and_then(|t| t.as_str())
                        {
                            return Some(StreamEvent::TextDelta {
                                text: text.to_string(),
                            });
                        }
                        // Tool use start
                        if let Some(tool_use) = event.get("content_block") {
                            let id = tool_use
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            let name = tool_use
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            return Some(StreamEvent::ToolCall {
                                id,
                                name,
                                arguments: String::new(),
                            });
                        }
                    }
                    Some("message_stop") => {
                        return Some(StreamEvent::Done);
                    }
                    _ => {}
                }
            }
        }
        None
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    async fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        system: &str,
        _session_id: Option<&str>,
    ) -> Result<EventStream> {
        let use_streaming = self.should_stream();
        
        let (system_prompt, anthropic_messages) = self.build_messages(messages, system);

        let mut request_body = serde_json::json!({
            "model": self.config.model,
            "messages": anthropic_messages,
            "max_tokens": 4096,
            "stream": use_streaming
        });

        if let Some(sys) = system_prompt {
            request_body["system"] = serde_json::Value::String(sys);
        }

        if !tools.is_empty() {
            request_body["tools"] = serde_json::to_value(self.build_tools(tools))?;
        }

        let url = format!("{}/v1/messages", self.base_url());
        let mut request = self
            .client
            .post(&url)
            .header("x-api-version", self.api_version())
            .header("Content-Type", "application/json");

        if let Some(api_key) = &self.config.api_key {
            request = request.header("x-api-key", api_key);
        }

        let response = request.json(&request_body).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Anthropic API error: {}", error_text));
        }

        if use_streaming {
            let body_bytes = response.bytes().await?;
            let text = String::from_utf8_lossy(&body_bytes);
            
            let events: Vec<StreamEvent> = text
                .lines()
                .filter_map(|line| self.parse_sse_line(line))
                .collect();
            
            let stream = stream::iter(events).map(|event| Ok(event));
            Ok(Box::pin(stream) as EventStream)
        } else {
            let response_json: serde_json::Value = response.json().await?;
            let content = self.extract_text(&response_json);
            let tool_calls = self.extract_tool_calls(&response_json);

            if tool_calls.is_empty() {
                let stream = stream::once(async move { Ok(StreamEvent::TextDelta { text: content }) });
                Ok(Box::pin(stream) as EventStream)
            } else {
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
            provider_id: "anthropic".to_string(),
            provider_label: "Anthropic Claude".to_string(),
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
                message: "ANTHROPIC_API_KEY not set".to_string(),
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
        "anthropic"
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
            "claude-3-5-sonnet-20241022".to_string(),
            "claude-3-5-haiku-20241022".to_string(),
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
        ]
    }

    fn cost_per_1k_tokens(&self) -> Option<(u64, u64)> {
        // Anthropic pricing (USD cents per 1K tokens)
        // Claude 3.5 Sonnet: $3/$15 per 1M tokens = 0.3/1.5 cents per 1K
        match self.config.model.as_str() {
            "claude-3-5-sonnet-20241022" => Some((30, 150)),
            "claude-3-5-haiku-20241022" => Some((8, 40)),
            "claude-3-opus-20240229" => Some((150, 750)),
            "claude-3-sonnet-20240229" => Some((30, 150)),
            "claude-3-haiku-20240307" => Some((8, 40)),
            _ => Some((30, 150)),
        }
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
        let config = AnthropicConfig {
            api_key: Some("test-key".to_string()),
            model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = AnthropicProvider::new(config);
        assert_eq!(provider.name(), "anthropic");
        assert_eq!(provider.model(), "claude-3-5-sonnet-20241022");
    }

    #[test]
    fn test_describe() {
        let config = AnthropicConfig {
            api_key: None,
            model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = AnthropicProvider::new(config);
        let metadata = provider.describe();
        assert_eq!(metadata.provider_id, "anthropic");
        assert_eq!(metadata.transport, "http");
        assert!(metadata
            .capabilities
            .features
            .contains(&"streaming".to_string()));
    }

    #[test]
    fn test_status_with_key() {
        let config = AnthropicConfig {
            api_key: Some("test-key".to_string()),
            model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = AnthropicProvider::new(config);
        let status = provider.status();
        assert!(matches!(status.availability, Availability::Ready));
    }

    #[test]
    fn test_status_without_key() {
        let config = AnthropicConfig {
            api_key: None,
            model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = AnthropicProvider::new(config);
        let status = provider.status();
        assert!(matches!(status.availability, Availability::Degraded));
    }

    #[test]
    fn test_build_messages() {
        let config = AnthropicConfig {
            api_key: None,
            model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = AnthropicProvider::new(config);
        let messages = vec![
            Message {
                role: "user".to_string(),
                content: "Hello".to_string(),
                tool_call_id: None,
            },
            Message {
                role: "assistant".to_string(),
                content: "Hi there".to_string(),
                tool_call_id: None,
            },
        ];
        let (system, anthropic_msgs) = provider.build_messages(&messages, "You are helpful");
        assert_eq!(system, Some("You are helpful".to_string()));
        assert_eq!(anthropic_msgs.len(), 2);
        assert_eq!(anthropic_msgs[0]["role"], "user");
        assert_eq!(anthropic_msgs[1]["role"], "assistant");
    }

    #[test]
    fn test_parse_sse_text_delta() {
        let config = AnthropicConfig {
            api_key: None,
            model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = AnthropicProvider::new(config);

        let line = r#"data: {"type":"content_block_delta","index":0,"delta":{"text":"Hello"}}"#;
        let event = provider.parse_sse_line(line);
        assert!(matches!(event, Some(StreamEvent::TextDelta { text }) if text == "Hello"));
    }

    #[test]
    fn test_parse_sse_done() {
        let config = AnthropicConfig {
            api_key: None,
            model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = AnthropicProvider::new(config);

        let line = r#"data: {"type":"message_stop"}"#;
        let event = provider.parse_sse_line(line);
        assert!(matches!(event, Some(StreamEvent::Done)));
    }

    #[test]
    fn test_cost_per_1k() {
        let config = AnthropicConfig {
            api_key: None,
            model: "claude-3-5-sonnet-20241022".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = AnthropicProvider::new(config);
        let cost = provider.cost_per_1k_tokens();
        assert_eq!(cost, Some((30, 150)));
    }
}
