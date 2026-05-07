---
name: cost-aware-llm-pipeline
description: Cost optimization patterns for Wireframe-AI LLM Provider system — model routing by task complexity, budget tracking, retry logic, and prompt caching.
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

# Cost-Aware LLM Pipeline for Wireframe-AI

Patterns for controlling LLM API costs while maintaining quality in the Wireframe-AI Provider system. Combines model routing, budget tracking, retry logic, and prompt caching into a composable pipeline.

## When to Activate

- Building applications that call LLM APIs through Wireframe-AI's Provider system
- Processing batches of items with varying complexity
- Need to stay within a budget for API spend
- Optimizing cost without sacrificing quality on complex tasks
- Implementing Provider trait implementations
- Designing multi-provider strategies in Wireframe-AI

## Core Concepts

### 1. Model Routing by Task Complexity

Automatically select cheaper models for simple tasks, reserving expensive models for complex ones.

```rust
use wireframe_ai::provider::{Provider, Model};

const MODEL_SONNET: Model = Model::Sonnet;
const MODEL_HAIKU: Model = Model::Haiku;

const SONNET_TEXT_THRESHOLD: usize = 10_000;  // chars
const SONNET_ITEM_THRESHOLD: usize = 30;     // items

fn select_model(
    text_length: usize,
    item_count: usize,
    force_model: Option<Model>,
) -> Model {
    match force_model {
        Some(model) => model,
        None if text_length >= SONNET_TEXT_THRESHOLD || item_count >= SONNET_ITEM_THRESHOLD => {
            MODEL_SONNET  // Complex task
        }
        None => MODEL_HAIKU  // Simple task (3-4x cheaper)
    }
}
```

### 2. Immutable Cost Tracking

Track cumulative spend with frozen dataclasses. Each API call returns a new tracker — never mutates state.

```rust
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecord {
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostTracker {
    pub budget_limit: f64,
    pub records: Vec<CostRecord>,
}

impl CostTracker {
    pub fn new(budget_limit: f64) -> Self {
        Self {
            budget_limit,
            records: Vec::new(),
        }
    }

    pub fn add(&self, record: CostRecord) -> Self {
        let mut new_tracker = self.clone();
        new_tracker.records.push(record);
        new_tracker
    }

    pub fn total_cost(&self) -> f64 {
        self.records.iter().map(|r| r.cost_usd).sum()
    }

    pub fn over_budget(&self) -> bool {
        self.total_cost() > self.budget_limit
    }
}
```

### 3. Narrow Retry Logic

Retry only on transient errors. Fail fast on authentication or bad request errors.

```rust
use wireframe_ai::provider::{ProviderError, Provider};
use tokio::time::{sleep, Duration};

const RETRYABLE_ERRORS: &[ProviderError] = &[
    ProviderError::RateLimit,
    ProviderError::Timeout,
    ProviderError::ConnectionError,
];

const MAX_RETRIES: u32 = 3;

async fn call_with_retry<F, Fut, T>(
    func: F,
) -> Result<T, ProviderError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, ProviderError>>,
{
    for attempt in 0..MAX_RETRIES {
        match func().await {
            Ok(result) => return Ok(result),
            Err(err) if RETRYABLE_ERRORS.contains(&err) => {
                if attempt == MAX_RETRIES - 1 {
                    return Err(err);
                }
                let delay = 2_u64.pow(attempt);
                sleep(Duration::from_secs(delay)).await;
            }
            Err(err) => return Err(err), // Non-retryable error
        }
    }
    unreachable!()
}
```

### 4. Prompt Caching

Cache long system prompts to avoid resending them on every request.

```rust
use wireframe_ai::provider::Message;

fn build_cached_messages(system_prompt: &str, user_input: &str) -> Vec<Message> {
    vec![
        Message {
            role: "user".to_string(),
            content: vec![
                MessageContent {
                    content_type: "text".to_string(),
                    text: system_prompt.to_string(),
                    cache_control: Some(CacheControl::Ephemeral),  // Cache this
                },
                MessageContent {
                    content_type: "text".to_string(),
                    text: user_input.to_string(),  // Variable part
                    cache_control: None,
                },
            ],
        },
    ]
}
```

## Composition

Combine all four techniques in a single pipeline function:

```rust
use wireframe_ai::provider::{Provider, ProviderError};

pub async fn process_with_cost_control(
    text: &str,
    provider: &dyn Provider,
    config: &Config,
    tracker: CostTracker,
) -> Result<(String, CostTracker), ProviderError> {
    // 1. Route model
    let model = select_model(text.len(), estimate_items(text), config.force_model);

    // 2. Check budget
    if tracker.over_budget() {
        return Err(ProviderError::BudgetExceeded {
            spent: tracker.total_cost(),
            limit: tracker.budget_limit,
        });
    }

    // 3. Call with retry + caching
    let response = call_with_retry(|| {
        provider.complete_with_model(
            &build_cached_messages(&config.system_prompt, text),
            model,
        )
    }).await?;

    // 4. Track cost (immutable)
    let record = CostRecord {
        model: model.to_string(),
        input_tokens: response.input_tokens,
        output_tokens: response.output_tokens,
        cost_usd: calculate_cost(&response),
    };
    let tracker = tracker.add(record);

    Ok((response.text, tracker))
}
```

## Best Practices

- **Start with the cheapest model** and only route to expensive models when complexity thresholds are met
- **Set explicit budget limits** before processing batches — fail early rather than overspend
- **Log model selection decisions** so you can tune thresholds based on real data
- **Use prompt caching** for system prompts over 1024 tokens — saves both cost and latency
- **Never retry on authentication or validation errors** — only transient failures (network, rate limit, server error)
- **Track costs per module** in Wireframe-AI to identify expensive operations
- **Use the Provider trait** for consistent cost tracking across implementations

## Anti-Patterns to Avoid

- Using the most expensive model for all requests regardless of complexity
- Retrying on all errors (wastes budget on permanent failures)
- Mutating cost tracking state (makes debugging and auditing difficult)
- Hardcoding model names throughout the codebase (use constants or config)
- Ignoring prompt caching for repetitive system prompts
- Not setting budget limits (risk of unexpected charges)
- Skipping cost tracking in development (leads to surprises in production)

## Integration with Wireframe-AI Skills

- Use `/agentic-engineering` for cost-aware development workflow
- Use `/rust-patterns` for idiomatic Rust implementation
- Use `/rust-testing` for testing cost tracking logic
- Use `/architecture-decision-records` to record cost-related decisions
- Use `/wireframe-workflow` for schema validation if storing cost data

## When to Use

- Any Wireframe-AI module calling LLM providers through the Provider system
- Batch processing pipelines where cost adds up quickly
- Multi-provider architectures that need intelligent routing
- Production systems that need budget guardrails
- Development environments where cost awareness is important

**Remember**: Cost control is a first-class concern in AI systems. Design for it from the start, not as an afterthought.

**See Also:** `cost-aware-llm-pipeline-advanced.md` for advanced Provider system integration, multi-provider strategies, and Wireframe-AI specific patterns.