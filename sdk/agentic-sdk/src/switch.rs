//! Module switch coordination logic.
//!
//! Handles the process of switching from one module to another at runtime,
//! including compatibility checks, process management, and NATS coordination.

use crate::compatibility::{CompatibilityChecker, CompatibilityResult};
use crate::registry::ModuleRegistry;
use anyhow::Result;
use async_nats::Client;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{error, info, warn};

/// Module switch request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSwitchRequest {
    pub old_module: String,
    pub new_module: String,
    pub reason: String,
    pub requested_by: String,
    pub force: bool, // Skip compatibility checks if true
    pub timestamp: i64,
}

/// Module switch acknowledgment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSwitchAck {
    pub request_id: String,
    pub status: SwitchStatus,
    pub message: String,
    pub old_pid: Option<u32>,
    pub new_pid: Option<u32>,
    pub compatibility_result: Option<CompatibilityResult>,
    pub timestamp: i64,
}

/// Switch status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SwitchStatus {
    Pending,
    CompatibilityCheckFailed,
    StoppingOldModule,
    StartingNewModule,
    Completed,
    Failed,
    RolledBack,
}

/// Module switch coordinator.
pub struct ModuleSwitchCoordinator {
    registry: ModuleRegistry,
    compatibility_checker: CompatibilityChecker,
    nats_client: Option<Client>,
}

impl ModuleSwitchCoordinator {
    /// Create a new module switch coordinator.
    pub fn new(registry: ModuleRegistry) -> Self {
        Self {
            registry,
            compatibility_checker: CompatibilityChecker::new(),
            nats_client: None,
        }
    }

    /// Create a new module switch coordinator with NATS client.
    pub fn with_nats_client(registry: ModuleRegistry, nats_client: Client) -> Self {
        Self {
            registry,
            compatibility_checker: CompatibilityChecker::new(),
            nats_client: Some(nats_client),
        }
    }

    /// Execute a module switch.
    pub async fn execute_switch(&self, request: ModuleSwitchRequest) -> Result<ModuleSwitchAck> {
        let request_id = format!("switch-{}-{}", request.old_module, request.timestamp);

        info!(
            "Executing module switch: {} -> {}",
            request.old_module, request.new_module
        );

        // Get module metadata
        let old_metadata = self
            .registry
            .get_module(&request.old_module)
            .ok_or_else(|| anyhow::anyhow!("Old module not found: {}", request.old_module))?;

        let new_metadata = self
            .registry
            .get_module(&request.new_module)
            .ok_or_else(|| anyhow::anyhow!("New module not found: {}", request.new_module))?;

        // Check compatibility (unless forced)
        let compatibility_result = if !request.force {
            let result = self
                .compatibility_checker
                .check_compatibility(&old_metadata.interface, &new_metadata.interface);

            if !result.is_compatible {
                error!("Compatibility check failed for module switch");
                return Ok(ModuleSwitchAck {
                    request_id: request_id.clone(),
                    status: SwitchStatus::CompatibilityCheckFailed,
                    message: "Compatibility check failed".to_string(),
                    old_pid: None,
                    new_pid: None,
                    compatibility_result: Some(result),
                    timestamp: current_timestamp(),
                });
            }

            Some(result)
        } else {
            warn!("Skipping compatibility check (force mode)");
            None
        };

        // Stop old module
        info!("Stopping old module: {}", request.old_module);
        let old_pid = self.stop_module(&request.old_module).await?;

        // Create NATS subscription BEFORE starting module to avoid race condition
        let subscriber = if let Some(client) = &self.nats_client {
            match client.subscribe("sys.module.online").await {
                Ok(sub) => Some(sub),
                Err(e) => {
                    warn!(
                        "Failed to subscribe to sys.module.online: {}, will use fallback timeout",
                        e
                    );
                    None
                }
            }
        } else {
            None
        };

        // Start new module
        info!("Starting new module: {}", request.new_module);
        let new_pid = self
            .start_module(&request.new_module, &new_metadata.binary_path)
            .await?;

        // Wait for new module to come online using NATS-based detection
        info!("Waiting for new module to come online via NATS");
        if let Some(sub) = subscriber {
            if let Err(e) = self
                .wait_for_module_online_with_subscription(&request.new_module, sub)
                .await
            {
                warn!(
                    "Failed to detect module online via NATS: {}, using fallback timeout",
                    e
                );
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        } else {
            // No NATS client or subscription failed, use fallback
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }

        Ok(ModuleSwitchAck {
            request_id,
            status: SwitchStatus::Completed,
            message: format!(
                "Successfully switched from {} to {}",
                request.old_module, request.new_module
            ),
            old_pid: Some(old_pid),
            new_pid: Some(new_pid),
            compatibility_result,
            timestamp: current_timestamp(),
        })
    }

    /// Stop a running module.
    async fn stop_module(&self, module_id: &str) -> Result<u32> {
        let _metadata = self
            .registry
            .get_module(module_id)
            .ok_or_else(|| anyhow::anyhow!("Module not found: {}", module_id))?;

        // Get process ID from module name
        let process_name = extract_process_name(module_id);
        let pid = find_process_by_name(&process_name)?;

        info!("Stopping module {} (PID: {})", module_id, pid);

        // Stop the process
        #[cfg(unix)]
        {
            use std::process::Command;
            Command::new("kill")
                .arg(pid.to_string())
                .status()
                .map_err(|e| anyhow::anyhow!("Failed to stop process: {}", e))?;
        }

        #[cfg(windows)]
        {
            use std::process::Command;
            Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .status()
                .map_err(|e| anyhow::anyhow!("Failed to stop process: {}", e))?;
        }

        // Wait for process to stop
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        Ok(pid)
    }

    /// Start a module.
    async fn start_module(&self, module_id: &str, binary_path: &PathBuf) -> Result<u32> {
        info!(
            "Starting module {} from {}",
            module_id,
            binary_path.display()
        );

        #[cfg(unix)]
        {
            use std::process::Command;
            let child = Command::new(binary_path)
                .spawn()
                .map_err(|e| anyhow::anyhow!("Failed to start module: {}", e))?;
            Ok(child.id())
        }

        #[cfg(windows)]
        {
            use std::process::Command;
            let child = Command::new(binary_path)
                .spawn()
                .map_err(|e| anyhow::anyhow!("Failed to start module: {}", e))?;
            Ok(child.id())
        }
    }

    /// Wait for a module to come online via NATS sys.module.online events.
    async fn wait_for_module_online(
        &self,
        module_id: &str,
        _metadata: &crate::registry::ModuleMetadata,
    ) -> Result<()> {
        let client = self
            .nats_client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("NATS client not configured for online detection"))?;

        let subscriber = client
            .subscribe("sys.module.online")
            .await
            .map_err(|e| anyhow::anyhow!("Failed to subscribe to sys.module.online: {}", e))?;

        self.wait_for_module_online_with_subscription(module_id, subscriber)
            .await
    }

