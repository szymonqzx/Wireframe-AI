//! Module compatibility checking for runtime module switching.
//!
//! Verifies that a new module is compatible with the current system
//! before allowing a runtime switch.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Module interface definition - what topics a module subscribes to and publishes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInterface {
    pub module_id: String,
    pub subscribes: Vec<String>,
    pub publishes: Vec<String>,
    pub version: String,
}

/// Compatibility check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityResult {
    pub is_compatible: bool,
    pub issues: Vec<CompatibilityIssue>,
    pub warnings: Vec<String>,
}

/// Specific compatibility issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityIssue {
    pub severity: IssueSeverity,
    pub description: String,
    pub field: String,
}

/// Severity of compatibility issue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueSeverity {
    Error,
    Warning,
    Info,
}

/// Compatibility checker.
pub struct CompatibilityChecker {
    /// Required topics that must be subscribed to for specific module types.
    required_subscriptions: HashSet<String>,
    /// Required topics that must be published for specific module types.
    required_publications: HashSet<String>,
}

impl CompatibilityChecker {
    /// Create a new compatibility checker.
    pub fn new() -> Self {
        let mut required_subscriptions = HashSet::new();
        let mut required_publications = HashSet::new();

        // Adapter-specific requirements
        required_subscriptions.insert("agent.job".to_string());
        required_publications.insert("agent.result".to_string());

        Self {
            required_subscriptions,
            required_publications,
        }
    }

    /// Check if a new module is compatible with the current module.
    pub fn check_compatibility(
        &self,
        current_module: &ModuleInterface,
        new_module: &ModuleInterface,
    ) -> CompatibilityResult {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check if new module subscribes to required topics
        for required in &self.required_subscriptions {
            if !new_module.subscribes.contains(required) {
                issues.push(CompatibilityIssue {
                    severity: IssueSeverity::Error,
                    description: format!("Missing required subscription: {}", required),
                    field: "subscribes".to_string(),
                });
            }
        }

        // Check if new module publishes required topics
        for required in &self.required_publications {
            if !new_module.publishes.contains(required) {
                issues.push(CompatibilityIssue {
                    severity: IssueSeverity::Error,
                    description: format!("Missing required publication: {}", required),
                    field: "publishes".to_string(),
                });
            }
        }

        // Check if subscription set is compatible (new module should handle all current subscriptions)
        for current_sub in &current_module.subscribes {
            if !new_module.subscribes.contains(current_sub) {
                warnings.push(format!(
                    "New module does not subscribe to topic that current module handles: {}",
                    current_sub
                ));
            }
        }

        // Check if publication set is compatible (new module should publish all current publications)
        for current_pub in &current_module.publishes {
            if !new_module.publishes.contains(current_pub) {
                warnings.push(format!(
                    "New module does not publish topic that current module publishes: {}",
                    current_pub
                ));
            }
        }

        // Check for additional subscriptions (may be okay, but worth noting)
        for new_sub in &new_module.subscribes {
            if !current_module.subscribes.contains(new_sub) {
                warnings.push(format!(
                    "New module subscribes to additional topic: {}",
                    new_sub
                ));
            }
        }

        // Check for additional publications (may be okay, but worth noting)
        for new_pub in &new_module.publishes {
            if !current_module.publishes.contains(new_pub) {
                warnings.push(format!(
                    "New module publishes to additional topic: {}",
                    new_pub
                ));
            }
        }

        let is_compatible = issues.iter().all(|i| i.severity != IssueSeverity::Error);

        CompatibilityResult {
            is_compatible,
            issues,
            warnings,
        }
    }

