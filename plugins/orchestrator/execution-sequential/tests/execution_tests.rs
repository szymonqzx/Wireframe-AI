//! Tests for the sequential execution plugin.

use agentic_sdk::Plugin;
use execution_sequential::SequentialExecution;

#[tokio::test]
async fn test_execution_sequential_plugin_id() {
    let execution = SequentialExecution::new();
    assert_eq!(execution.plugin_id(), "execution-sequential");
}