    /// Wait for a module to come online using a pre-existing NATS subscription.
    async fn wait_for_module_online_with_subscription(
        &self,
        module_id: &str,
        mut subscriber: async_nats::Subscriber,
    ) -> Result<()> {
        let timeout = tokio::time::Duration::from_secs(10);
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            match tokio::time::timeout(tokio::time::Duration::from_secs(1), subscriber.next()).await
            {
                Ok(Some(msg)) => {
                    if let Ok(envelope) = serde_json::from_slice::<serde_json::Value>(&msg.payload)
                    {
                        // The sys.module.online message is wrapped in an Envelope,
                        // so module_id lives under the `payload` field. Fall back to
                        // a top-level lookup for robustness against unwrapped messages.
                        let online_module_id = envelope
                            .get("payload")
                            .and_then(|p| p.get("module_id"))
                            .or_else(|| envelope.get("module_id"))
                            .and_then(|v| v.as_str());

                        if let Some(online_module_id) = online_module_id {
                            if online_module_id == module_id {
                                info!(module_id, "detected module online via NATS");
                                return Ok(());
                            }
                        }
                    }
                }
                Ok(None) => {
                    warn!("NATS subscription closed unexpectedly");
                    break;
                }
                Err(_) => {
                    // Timeout on individual message, continue waiting
                    continue;
                }
            }
        }

        Err(anyhow::anyhow!(
            "Timeout waiting for module {} to come online",
            module_id
        ))
    }

    /// Rollback a failed switch.
    pub async fn rollback_switch(&self, request: &ModuleSwitchRequest) -> Result<()> {
        warn!(
            "Rolling back module switch: {} -> {}",
            request.old_module, request.new_module
        );

        // Stop new module
        if let Ok(pid) = self.stop_module(&request.new_module).await {
            info!("Stopped new module (PID: {})", pid);
        }

        // Start old module
        if let Some(old_metadata) = self.registry.get_module(&request.old_module) {
            if let Ok(pid) = self
                .start_module(&request.old_module, &old_metadata.binary_path)
                .await
            {
                info!("Restarted old module (PID: {})", pid);
            }
        }

        Ok(())
    }
}

/// Extract process name from module ID.
fn extract_process_name(module_id: &str) -> String {
    module_id
        .replace("wireframe-", "")
        .replace("wireframe-ai-", "")
        .replace("-", "_")
}

/// Find process ID by name.
fn find_process_by_name(name: &str) -> Result<u32> {
    #[cfg(unix)]
    {
        use std::process::Command;
        let output = Command::new("pgrep")
            .arg(name)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to find process: {}", e))?;

        let pid_str = String::from_utf8_lossy(&output.stdout);
        let pid: u32 = pid_str
            .trim()
            .parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse PID: {}", e))?;

        Ok(pid)
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        let output = Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {}.exe", name)])
            .args(["/FO", "CSV"])
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to find process: {}", e))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        // Parse CSV output to extract PID
        // This is simplified - in production, use proper CSV parsing
        let lines: Vec<&str> = output_str.lines().collect();
        if lines.len() > 1 {
            let second_line = lines[1];
            let parts: Vec<&str> = second_line.split(',').collect();
            if parts.len() > 1 {
                let pid_str = parts[1].trim_matches('"');
                let pid: u32 = pid_str
                    .parse()
                    .map_err(|e| anyhow::anyhow!("Failed to parse PID: {}", e))?;
                return Ok(pid);
            }
        }

        Err(anyhow::anyhow!("Process not found"))
    }
}

/// Get current timestamp.
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_process_name() {
        assert_eq!(
            extract_process_name("wireframe-adapter-rust"),
            "adapter_rust"
        );
        assert_eq!(extract_process_name("wireframe-ai-context"), "ai_context");
        assert_eq!(extract_process_name("custom-module"), "custom_module");
    }
}
