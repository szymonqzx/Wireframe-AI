//! Google Gemini provider for Wireframe-AI.
//!
//! Uses the Google Gemini API (generateContent).

use anyhow::Result;
use async_trait::async_trait;
use futures::{stream, StreamExt};
use reqwest::Client;
use std::sync::Arc;
use wireframe_provider_core::{
    Availability, Diagnostic, EventStream, Message, Provider, ProviderCapabilities,
    ProviderMetadata, ProviderStatus, SetupState, StreamEvent, ToolDefinition,
};

/// Google Gemini provider configuration.
#[derive(Debug, Clone)]
pub struct GeminiConfig {
    pub api_key: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
    pub stream: Option<bool>,
}

/// Google Gemini provider implementation.
#[derive(Debug, Clone)]
pub struct GeminiProvider {
    config: GeminiConfig,
    client: Client,
}

impl GeminiProvider {
    pub fn new(config: GeminiConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    fn base_url(&self) -> String {
        self.config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string())
    }

    fn model_path(&self) -> String {
        format!("models/{}:generateContent", self.config.model)
    }

    fn should_stream(&self) -> bool {
        self.config.stream.unwrap_or(true)
    }

    /// Convert wireframe messages to Gemini format.
    /// Gemini uses a single systemInstruction field and contents array.
    fn build_contents(
        &self,
        messages: &[Message],
        system: &str,
    ) -> (Option<serde_json::Value>, Vec<serde_json::Value>) {
        let system_instruction = if system.is_empty() {
            None
        } else {
            Some(serde_json::json!({
                "parts": [{ "text": system }]
            }))
        };

        let contents: Vec<serde_json::Value> = messages
            .iter()
            .map(|msg| {
                let role = match msg.role.as_str() {
                    "system" => "user",
                    "assistant" => "model",
                    r => r,
                };
                serde_json::json!({
                    "role": role,
                    "parts": [{ "text": msg.content }]
                })
            })
            .collect();

        (system_instruction, contents)
    }

