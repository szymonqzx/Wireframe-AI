//! Validator Module — Example Wireframe AI module.
//!
//! Subscribes to `validate.request` and validates JSON payloads against
//! expected schemas. Returns validation results with detailed errors.
//!
//! ## Request format
//!
//! ```json
//! {
//!   "schema_type": "task_submitted",
//!   "payload": { ... }
//! }
//! ```

use agentic_sdk::Module;

struct ValidatorModule;

#[agentic_sdk::module(
    subscribes = ["validate.request"],
    publishes  = ["validate.response"],
    queue_group = "validator"
)]
impl Module for ValidatorModule {
    async fn handle(
        &mut self,
        env: Envelope<serde_json::Value>,
    ) -> Vec<Envelope<serde_json::Value>> {
        let request = &env.payload;
        let schema_type = request
            .get("schema_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let payload = request.get("payload").cloned().unwrap_or_default();

        tracing::info!(schema_type, "validating payload");

        let result = match schema_type {
            "task_submitted" => validate_task_submitted(&payload),
            "task_complete" => validate_task_complete(&payload),
            "agent_job" => validate_agent_job(&payload),
            "agent_result" => validate_agent_result(&payload),
            _ => serde_json::json!({
                "valid": false,
                "errors": [format!("Unknown schema type: {}", schema_type)]
            }),
        };

        let response = serde_json::json!({
            "schema_type": schema_type,
            "validation": result,
            "validated_at": chrono::Utc::now().timestamp(),
        });

        vec![env.reply("validate.response", response)]
    }
}

fn validate_task_submitted(payload: &serde_json::Value) -> serde_json::Value {
    let mut errors = Vec::new();
    if payload.get("session_id").is_none() {
        errors.push("missing session_id");
    }
    if payload.get("user_input").is_none() {
        errors.push("missing user_input");
    }
    if payload.get("submitted_at").is_none() {
        errors.push("missing submitted_at");
    }

    serde_json::json!({
        "valid": errors.is_empty(),
        "errors": errors
    })
}

fn validate_task_complete(payload: &serde_json::Value) -> serde_json::Value {
    let mut errors = Vec::new();
    if payload.get("session_id").is_none() {
        errors.push("missing session_id");
    }
    if payload.get("correlation_id").is_none() {
        errors.push("missing correlation_id");
    }
    if payload.get("result").is_none() {
        errors.push("missing result");
    }

    serde_json::json!({
        "valid": errors.is_empty(),
        "errors": errors
    })
}

fn validate_agent_job(payload: &serde_json::Value) -> serde_json::Value {
    let mut errors = Vec::new();
    if payload.get("job_id").is_none() {
        errors.push("missing job_id");
    }
    if payload.get("correlation_parent").is_none() {
        errors.push("missing correlation_parent");
    }
    if payload.get("task").is_none() {
        errors.push("missing task");
    }

    serde_json::json!({
        "valid": errors.is_empty(),
        "errors": errors
    })
}

fn validate_agent_result(payload: &serde_json::Value) -> serde_json::Value {
    let mut errors = Vec::new();
    if payload.get("job_id").is_none() {
        errors.push("missing job_id");
    }
    if payload.get("correlation_parent").is_none() {
        errors.push("missing correlation_parent");
    }

    serde_json::json!({
        "valid": errors.is_empty(),
        "errors": errors
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    ValidatorModule.run("nats://localhost:4222").await
}
