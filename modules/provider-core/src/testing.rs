//! Provider testing framework for Wireframe-AI.
//!
//! Provides utilities for testing provider implementations,
//! including mock providers, test fixtures, and conformance checks.

use crate::{
    EventStream, Message, Provider, ProviderMetadata, ProviderStatus, StreamEvent, ToolDefinition,
};
use anyhow::Result;
use async_trait::async_trait;
use futures::stream;
use std::sync::Arc;

/// A mock provider for testing adapter logic without calling real APIs.
pub struct MockProvider {
    name: String,
    model: String,
    response_text: String,
    metadata: ProviderMetadata,
    status: ProviderStatus,
    should_fail: bool,
}

impl MockProvider {
    pub fn new(name: impl Into<String>, model: impl Into<String>) -> Self {
        let name = name.into();
        let model = model.into();
        Self {
            name: name.clone(),
            model: model.clone(),
            response_text: "Mock response".to_string(),
            metadata: ProviderMetadata {
                provider_id: name.clone(),
                provider_label: format!("Mock {}", name),
                provider_version: "0.1.0".to_string(),
                protocol_version: "0.1.0".to_string(),
                transport: "mock".to_string(),
                capabilities: crate::ProviderCapabilities {
                    core_methods: vec![
                        "complete".to_string(),
                        "describe".to_string(),
                        "status".to_string(),
                    ],
                    optional_methods: vec![],
                    features: vec!["tools".to_string()],
                    custom_methods: vec![],
                },
            },
            status: ProviderStatus {
                availability: crate::Availability::Ready,
                setup_state: crate::SetupState::Complete,
                requires_manual_setup: false,
                diagnostics: vec![],
            },
            should_fail: false,
        }
    }

    pub fn with_response(mut self, text: impl Into<String>) -> Self {
        self.response_text = text.into();
        self
    }

