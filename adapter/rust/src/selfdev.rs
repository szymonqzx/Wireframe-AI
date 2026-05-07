//! Selfdev (self-development) support for the adapter.
//!
//! This module provides safety checks and compilation/restart functionality
//! for agents to modify their own source code at runtime.

use anyhow::Result;
use serde::Serialize;
use std::path::Path;
use tracing::info;

/// Results from a single safety check step.
#[derive(Debug, Serialize)]
pub struct CheckStep {
    pub name: &'static str,
    pub passed: bool,
    pub output: String,
    pub duration_ms: u64,
}

/// Aggregate result from all safety checks.
#[derive(Debug, Serialize)]
pub struct SafetyCheckResult {
    pub steps: Vec<CheckStep>,
    pub all_passed: bool,
}

impl SafetyCheckResult {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "all_passed": self.all_passed,
            "steps": self.steps.iter().map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "passed": s.passed,
                    "duration_ms": s.duration_ms,
                    "output_preview": s.output.chars().take(500).collect::<String>()
                })
            }).collect::<Vec<_>>()
        })
    }
}

/// Run safety checks before selfdev compilation or restart.
/// Checks: cargo check, cargo test (adapter only), cargo clippy.
pub async fn run_safety_checks(source_root: &Path) -> Result<SafetyCheckResult> {
    let mut steps = Vec::new();

    // 1. cargo check
    let start = tokio::time::Instant::now();
    let check_output = tokio::process::Command::new("cargo")
        .args(["check", "-p", "wireframe-adapter-rust"])
        .current_dir(source_root)
        .output()
        .await;
    let duration = start.elapsed().as_millis() as u64;

    match check_output {
        Ok(output) => {
            let passed = output.status.success();
            steps.push(CheckStep {
                name: "cargo check",
                passed,
                output: String::from_utf8_lossy(if passed {
                    &output.stdout
                } else {
                    &output.stderr
                })
                .to_string(),
                duration_ms: duration,
            });
        }
        Err(e) => {
            steps.push(CheckStep {
                name: "cargo check",
                passed: false,
                output: format!("Failed to execute: {}", e),
                duration_ms: duration,
            });
        }
    }

    // 2. cargo test (adapter only)
    let start = tokio::time::Instant::now();
    let test_output = tokio::process::Command::new("cargo")
        .args([
            "test",
            "-p",
            "wireframe-adapter-rust",
            "--bin",
            "wireframe-adapter",
        ])
        .current_dir(source_root)
        .output()
        .await;
    let duration = start.elapsed().as_millis() as u64;

    match test_output {
        Ok(output) => {
            let passed = output.status.success();
            steps.push(CheckStep {
                name: "cargo test",
                passed,
                output: String::from_utf8_lossy(if passed {
                    &output.stdout
                } else {
                    &output.stderr
                })
                .to_string(),
                duration_ms: duration,
            });
        }
        Err(e) => {
            steps.push(CheckStep {
                name: "cargo test",
                passed: false,
                output: format!("Failed to execute: {}", e),
                duration_ms: duration,
            });
        }
    }

    // 3. cargo clippy (treat warnings as errors for safety)
    let start = tokio::time::Instant::now();
    let clippy_output = tokio::process::Command::new("cargo")
        .args([
            "clippy",
            "-p",
            "wireframe-adapter-rust",
            "--",
            "-D",
            "warnings",
        ])
        .current_dir(source_root)
        .output()
        .await;
    let duration = start.elapsed().as_millis() as u64;

    match clippy_output {
        Ok(output) => {
            let passed = output.status.success();
            steps.push(CheckStep {
                name: "cargo clippy",
                passed,
                output: String::from_utf8_lossy(if passed {
                    &output.stdout
                } else {
                    &output.stderr
                })
                .to_string(),
                duration_ms: duration,
            });
        }
        Err(e) => {
            steps.push(CheckStep {
                name: "cargo clippy",
                passed: false,
                output: format!("Failed to execute: {}", e),
                duration_ms: duration,
            });
        }
    }

    let all_passed = steps.iter().all(|s| s.passed);

    info!(
        "Safety checks complete: {}/{} passed",
        steps.iter().filter(|s| s.passed).count(),
        steps.len()
    );

    Ok(SafetyCheckResult { steps, all_passed })
}

/// Compile the adapter module.
pub async fn compile_adapter(source_root: &Path) -> Result<serde_json::Value> {
    let compile_result = tokio::process::Command::new("cargo")
        .args(["build", "--release", "-p", "wireframe-adapter-rust"])
        .current_dir(source_root)
        .output()
        .await;

    match compile_result {
        Ok(output) => {
            let success = output.status.success();
            Ok(serde_json::json!({
                "success": success,
                "stdout": String::from_utf8_lossy(&output.stdout).to_string(),
                "stderr": String::from_utf8_lossy(&output.stderr).to_string(),
                "exit_code": output.status.code().unwrap_or(-1)
            }))
        }
        Err(e) => {
            Ok(serde_json::json!({
                "error": format!("Compilation failed: {}", e)
            }))
        }
    }
}
