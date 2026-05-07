use agentic_sdk::pipeline::{Pipeline, PipelineStep};
use agentic_sdk::plugin::{Plugin, PluginError};
use serde_json::Value;

struct StepPlugin {
    name: &'static str,
}

#[async_trait::async_trait]
impl Plugin for StepPlugin {
    fn plugin_id(&self) -> &'static str {
        self.name
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Step plugin"
    }

    async fn initialize(&mut self, _config: &Value) -> Result<(), PluginError> {
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[tokio::test]
async fn test_pipeline_execution() {
    let mut pipeline = Pipeline::new();

    let step1 = StepPlugin { name: "step1" };
    let step2 = StepPlugin { name: "step2" };

    pipeline.add_step(PipelineStep {
        plugin: Box::new(step1),
        order: 1,
    });

    pipeline.add_step(PipelineStep {
        plugin: Box::new(step2),
        order: 2,
    });

    let result = pipeline.execute(Value::Null).await;
    assert!(result.is_ok());
}
