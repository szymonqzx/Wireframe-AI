---
name: cost-aware-llm-pipeline-advanced
description: Advanced cost-aware LLM patterns — Provider system integration, multi-provider strategies, pricing calculations, token estimation, and Wireframe-AI specific implementations.
allowed-tools:
  - read
  - grep
  - glob
  - edit
  - write
  - exec
triggers:
  - model
---

# Cost-Aware LLM Pipeline — Advanced Patterns

Advanced integration patterns for Wireframe-AI's Provider system, multi-provider strategies, and production cost management.

## Wireframe-AI Provider System Integration

### Provider Trait Implementation

```rust
use async_trait::async_trait;

#[async_trait]
pub trait Provider: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String, ProviderError>;
    async fn complete_with_model(
        &self,
        messages: &[Message],
        model: Model,
    ) -> Result<CompletionResponse, ProviderError>;
    async fn stream(
        &self,
        prompt: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = String> + Send>>, ProviderError>;
}
```

### Cost-Aware Provider Wrapper

```rust
pub struct CostAwareProvider<P> {
    inner: P,
    tracker: Arc<Mutex<CostTracker>>,
}

impl<P: Provider> CostAwareProvider<P> {
    pub fn new(inner: P, budget_limit: f64) -> Self {
        Self {
            inner,
            tracker: Arc::new(Mutex::new(CostTracker::new(budget_limit))),
        }
    }

    pub fn total_cost(&self) -> f64 {
        let tracker = self.tracker.lock().unwrap();
        tracker.total_cost()
    }
}

#[async_trait]
impl<P: Provider> Provider for CostAwareProvider<P> {
    async fn complete(&self, prompt: &str) -> Result<String, ProviderError> {
        let response = self.inner.complete(prompt).await?;

        // Track cost
        let mut tracker = self.tracker.lock().unwrap();
        let record = CostRecord {
            model: "default".to_string(),
            input_tokens: estimate_tokens(prompt),
            output_tokens: estimate_tokens(&response),
            cost_usd: estimate_cost(prompt, &response),
        };
        *tracker = tracker.add(record);

        Ok(response)
    }

    async fn complete_with_model(
        &self,
        messages: &[Message],
        model: Model,
    ) -> Result<CompletionResponse, ProviderError> {
        let response = self.inner.complete_with_model(messages, model).await?;

        // Track cost
        let mut tracker = self.tracker.lock().unwrap();
        let record = CostRecord {
            model: model.to_string(),
            input_tokens: response.input_tokens,
            output_tokens: response.output_tokens,
            cost_usd: calculate_cost(&response),
        };
        *tracker = tracker.add(record);

        Ok(response)
    }

    async fn stream(
        &self,
        prompt: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = String> + Send>>, ProviderError> {
        self.inner.stream(prompt).await
    }
}
```

### Multi-Provider Strategy

```rust
pub struct MultiProviderStrategy {
    providers: Vec<Box<dyn Provider>>,
    cost_tracker: CostTracker,
}

impl MultiProviderStrategy {
    pub fn new(providers: Vec<Box<dyn Provider>>, budget_limit: f64) -> Self {
        Self {
            providers,
            cost_tracker: CostTracker::new(budget_limit),
        }
    }

    pub async fn complete_with_fallback(&mut self, prompt: &str) -> Result<String, ProviderError> {
        for provider in &self.providers {
            match provider.complete(prompt).await {
                Ok(response) => {
                    // Track cost
                    let record = CostRecord {
                        model: "multi".to_string(),
                        input_tokens: estimate_tokens(prompt),
                        output_tokens: estimate_tokens(&response),
                        cost_usd: estimate_cost(prompt, &response),
                    };
                    self.cost_tracker = self.cost_tracker.add(record);
                    return Ok(response);
                }
                Err(err) => {
                    eprintln!("Provider failed: {:?}, trying next", err);
                    continue;
                }
            }
        }
        Err(ProviderError::AllProvidersFailed)
    }
}
```

## Pricing Reference (2025-2026)

| Model | Input ($/1M tokens) | Output ($/1M tokens) | Relative Cost |
|-------|---------------------|----------------------|---------------|
| Haiku 4.5 | $0.80 | $4.00 | 1x |
| Sonnet 4.6 | $3.00 | $15.00 | ~4x |
| Opus 4.5 | $15.00 | $75.00 | ~19x |

