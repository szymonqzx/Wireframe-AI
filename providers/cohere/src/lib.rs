//! Cohere provider for Wireframe-AI.
//!
//! Uses the Cohere Chat API.

use anyhow::Result;
use async_trait::async_trait;
use futures::{stream, StreamExt};
use reqwest::Client;
use std::sync::Arc;
use wireframe_provider_core::{
    Availability, Diagnostic, EventStream, Message, Provider, ProviderCapabilities,
    ProviderMetadata, ProviderStatus, SetupState, StreamEvent, ToolDefinition,
};

/// Cohere provider configuration.
#[derive(Debug, Clone)]
pub struct CohereConfig {
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
    pub stream: Option<bool>,
}

/// Cohere provider implementation.
#[derive(Debug, Clone)]
pub struct CohereProvider {
    config: CohereConfig,
    client: Client,
}

impl CohereProvider {
    pub fn new(config: CohereConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    fn base_url(&self) -> String {
        self.config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.cohere.com".to_string())
    }

    fn should_stream(&self) -> bool {
        self.config.stream.unwrap_or(true)
    }

    /// Convert wireframe messages to Cohere chat history format.
    /// Cohere uses a `message` (current user message) and `chat_history` (previous turns).
    fn build_messages(
        &self,
        messages: &[Message],
        system: &str,
    ) -> (Option<String>, String, Vec<serde_json::Value>) {
        // Cohere requires a single user message and chat_history for context
        let mut history: Vec<serde_json::Value> = Vec::new();

        for msg in messages.iter().take(messages.len().saturating_sub(1)) {
            let role = match msg.role.as_str() {
                "system" => "SYSTEM",
                "user" => "USER",
                "assistant" => "CHATBOT",
                _ => "USER",
            };
            history.push(serde_json::json!({
                "role": role,
                "message": msg.content
            }));
        }

        let last_message = messages
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();

        let preamble = if system.is_empty() {
            None
        } else {
            Some(system.to_string())
        };

        (preamble, last_message, history)
    }

    /// Build Cohere tool definitions.
    fn build_tools(&self, tools: &[ToolDefinition]) -> Vec<serde_json::Value> {
        tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "parameter_definitions": t.parameters.get("properties").unwrap_or(&serde_json::Value::Object(serde_json::Map::new()))
                })
            })
            .collect()
    }

    /// Extract text from Cohere response.
    fn extract_text(&self, response: &serde_json::Value) -> String {
        response
            .get("text")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string()
    }

    /// Extract tool calls from Cohere response.
    fn extract_tool_calls(&self, response: &serde_json::Value) -> Vec<(String, String, String)> {
        response
            .get("tool_calls")
            .and_then(|tc| tc.as_array())
            .map(|calls| {
                calls
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, call)| {
                        let name = call.get("name").and_then(|n| n.as_str())?;
                        let args = call.get("parameters").unwrap_or(&serde_json::Value::Null);
                        let args_str = serde_json::to_string(args).unwrap_or_default();
                        let id = format!("call_{}_{}", name, idx);
                        Some((id, name.to_string(), args_str))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Parse SSE lines from Cohere streaming response.
    pub fn parse_sse_line(&self, line: &str) -> Option<StreamEvent> {
        if let Some(data) = line.strip_prefix("data: ") {
            if data == "[DONE]" {
                return Some(StreamEvent::Done);
            }

            let json: serde_json::Value = serde_json::from_str(data).ok()?;

            // Extract text delta from Cohere streaming format
            if let Some(event_type) = json.get("event_type").and_then(|e| e.as_str()) {
                if event_type == "text-generation" {
                    if let Some(text) = json.get("text").and_then(|t| t.as_str()) {
                        if !text.is_empty() {
                            return Some(StreamEvent::TextDelta {
                                text: text.to_string(),
                            });
                        }
                    }
                }

                // Extract tool calls from Cohere streaming format
                if event_type == "tool-calls-generation" {
                    if let Some(calls) = json.get("tool_calls").and_then(|c| c.as_array()) {
                        for (idx, call) in calls.iter().enumerate() {
                            if let (Some(name), Some(parameters)) = (
                                call.get("name").and_then(|n| n.as_str()),
                                call.get("parameters"),
                            ) {
                                let args = serde_json::to_string(parameters).ok()?;
                                let id = format!("call_{}_{}", name, idx);
                                return Some(StreamEvent::ToolCall {
                                    id,
                                    name: name.to_string(),
                                    arguments: args,
                                });
                            }
                        }
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
impl Provider for CohereProvider {
    async fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        system: &str,
        _session_id: Option<&str>,
    ) -> Result<EventStream> {
        let use_streaming = self.should_stream();
        let (preamble, message, chat_history) = self.build_messages(messages, system);

        let mut request_body = serde_json::json!({
            "model": self.config.model,
            "message": message,
            "stream": use_streaming
        });

        if let Some(p) = preamble {
            request_body["preamble"] = serde_json::Value::String(p);
        }

        if !chat_history.is_empty() {
            request_body["chat_history"] = serde_json::to_value(chat_history)?;
        }

        if !tools.is_empty() {
            request_body["tools"] = serde_json::to_value(self.build_tools(tools))?;
        }

        let url = format!("{}/v1/chat", self.base_url());
        let mut request = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body);

        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Cohere API error: {}", error_text));
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
                let stream =
                    stream::once(async move { Ok(StreamEvent::TextDelta { text: content }) });
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
            provider_id: "cohere".to_string(),
            provider_label: "Cohere".to_string(),
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
                message: "COHERE_API_KEY not set".to_string(),
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
        "cohere"
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
            "command-r".to_string(),
            "command-r-plus".to_string(),
            "command".to_string(),
            "command-light".to_string(),
        ]
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn cost_per_1k_tokens(&self) -> Option<(u64, u64)> {
        // Cohere pricing (USD cents per 1K tokens)
        match self.config.model.as_str() {
            "command-r" => Some((15, 60)),       // $0.15/$0.60 per 1K
            "command-r-plus" => Some((30, 150)), // $0.30/$1.50 per 1K
            "command" => Some((100, 200)),       // $1.00/$2.00 per 1K
            "command-light" => Some((30, 60)),   // $0.30/$0.60 per 1K
            _ => Some((15, 60)),
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
        let config = CohereConfig {
            api_key: Some("test-key".to_string()),
            model: "command-r".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = CohereProvider::new(config);
        assert_eq!(provider.name(), "cohere");
        assert_eq!(provider.model(), "command-r");
    }

    #[test]
    fn test_describe() {
        let config = CohereConfig {
            api_key: None,
            model: "command-r".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = CohereProvider::new(config);
        let metadata = provider.describe();
        assert_eq!(metadata.provider_id, "cohere");
        assert_eq!(metadata.transport, "http");
    }

    #[test]
    fn test_status_with_key() {
        let config = CohereConfig {
            api_key: Some("test-key".to_string()),
            model: "command-r".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = CohereProvider::new(config);
        let status = provider.status();
        assert!(matches!(status.availability, Availability::Ready));
    }

    #[test]
    fn test_build_messages() {
        let config = CohereConfig {
            api_key: None,
            model: "command-r".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = CohereProvider::new(config);
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
            Message {
                role: "user".to_string(),
                content: "What's up?".to_string(),
                tool_call_id: None,
            },
        ];
        let (preamble, last_msg, history) = provider.build_messages(&messages, "System prompt");
        assert_eq!(preamble, Some("System prompt".to_string()));
        assert_eq!(last_msg, "What's up?");
        assert_eq!(history.len(), 2);
        assert_eq!(history[0]["role"], "USER");
        assert_eq!(history[0]["message"], "Hello");
    }

    #[test]
    fn test_extract_text() {
        let config = CohereConfig {
            api_key: None,
            model: "command-r".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = CohereProvider::new(config);
        let response = serde_json::json!({"text": "Hello world"});
        assert_eq!(provider.extract_text(&response), "Hello world");
    }

    #[test]
    fn test_extract_tool_calls() {
        let config = CohereConfig {
            api_key: None,
            model: "command-r".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = CohereProvider::new(config);
        let response = serde_json::json!({
            "text": "",
            "tool_calls": [
                {
                    "name": "search",
                    "parameters": {"query": "test"}
                }
            ]
        });
        let calls = provider.extract_tool_calls(&response);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].1, "search");
    }

    #[test]
    fn test_cost_per_1k() {
        let config = CohereConfig {
            api_key: None,
            model: "command-r".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = CohereProvider::new(config);
        let cost = provider.cost_per_1k_tokens();
        assert_eq!(cost, Some((15, 60)));
    }
}
