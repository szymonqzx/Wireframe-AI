//! Unix resource limiter — enforces CPU and memory limits using rlimit.

use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::sandbox::{ResourceError, ResourceLimiter};
use async_trait::async_trait;
use serde_json::Value;
use std::time::{Duration, Instant};

#[cfg(unix)]
use libc::{rlimit, setrlimit, RLIMIT_AS, RLIMIT_CPU};

/// Unix resource limiter using rlimit.
pub struct UnixResourceLimiter {
    cpu_limit_secs: u64,
    memory_limit_mb: u64,
    timeout_secs: u64,
}

impl UnixResourceLimiter {
    pub fn new() -> Self {
        Self {
            cpu_limit_secs: 300,   // 5 minutes
            memory_limit_mb: 1024, // 1GB
            timeout_secs: 30,
        }
    }

    pub fn with_cpu_limit(mut self, cpu_limit_secs: u64) -> Self {
        self.cpu_limit_secs = cpu_limit_secs;
        self
    }

    pub fn with_memory_limit(mut self, memory_limit_mb: u64) -> Self {
        self.memory_limit_mb = memory_limit_mb;
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Apply rlimit settings to the current process (Unix only).
    #[cfg(unix)]
    pub fn apply_rlimits(&self) -> Result<(), ResourceError> {
        // Limit CPU time
        let cpu_limit = rlimit {
            rlim_cur: self.cpu_limit_secs,
            rlim_max: self.cpu_limit_secs,
        };
        unsafe {
            if setrlimit(RLIMIT_CPU, &cpu_limit) != 0 {
                return Err(ResourceError::MonitoringFailed(
                    "Failed to set CPU limit".to_string(),
                ));
            }
        }

        // Limit address space (memory)
        let mem_limit_bytes = self.memory_limit_mb * 1024 * 1024;
        let mem_limit = rlimit {
            rlim_cur: mem_limit_bytes,
            rlim_max: mem_limit_bytes,
        };
        unsafe {
            if setrlimit(RLIMIT_AS, &mem_limit) != 0 {
                return Err(ResourceError::MonitoringFailed(
                    "Failed to set memory limit".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// No-op on non-Unix platforms.
    #[cfg(not(unix))]
    pub fn apply_rlimits(&self) -> Result<(), ResourceError> {
        Ok(())
    }
}

impl Default for UnixResourceLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for UnixResourceLimiter {
    fn plugin_id(&self) -> &'static str {
        "limits-unix"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Unix resource limiter for sandbox"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), agentic_sdk::plugin::PluginError> {
        if let Some(cpu) = config.get("cpu_limit_secs").and_then(|v| v.as_u64()) {
            self.cpu_limit_secs = cpu;
        }

        if let Some(mem) = config.get("memory_limit_mb").and_then(|v| v.as_u64()) {
            self.memory_limit_mb = mem;
        }

        if let Some(timeout) = config.get("timeout_secs").and_then(|v| v.as_u64()) {
            self.timeout_secs = timeout;
        }

        Ok(())
    }

    async fn health_check(&self) -> Result<bool, agentic_sdk::plugin::PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), agentic_sdk::plugin::PluginError> {
        Ok(())
    }
}

#[async_trait]
impl ResourceLimiter for UnixResourceLimiter {
    async fn check_cpu_limit(&self, current_usage: Duration) -> Result<bool, ResourceError> {
        let limit = Duration::from_secs(self.cpu_limit_secs);
        Ok(current_usage < limit)
    }

    async fn check_memory_limit(&self, current_usage: usize) -> Result<bool, ResourceError> {
        let limit_bytes = (self.memory_limit_mb * 1024 * 1024) as usize;
        Ok(current_usage < limit_bytes)
    }

    async fn enforce_timeout(
        &self,
        started_at: Instant,
        timeout: Duration,
    ) -> Result<(), ResourceError> {
        let elapsed = started_at.elapsed();
        if elapsed > timeout {
            return Err(ResourceError::TimeoutExceeded);
        }
        Ok(())
    }
}