    /// Build Gemini tool declarations (function declarations).
    fn build_tools(&self, tools: &[ToolDefinition]) -> Vec<serde_json::Value> {
        tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters
                })
            })
            .collect()
    }

    /// Extract text from Gemini response.
    fn extract_text(&self, response: &serde_json::Value) -> String {
        response
            .get("candidates")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|c| c.get("content"))
            .and_then(|content| content.get("parts"))
            .and_then(|parts| parts.as_array())
            .map(|parts| {
                parts
                    .iter()
                    .filter_map(|part| part.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default()
    }

    /// Extract tool calls from Gemini response.
    fn extract_tool_calls(&self, response: &serde_json::Value) -> Vec<(String, String, String)> {
        response
            .get("candidates")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|c| c.get("content"))
            .and_then(|content| content.get("parts"))
            .and_then(|parts| parts.as_array())
            .map(|parts| {
                parts
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, part)| {
                        let func_call = part.get("functionCall")?;
                        let name = func_call.get("name").and_then(|n| n.as_str())?.to_string();
                        let args = serde_json::to_string(
                            func_call.get("args").unwrap_or(&serde_json::Value::Null),
                        )
                        .unwrap_or_default();
                        let id = format!("call_{}_{}", name, idx);
                        Some((id, name, args))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Extract usage metadata from Gemini response.
    #[allow(dead_code)]
    fn extract_usage(&self, response: &serde_json::Value) -> (usize, usize) {
        let metadata = response.get("usageMetadata");
        let prompt_tokens = metadata
            .and_then(|m| m.get("promptTokenCount"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        let completion_tokens = metadata
            .and_then(|m| m.get("candidatesTokenCount"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;
        (prompt_tokens, completion_tokens)
    }

    /// Parse SSE lines from Gemini streaming response.
    pub fn parse_sse_line(&self, line: &str) -> Option<StreamEvent> {
        if let Some(data) = line.strip_prefix("data: ") {
            if data == "[DONE]" {
                return Some(StreamEvent::Done);
            }

            let json: serde_json::Value = serde_json::from_str(data).ok()?;

            // Extract text delta from Gemini streaming format
            if let Some(candidates) = json.get("candidates").and_then(|c| c.as_array()) {
                if let Some(first) = candidates.first() {
                    if let Some(content) = first.get("content") {
                        if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                            if let Some(first_part) = parts.first() {
                                if let Some(text) = first_part.get("text").and_then(|t| t.as_str())
                                {
                                    if !text.is_empty() {
                                        return Some(StreamEvent::TextDelta {
                                            text: text.to_string(),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Extract tool calls from Gemini streaming format
            if let Some(candidates) = json.get("candidates").and_then(|c| c.as_array()) {
                if let Some(first) = candidates.first() {
                    if let Some(content) = first.get("content") {
                        if let Some(parts) = content.get("parts").and_then(|p| p.as_array()) {
                            for (idx, part) in parts.iter().enumerate() {
                                if let Some(func_call) = part.get("functionCall") {
                                    let name = func_call.get("name").and_then(|n| n.as_str())?;
                                    let args = serde_json::to_string(
                                        func_call.get("args").unwrap_or(&serde_json::Value::Null),
                                    )
                                    .ok()?;
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
            }

            None
        } else {
            None
        }
    }
}

#[async_trait]
impl Provider for GeminiProvider {
    async fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        system: &str,
        _session_id: Option<&str>,
    ) -> Result<EventStream> {
        let use_streaming = self.should_stream();
        let (system_instruction, contents) = self.build_contents(messages, system);

        let mut request_body = serde_json::json!({
            "contents": contents,
            "generationConfig": {
                "maxOutputTokens": 4096,
                "temperature": 0.7
            }
        });

        if let Some(sys) = system_instruction {
            request_body["systemInstruction"] = sys;
        }

        if !tools.is_empty() {
            let declarations = self.build_tools(tools);
            request_body["tools"] = serde_json::json!([{
                "functionDeclarations": declarations
            }]);
        }

        // Use streamGenerateContent for streaming, generateContent for non-streaming
        let model_path = if use_streaming {
            format!("models/{}:streamGenerateContent", self.config.model)
        } else {
            self.model_path()
        };

        let url = format!(
            "{}/{}?key={}",
            self.base_url(),
            model_path,
            self.config.api_key.as_deref().unwrap_or("")
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!("Gemini API error: {}", error_text));
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

            // Check for blocked content
            if let Some(blocked) = response_json
                .get("promptFeedback")
                .and_then(|f| f.get("blockReason"))
            {
                return Err(anyhow::anyhow!(
                    "Gemini blocked the prompt: {}",
                    blocked.as_str().unwrap_or("unknown")
                ));
            }

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
            provider_id: "google".to_string(),
            provider_label: "Google Gemini".to_string(),
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
                message: "GOOGLE_API_KEY not set".to_string(),
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
        "google"
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
            "gemini-2.0-flash-exp".to_string(),
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
            "gemini-1.0-pro".to_string(),
        ]
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn cost_per_1k_tokens(&self) -> Option<(u64, u64)> {
        // Google pricing (USD cents per 1K tokens)
        match self.config.model.as_str() {
            "gemini-2.0-flash-exp" => Some((0, 0)), // Free during experimental
            "gemini-1.5-pro" => Some((35, 175)),    // $3.50/$17.50 per 1M
            "gemini-1.5-flash" => Some((8, 30)),    // $0.80/$3.00 per 1M
            "gemini-1.0-pro" => Some((25, 50)),     // $2.50/$5.00 per 1M
            _ => Some((35, 175)),
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
        let config = GeminiConfig {
            api_key: Some("test-key".to_string()),
            model: "gemini-1.5-pro".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = GeminiProvider::new(config);
        assert_eq!(provider.name(), "google");
        assert_eq!(provider.model(), "gemini-1.5-pro");
    }

    #[test]
    fn test_describe() {
        let config = GeminiConfig {
            api_key: None,
            model: "gemini-1.5-pro".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = GeminiProvider::new(config);
        let metadata = provider.describe();
        assert_eq!(metadata.provider_id, "google");
        assert_eq!(metadata.transport, "http");
    }

    #[test]
    fn test_status_with_key() {
        let config = GeminiConfig {
            api_key: Some("test-key".to_string()),
            model: "gemini-1.5-pro".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = GeminiProvider::new(config);
        let status = provider.status();
        assert!(matches!(status.availability, Availability::Ready));
    }

    #[test]
    fn test_build_contents() {
        let config = GeminiConfig {
            api_key: None,
            model: "gemini-1.5-pro".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = GeminiProvider::new(config);
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
        let (system, contents) = provider.build_contents(&messages, "You are helpful");
        assert!(system.is_some());
        assert_eq!(contents.len(), 2);
        assert_eq!(contents[0]["role"], "user");
        assert_eq!(contents[1]["role"], "model");
    }

    #[test]
    fn test_extract_text() {
        let config = GeminiConfig {
            api_key: None,
            model: "gemini-1.5-pro".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = GeminiProvider::new(config);
        let response = serde_json::json!({
            "candidates": [{
                "content": {
                    "parts": [{"text": "Hello world"}]
                }
            }]
        });
        assert_eq!(provider.extract_text(&response), "Hello world");
    }

    #[test]
    fn test_extract_text_empty() {
        let config = GeminiConfig {
            api_key: None,
            model: "gemini-1.5-pro".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = GeminiProvider::new(config);
        let response = serde_json::json!({ "candidates": [] });
        assert_eq!(provider.extract_text(&response), "");
    }

    #[test]
    fn test_cost_per_1k() {
        let config = GeminiConfig {
            api_key: None,
            model: "gemini-1.5-pro".to_string(),
            base_url: None,
            stream: Some(true),
        };
        let provider = GeminiProvider::new(config);
        let cost = provider.cost_per_1k_tokens();
        assert_eq!(cost, Some((35, 175)));
    }
}
