//! Module registry for tracking installed modules and their metadata.
//!
//! Maintains a registry of available modules with their binary paths,
//! versions, and interface definitions for runtime module switching.

use crate::compatibility::ModuleInterface;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Module metadata stored in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMetadata {
    pub module_id: String,
    pub module_type: String, // "adapter", "context", "orchestrator", etc.
    pub binary_path: PathBuf,
    pub source_path: Option<PathBuf>,
    pub version: String,
    pub interface: ModuleInterface,
    pub enabled: bool,
    pub installed_at: i64,
}

/// Module registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleRegistry {
    modules: HashMap<String, ModuleMetadata>,
}

impl ModuleRegistry {
    /// Create a new empty module registry.
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    /// Register a module in the registry.
    pub fn register_module(&mut self, metadata: ModuleMetadata) -> Result<(), String> {
        let module_id = metadata.module_id.clone();
        self.modules.insert(module_id, metadata);
        Ok(())
    }

    /// Unregister a module from the registry.
    pub fn unregister_module(&mut self, module_id: &str) -> Option<ModuleMetadata> {
        self.modules.remove(module_id)
    }

    /// Get module metadata by ID.
    pub fn get_module(&self, module_id: &str) -> Option<&ModuleMetadata> {
        self.modules.get(module_id)
    }

    /// Get module metadata by ID (mutable).
    pub fn get_module_mut(&mut self, module_id: &str) -> Option<&mut ModuleMetadata> {
        self.modules.get_mut(module_id)
    }

    /// List all registered modules.
    pub fn list_modules(&self) -> Vec<&ModuleMetadata> {
        self.modules.values().collect()
    }

    /// List modules by type.
    pub fn list_modules_by_type(&self, module_type: &str) -> Vec<&ModuleMetadata> {
        self.modules
            .values()
            .filter(|m| m.module_type == module_type)
            .collect()
    }

    /// List enabled modules.
    pub fn list_enabled_modules(&self) -> Vec<&ModuleMetadata> {
        self.modules.values().filter(|m| m.enabled).collect()
    }

    /// Enable a module.
    pub fn enable_module(&mut self, module_id: &str) -> Result<(), String> {
        if let Some(metadata) = self.modules.get_mut(module_id) {
            metadata.enabled = true;
            Ok(())
        } else {
            Err(format!("Module not found: {}", module_id))
        }
    }

    /// Disable a module.
    pub fn disable_module(&mut self, module_id: &str) -> Result<(), String> {
        if let Some(metadata) = self.modules.get_mut(module_id) {
            metadata.enabled = false;
            Ok(())
        } else {
            Err(format!("Module not found: {}", module_id))
        }
    }

    /// Update module metadata.
    pub fn update_module(&mut self, metadata: ModuleMetadata) -> Result<(), String> {
        let module_id = metadata.module_id.clone();
        match self.modules.entry(module_id) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                e.insert(metadata);
                Ok(())
            }
            std::collections::hash_map::Entry::Vacant(_) => {
                Err(format!("Module not found: {}", metadata.module_id))
            }
        }
    }

    /// Check if a module is registered.
    pub fn is_registered(&self, module_id: &str) -> bool {
        self.modules.contains_key(module_id)
    }

    /// Check if a module is enabled.
    pub fn is_enabled(&self, module_id: &str) -> bool {
        self.modules
            .get(module_id)
            .map(|m| m.enabled)
            .unwrap_or(false)
    }

    /// Get the count of registered modules.
    pub fn count(&self) -> usize {
        self.modules.len()
    }

    /// Clear the registry.
    pub fn clear(&mut self) {
        self.modules.clear();
    }

    /// Load registry from a file.
    pub fn load_from_file(path: &PathBuf) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read registry file: {}", e))?;

        let registry: ModuleRegistry = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse registry file: {}", e))?;

        Ok(registry)
    }

    /// Save registry to a file.
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), String> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize registry: {}", e))?;

        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write registry file: {}", e))?;

        Ok(())
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_register_and_get_module() {
        let mut registry = ModuleRegistry::new();

        let metadata = ModuleMetadata {
            module_id: "test-module".to_string(),
            module_type: "adapter".to_string(),
            binary_path: PathBuf::from("/path/to/binary"),
            source_path: Some(PathBuf::from("/path/to/source")),
            version: "1.0.0".to_string(),
            interface: ModuleInterface {
                module_id: "test-module".to_string(),
                subscribes: vec!["agent.job".to_string()],
                publishes: vec!["agent.result".to_string()],
                version: "1.0.0".to_string(),
            },
            enabled: true,
            installed_at: 1234567890,
        };

        registry.register_module(metadata.clone()).unwrap();
        assert!(registry.is_registered("test-module"));
        assert_eq!(
            registry.get_module("test-module").unwrap().module_id,
            "test-module"
        );
    }

    #[test]
    fn test_list_modules_by_type() {
        let mut registry = ModuleRegistry::new();

        let adapter_metadata = ModuleMetadata {
            module_id: "adapter-1".to_string(),
            module_type: "adapter".to_string(),
            binary_path: PathBuf::from("/path/to/adapter-1"),
            source_path: None,
            version: "1.0.0".to_string(),
            interface: ModuleInterface {
                module_id: "adapter-1".to_string(),
                subscribes: vec!["agent.job".to_string()],
                publishes: vec!["agent.result".to_string()],
                version: "1.0.0".to_string(),
            },
            enabled: true,
            installed_at: 1234567890,
        };

        let context_metadata = ModuleMetadata {
            module_id: "context-1".to_string(),
            module_type: "context".to_string(),
            binary_path: PathBuf::from("/path/to/context-1"),
            source_path: None,
            version: "1.0.0".to_string(),
            interface: ModuleInterface {
                module_id: "context-1".to_string(),
                subscribes: vec!["task.submitted".to_string()],
                publishes: vec!["task.enriched".to_string()],
                version: "1.0.0".to_string(),
            },
            enabled: true,
            installed_at: 1234567890,
        };

        registry.register_module(adapter_metadata).unwrap();
        registry.register_module(context_metadata).unwrap();

        let adapters = registry.list_modules_by_type("adapter");
        assert_eq!(adapters.len(), 1);
        assert_eq!(adapters[0].module_type, "adapter");
    }

    #[test]
    fn test_enable_disable_module() {
        let mut registry = ModuleRegistry::new();

        let metadata = ModuleMetadata {
            module_id: "test-module".to_string(),
            module_type: "adapter".to_string(),
            binary_path: PathBuf::from("/path/to/binary"),
            source_path: None,
            version: "1.0.0".to_string(),
            interface: ModuleInterface {
                module_id: "test-module".to_string(),
                subscribes: vec!["agent.job".to_string()],
                publishes: vec!["agent.result".to_string()],
                version: "1.0.0".to_string(),
            },
            enabled: true,
            installed_at: 1234567890,
        };

        registry.register_module(metadata).unwrap();
        assert!(registry.is_enabled("test-module"));

        registry.disable_module("test-module").unwrap();
        assert!(!registry.is_enabled("test-module"));

        registry.enable_module("test-module").unwrap();
        assert!(registry.is_enabled("test-module"));
    }
}
