//! Ollama local model provider for Wireframe-AI.
//!
//! Uses Ollama's OpenAI-compatible API (default port 11434).

use anyhow::Result;
use async_trait::async_trait;
use futures::{stream, StreamExt};
use reqwest::Client;
use std::sync::Arc;
use wireframe_provider_core::{
    Availability, Diagnostic, EventStream, Message, Provider, ProviderCapabilities,
    ProviderMetadata, ProviderStatus, SetupState, StreamEvent, ToolDefinition,
};

/// Ollama provider configuration.
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub model: String,
    pub base_url: Option<String>,
    pub stream: Option<bool>,
}

/// Ollama provider implementation (OpenAI-compatible API).
#[derive(Debug, Clone)]
pub struct OllamaProvider {
    config: OllamaConfig,
    client: Client,
}

impl OllamaProvider {
    pub fn new(config: OllamaConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    fn base_url(&self) -> String {
        self.config
            .base_url
            .clone()
            .unwrap_or_else(|| "http://localhost:11434/v1".to_string())
    }

    fn should_stream(&self) -> bool {
        self.config.stream.unwrap_or(true)
    }

    /// Convert wireframe messages to OpenAI format.
    fn build_messages(&self, messages: &[Message]) -> Vec<serde_json::Value> {
        messages
            .iter()
            .map(|msg| {
                serde_json::json!({
                    "role": msg.role,
                    "content": msg.content
                })
            })
            .collect()
    }

    /// Build Ollama/OpenAI tool definitions.
    fn build_tools(&self, tools: &[ToolDefinition]) -> Vec<serde_json::Value> {
        tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters
                    }
                })
            })
            .collect()
    }

    /// Check if Ollama server is running.
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/models", self.base_url());
        match self.client.get(&url).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Extract text from Ollama non-streaming response.
    fn extract_text(&self, response: &serde_json::Value) -> String {
        response
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|msg| msg.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string()
    }

    /// Extract tool calls from Ollama response.
    fn extract_tool_calls(&self, response: &serde_json::Value) -> Vec<(String, String, String)> {
        response
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|msg| msg.get("tool_calls"))
            .and_then(|tc| tc.as_array())
            .map(|calls| {
                calls
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, call)| {
                        let func = call.get("function")?;
                        let name = func.get("name").and_then(|n| n.as_str())?.to_string();
                        let args = func
                            .get("arguments")
                            .and_then(|a| a.as_str())
                            .map(|s| s.to_string())
                            .or_else(|| {
                                serde_json::to_string(
                                    func.get("arguments").unwrap_or(&serde_json::Value::Null),
                                )
                                .ok()
                            })
                            .unwrap_or_default();
                        let id = format!("call_{}_{}", name, idx);
                        Some((id, name, args))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Parse SSE lines from Ollama streaming response (OpenAI-compatible format).
    pub fn parse_sse_line(&self, line: &str) -> Option<StreamEvent> {
        if let Some(data) = line.strip_prefix("data: ") {
            if data == "[DONE]" {
                return Some(StreamEvent::Done);
            }

            let json: serde_json::Value = serde_json::from_str(data).ok()?;

            // Extract text delta (OpenAI-compatible format)
            if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                if !content.is_empty() {
                    return Some(StreamEvent::TextDelta {
                        text: content.to_string(),
                    });
                }
            }

            // Extract tool calls (OpenAI-compatible format)
            if let Some(calls) = json["choices"][0]["delta"]["tool_calls"].as_array() {
                for call in calls {
                    if let (Some(id), Some(name)) = (
                        call["id"].as_str(),
                        call["function"]["name"].as_str(),
                    ) {
                        let args = call["function"]["arguments"]
                            .as_str()
                            .unwrap_or("");
                        return Some(StreamEvent::ToolCall {
                            id: id.to_string(),
                            name: name.to_string(),
                            arguments: args.to_string(),
                        });
                    }
                }
            }

            None
        } else {
            None
        }
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    async fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        system: &str,
        _session_id: Option<&str>,
    ) -> Result<EventStream> {
        let use_streaming = self.should_stream();
        let mut openai_messages = self.build_messages(messages);

        // Inject system prompt if not already present
        if !system.is_empty()
            && !openai_messages
                .iter()
                .any(|m| m.get("role") == Some(&serde_json::Value::String("system".to_string())))
        {
            openai_messages.insert(
                0,
                serde_json::json!({
                    "role": "system",
                    "content": system
                }),
            );
        }

        let mut request_body = serde_json::json!({
            "model": self.config.model,
            "messages": openai_messages,
            "stream": use_streaming
        });

        if !tools.is_empty() {
            request_body["tools"] = serde_json::to_value(self.build_tools(tools))?;
        }

        let url = format!("{}/chat/completions", self.base_url());
        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "Ollama API error (model may not be pulled): {}",
                error_text
            ));
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
            provider_id: "ollama".to_string(),
            provider_label: "Ollama (Local)".to_string(),
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
                features: vec!["tools".to_string(), "streaming".to_string()],
                custom_methods: vec![wireframe_provider_core::CustomMethod {
                    name: "health_check".to_string(),
                    stability: "experimental".to_string(),
                    description: "Check Ollama server health".to_string(),
                }],
            },
        }
    }

    fn status(&self) -> ProviderStatus {
        ProviderStatus {
            availability: Availability::Ready, // Assume ready; health checked at runtime
            setup_state: SetupState::Complete,
            requires_manual_setup: false,
            diagnostics: vec![Diagnostic {
                level: "info".to_string(),
                code: "local_server".to_string(),
                message: format!("Requires Ollama server at {}", self.base_url()),
            }],
        }
    }

    fn name(&self) -> &str {
        "ollama"
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
            "llama3.2".to_string(),
            "llama3.1".to_string(),
            "mistral".to_string(),
            "qwen2.5".to_string(),
            "phi4".to_string(),
            "deepseek-coder".to_string(),
        ]
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn cost_per_1k_tokens(&self) -> Option<(u64, u64)> {
        // Local models are free (0 cost)
        Some((0, 0))
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
        let config = OllamaConfig {
            model: "llama3.2".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = OllamaProvider::new(config);
        assert_eq!(provider.name(), "ollama");
        assert_eq!(provider.model(), "llama3.2");
        assert_eq!(provider.base_url(), "http://localhost:11434/v1");
    }

    #[test]
    fn test_provider_custom_url() {
        let config = OllamaConfig {
            model: "mistral".to_string(),
            base_url: Some("http://192.168.1.100:11434/v1".to_string()),
            stream: Some(true),
        };
        let provider = OllamaProvider::new(config);
        assert_eq!(provider.base_url(), "http://192.168.1.100:11434/v1");
    }

    #[test]
    fn test_describe() {
        let config = OllamaConfig {
            model: "llama3.2".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = OllamaProvider::new(config);
        let metadata = provider.describe();
        assert_eq!(metadata.provider_id, "ollama");
        assert_eq!(metadata.transport, "http");
        assert!(metadata
            .capabilities
            .custom_methods
            .iter()
            .any(|m| m.name == "health_check"));
    }

    #[test]
    fn test_build_messages() {
        let config = OllamaConfig {
            model: "llama3.2".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = OllamaProvider::new(config);
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
        let msgs = provider.build_messages(&messages);
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0]["role"], "user");
        assert_eq!(msgs[1]["role"], "assistant");
    }

    #[test]
    fn test_extract_text() {
        let config = OllamaConfig {
            model: "llama3.2".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = OllamaProvider::new(config);
        let response = serde_json::json!({
            "choices": [{
                "message": {
                    "content": "Hello world"
                }
            }]
        });
        assert_eq!(provider.extract_text(&response), "Hello world");
    }

    #[test]
    fn test_extract_tool_calls() {
        let config = OllamaConfig {
            model: "llama3.2".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = OllamaProvider::new(config);
        let response = serde_json::json!({
            "choices": [{
                "message": {
                    "content": "",
                    "tool_calls": [{
                        "function": {
                            "name": "search",
                            "arguments": "{\"query\": \"test\"}"
                        }
                    }]
                }
            }]
        });
        let calls = provider.extract_tool_calls(&response);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].1, "search");
        assert_eq!(calls[0].2, "{\"query\": \"test\"}");
    }

    #[test]
    fn test_cost_per_1k() {
        let config = OllamaConfig {
            model: "llama3.2".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = OllamaProvider::new(config);
        let cost = provider.cost_per_1k_tokens();
        assert_eq!(cost, Some((0, 0)));
    }
}