## Cost Calculation

```rust
fn calculate_cost(response: &CompletionResponse) -> f64 {
    let input_cost = match response.model {
        Model::Haiku => (response.input_tokens as f64) * 0.80 / 1_000_000.0,
        Model::Sonnet => (response.input_tokens as f64) * 3.00 / 1_000_000.0,
        Model::Opus => (response.input_tokens as f64) * 15.00 / 1_000_000.0,
    };

    let output_cost = match response.model {
        Model::Haiku => (response.output_tokens as f64) * 4.00 / 1_000_000.0,
        Model::Sonnet => (response.output_tokens as f64) * 15.00 / 1_000_000.0,
        Model::Opus => (response.output_tokens as f64) * 75.00 / 1_000_000.0,
    };

    input_cost + output_cost
}
```

## Token Estimation

```rust
// Rough estimation: ~4 characters per token
fn estimate_tokens(text: &str) -> u32 {
    ((text.len() as f32) / 4.0).ceil() as u32
}

// More accurate estimation using tiktoken-rs
use tiktoken_rs::tiktoken::p50k_base;

fn estimate_tokens_accurate(text: &str) -> u32 {
    let bpe = p50k_base().unwrap();
    let tokens = bpe.encode_with_special_tokens(text);
    tokens.len() as u32
}
```

## Wireframe-AI Specific Considerations

### Module-Level Cost Tracking

```rust
// In each module that uses providers
pub struct Orchestrator {
    provider: Box<dyn Provider>,
    cost_tracker: CostTracker,
}

impl Orchestrator {
    pub async fn process_task(&mut self, task: &Task) -> Result<TaskResult, ProviderError> {
        let prompt = self.build_prompt(task);
        let model = select_model(prompt.len(), 1, None);

        let response = self.provider.complete_with_model(
            &build_cached_messages(&self.system_prompt, &prompt),
            model,
        ).await?;

        // Track cost
        let record = CostRecord {
            model: model.to_string(),
            input_tokens: response.input_tokens,
            output_tokens: response.output_tokens,
            cost_usd: calculate_cost(&response),
        };
        self.cost_tracker = self.cost_tracker.add(record);

        Ok(self.parse_result(response))
    }

    pub fn total_cost(&self) -> f64 {
        self.cost_tracker.total_cost()
    }
}
```

### Budget Enforcement

```rust
pub fn check_budget_before_operation(tracker: &CostTracker, estimated_cost: f64) -> Result<(), ProviderError> {
    if tracker.total_cost() + estimated_cost > tracker.budget_limit {
        Err(ProviderError::BudgetExceeded {
            spent: tracker.total_cost(),
            limit: tracker.budget_limit,
        })
    } else {
        Ok(())
    }
}
```

### Cost Reporting

```rust
pub fn generate_cost_report(tracker: &CostTracker) -> String {
    let total_cost = tracker.total_cost();
    let by_model: std::collections::HashMap<String, (u32, u32, f64)> = tracker
        .records
        .iter()
        .fold(Default::default(), |mut acc, record| {
            let entry = acc.entry(record.model.clone()).or_insert((0, 0, 0.0));
            entry.0 += record.input_tokens;
            entry.1 += record.output_tokens;
            entry.2 += record.cost_usd;
            acc
        });

    let mut report = format!("Total Cost: ${:.2}\n", total_cost);
    report.push_str("\nBy Model:\n");
    for (model, (input, output, cost)) in by_model {
        report.push_str(&format!(
            "  {}: {} input tokens, {} output tokens, ${:.2}\n",
            model, input, output, cost
        ));
    }

    report
}
```

## Production Considerations

### Cost Monitoring

- Track costs per module and per operation
- Set up alerts for budget thresholds
- Log model selection decisions for analysis
- Monitor token usage patterns

### Budget Management

- Set per-module budget limits
- Implement circuit breakers for over-budget scenarios
- Use cost estimates before API calls
- Provide cost visibility to operators

### Performance Optimization

- Cache expensive provider responses
- Use streaming for long responses to enable early termination
- Batch similar requests when possible
- Implement request queuing for rate-limited providers