    /// Check if a module can safely replace another module of the same type.
    pub fn check_module_type_compatibility(
        &self,
        module_type: &str,
        new_module: &ModuleInterface,
    ) -> CompatibilityResult {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        match module_type {
            "adapter" => {
                // Adapter must subscribe to agent.job
                if !new_module.subscribes.contains(&"agent.job".to_string()) {
                    issues.push(CompatibilityIssue {
                        severity: IssueSeverity::Error,
                        description: "Adapter must subscribe to agent.job".to_string(),
                        field: "subscribes".to_string(),
                    });
                }

                // Adapter must publish agent.result
                if !new_module.publishes.contains(&"agent.result".to_string()) {
                    issues.push(CompatibilityIssue {
                        severity: IssueSeverity::Error,
                        description: "Adapter must publish agent.result".to_string(),
                        field: "publishes".to_string(),
                    });
                }
            }
            "context" => {
                // Context must subscribe to task.submitted
                if !new_module
                    .subscribes
                    .contains(&"task.submitted".to_string())
                {
                    issues.push(CompatibilityIssue {
                        severity: IssueSeverity::Error,
                        description: "Context must subscribe to task.submitted".to_string(),
                        field: "subscribes".to_string(),
                    });
                }

                // Context must publish task.enriched
                if !new_module.publishes.contains(&"task.enriched".to_string()) {
                    issues.push(CompatibilityIssue {
                        severity: IssueSeverity::Error,
                        description: "Context must publish task.enriched".to_string(),
                        field: "publishes".to_string(),
                    });
                }
            }
            "orchestrator" => {
                // Orchestrator must subscribe to task.enriched
                if !new_module.subscribes.contains(&"task.enriched".to_string()) {
                    issues.push(CompatibilityIssue {
                        severity: IssueSeverity::Error,
                        description: "Orchestrator must subscribe to task.enriched".to_string(),
                        field: "subscribes".to_string(),
                    });
                }

                // Orchestrator must publish agent.job
                if !new_module.publishes.contains(&"agent.job".to_string()) {
                    issues.push(CompatibilityIssue {
                        severity: IssueSeverity::Error,
                        description: "Orchestrator must publish agent.job".to_string(),
                        field: "publishes".to_string(),
                    });
                }

                // Orchestrator must publish task.complete
                if !new_module.publishes.contains(&"task.complete".to_string()) {
                    issues.push(CompatibilityIssue {
                        severity: IssueSeverity::Error,
                        description: "Orchestrator must publish task.complete".to_string(),
                        field: "publishes".to_string(),
                    });
                }
            }
            _ => {
                warnings.push(format!(
                    "Unknown module type: {}, skipping type-specific checks",
                    module_type
                ));
            }
        }

        let is_compatible = issues.iter().all(|i| i.severity != IssueSeverity::Error);

        CompatibilityResult {
            is_compatible,
            issues,
            warnings,
        }
    }
}

impl Default for CompatibilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compatible_modules() {
        let checker = CompatibilityChecker::new();

        let current = ModuleInterface {
            module_id: "wireframe-adapter-rust".to_string(),
            subscribes: vec!["agent.job".to_string()],
            publishes: vec!["agent.result".to_string()],
            version: "0.1.0".to_string(),
        };

        let new_module = ModuleInterface {
            module_id: "community-adapter-x".to_string(),
            subscribes: vec!["agent.job".to_string()],
            publishes: vec!["agent.result".to_string()],
            version: "1.0.0".to_string(),
        };

        let result = checker.check_compatibility(&current, &new_module);
        assert!(result.is_compatible);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_incompatible_modules() {
        let checker = CompatibilityChecker::new();

        let current = ModuleInterface {
            module_id: "wireframe-adapter-rust".to_string(),
            subscribes: vec!["agent.job".to_string()],
            publishes: vec!["agent.result".to_string()],
            version: "0.1.0".to_string(),
        };

        let new_module = ModuleInterface {
            module_id: "broken-adapter".to_string(),
            subscribes: vec![], // Missing required subscription
            publishes: vec!["agent.result".to_string()],
            version: "1.0.0".to_string(),
        };

        let result = checker.check_compatibility(&current, &new_module);
        assert!(!result.is_compatible);
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_adapter_type_check() {
        let checker = CompatibilityChecker::new();

        let adapter = ModuleInterface {
            module_id: "wireframe-adapter-rust".to_string(),
            subscribes: vec!["agent.job".to_string()],
            publishes: vec!["agent.result".to_string()],
            version: "0.1.0".to_string(),
        };

        let result = checker.check_module_type_compatibility("adapter", &adapter);
        assert!(result.is_compatible);
    }
}
