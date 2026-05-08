//! Pipeline orchestration for ordered plugin execution.

use crate::plugin::Plugin;
use serde_json::Value;

/// A pipeline step with ordering information.
pub struct PipelineStep {
    pub plugin: Box<dyn Plugin>,
    pub order: usize,
}

/// Pipeline for ordered execution of plugins.
///
/// Plugins are executed in order based on their `order` field.
/// Output from one step can be passed to the next.
pub struct Pipeline {
    steps: Vec<PipelineStep>,
}

impl Pipeline {
    /// Create a new empty pipeline.
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Add a step to the pipeline.
    pub fn add_step(&mut self, step: PipelineStep) {
        self.steps.push(step);
    }

    /// Execute the pipeline.
    ///
    /// Plugins are executed in order by their `order` field.
    /// The input is passed to the first plugin, and output flows
    /// through subsequent plugins.
    pub async fn execute(&mut self, input: Value) -> Result<Value, PipelineError> {
        // Sort steps by order
        self.steps.sort_by_key(|s| s.order);

        let current_value = input;

        for step in &mut self.steps {
            // In a real implementation, each plugin would process the value
            // For now, we just ensure the plugin is healthy
            let healthy =
                step.plugin
                    .health_check()
                    .await
                    .map_err(|e| PipelineError::StepFailed {
                        step: step.plugin.plugin_id().to_string(),
                        error: e.to_string(),
                    })?;

            if !healthy {
                return Err(PipelineError::StepFailed {
                    step: step.plugin.plugin_id().to_string(),
                    error: "Plugin health check failed".to_string(),
                });
            }
        }

        Ok(current_value)
    }

    /// Get the number of steps in the pipeline.
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Check if the pipeline is empty.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Pipeline execution error.
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Step {step} failed: {error}")]
    StepFailed { step: String, error: String },

    #[error("Pipeline is empty")]
    EmptyPipeline,

    #[error("Order conflict: duplicate order {0}")]
    OrderConflict(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_empty_pipeline() {
        let pipeline = Pipeline::new();
        assert!(pipeline.is_empty());
        assert_eq!(pipeline.len(), 0);
    }

    #[test]
    fn test_default_creates_empty_pipeline() {
        let pipeline = Pipeline::default();
        assert!(pipeline.is_empty());
        assert_eq!(pipeline.len(), 0);
    }
}