    pub fn fail_next(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[async_trait]
impl Provider for MockProvider {
    async fn complete(
        &self,
        _messages: &[Message],
        _tools: &[ToolDefinition],
        _system: &str,
        _session_id: Option<&str>,
    ) -> Result<EventStream> {
        if self.should_fail {
            return Err(anyhow::anyhow!("Mock provider failure"));
        }
        let text = self.response_text.clone();
        let stream = stream::once(async move { Ok(StreamEvent::TextDelta { text }) });
        Ok(Box::pin(stream) as EventStream)
    }

    fn describe(&self) -> ProviderMetadata {
        self.metadata.clone()
    }

    fn status(&self) -> ProviderStatus {
        self.status.clone()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn model(&self) -> String {
        self.model.clone()
    }

    fn fork(&self) -> Arc<dyn Provider> {
        Arc::new(Self {
            name: self.name.clone(),
            model: self.model.clone(),
            response_text: self.response_text.clone(),
            metadata: self.metadata.clone(),
            status: self.status.clone(),
            should_fail: false,
        })
    }
}

/// Test harness for provider conformance testing.
pub struct ProviderTestHarness {
    pub provider: Arc<dyn Provider>,
    pub results: Vec<TestResult>,
}

/// Result of a single conformance test.
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub message: String,
}

impl ProviderTestHarness {
    pub fn new(provider: Arc<dyn Provider>) -> Self {
        Self {
            provider,
            results: vec![],
        }
    }

    /// Run all conformance tests.
    pub async fn run_all(&mut self) {
        self.test_describe().await;
        self.test_status().await;
        self.test_complete_empty().await;
        self.test_complete_with_messages().await;
        self.test_name().await;
        self.test_model().await;
    }

    async fn test_describe(&mut self) {
        let meta = self.provider.describe();
        let passed = !meta.provider_id.is_empty() && !meta.provider_label.is_empty();
        self.results.push(TestResult {
            test_name: "describe".to_string(),
            passed,
            message: if passed {
                format!(
                    "Provider ID: {}, Label: {}",
                    meta.provider_id, meta.provider_label
                )
            } else {
                "Missing provider_id or provider_label".to_string()
            },
        });
    }

    async fn test_status(&mut self) {
        let status = self.provider.status();
        let passed = matches!(
            status.availability,
            crate::Availability::Ready
                | crate::Availability::Degraded
                | crate::Availability::Unavailable
        );
        self.results.push(TestResult {
            test_name: "status".to_string(),
            passed,
            message: format!("Availability: {:?}", status.availability),
        });
    }

    async fn test_complete_empty(&mut self) {
        let result = self.provider.complete(&[], &[], "", None).await;
        let passed = result.is_ok();
        self.results.push(TestResult {
            test_name: "complete_empty".to_string(),
            passed,
            message: if passed {
                "OK".to_string()
            } else {
                "Failed with empty input".to_string()
            },
        });
    }

    async fn test_complete_with_messages(&mut self) {
        let messages = vec![Message {
            role: "user".to_string(),
            content: "Hello".to_string(),
            tool_call_id: None,
        }];
        let result = self.provider.complete(&messages, &[], "", None).await;
        let passed = result.is_ok();
        self.results.push(TestResult {
            test_name: "complete_with_messages".to_string(),
            passed,
            message: if passed {
                "OK".to_string()
            } else {
                "Failed with messages".to_string()
            },
        });
    }

    async fn test_name(&mut self) {
        let name = self.provider.name();
        let passed = !name.is_empty();
        self.results.push(TestResult {
            test_name: "name".to_string(),
            passed,
            message: format!("Name: {}", name),
        });
    }

    async fn test_model(&mut self) {
        let model = self.provider.model();
        let passed = !model.is_empty();
        self.results.push(TestResult {
            test_name: "model".to_string(),
            passed,
            message: format!("Model: {}", model),
        });
    }

    /// Print a summary of all test results.
    pub fn print_summary(&self) {
        let passed = self.results.iter().filter(|r| r.passed).count();
        let total = self.results.len();
        println!(
            "\nProvider Conformance Test Results: {}/{} passed",
            passed, total
        );
        for result in &self.results {
            let icon = if result.passed { "PASS" } else { "FAIL" };
            println!("  [{}] {}: {}", icon, result.test_name, result.message);
        }
    }
}

/// Test fixtures for common message patterns.
pub mod fixtures {
    use super::Message;

    pub fn simple_user_message(content: impl Into<String>) -> Message {
        Message {
            role: "user".to_string(),
            content: content.into(),
            tool_call_id: None,
        }
    }

    pub fn assistant_message(content: impl Into<String>) -> Message {
        Message {
            role: "assistant".to_string(),
            content: content.into(),
            tool_call_id: None,
        }
    }

    pub fn system_message(content: impl Into<String>) -> Message {
        Message {
            role: "system".to_string(),
            content: content.into(),
            tool_call_id: None,
        }
    }

    pub fn tool_result_message(
        tool_call_id: impl Into<String>,
        content: impl Into<String>,
    ) -> Message {
        Message {
            role: "tool".to_string(),
            content: content.into(),
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_mock_provider_responds() {
        let provider = MockProvider::new("test", "test-model").with_response("Hello!");
        let messages = vec![fixtures::simple_user_message("Hi")];
        let mut stream = provider.complete(&messages, &[], "", None).await.unwrap();
        let event = stream.next().await.unwrap().unwrap();
        assert!(matches!(event, StreamEvent::TextDelta { text } if text == "Hello!"));
    }

    #[tokio::test]
    async fn test_mock_provider_failure() {
        let provider = MockProvider::new("test", "test-model").fail_next();
        let result = provider.complete(&[], &[], "", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_harness_runs() {
        let provider = Arc::new(MockProvider::new("test", "test-model")) as Arc<dyn Provider>;
        let mut harness = ProviderTestHarness::new(provider);
        harness.run_all().await;
        assert!(!harness.results.is_empty());
        assert!(harness.results.iter().all(|r| r.passed));
    }
}
