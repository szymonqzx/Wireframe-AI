//! Universal plugin registry for dynamic plugin management.

use crate::config::{ConfigError, PluginConfig};
use crate::plugin::{Plugin, PluginError};
use std::sync::Arc;

/// Universal plugin registry.
///
/// Maintains a registry of all loaded plugins and provides
/// methods for registration, retrieval, and lifecycle management.
/// Uses dashmap for better concurrent access performance.
pub struct PluginRegistry {
    plugins: dashmap::DashMap<String, Arc<dyn Plugin>>,
}

impl PluginRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            plugins: dashmap::DashMap::new(),
        }
    }

    /// Register a plugin.
    #[inline]
    pub fn register(&self, plugin: Box<dyn Plugin>) -> Result<(), PluginError> {
        let id = plugin.plugin_id().to_string();
        self.plugins.insert(id, Arc::from(plugin));
        Ok(())
    }

    /// Get a plugin by ID and downcast to specific type.
    ///
    /// This is unsafe and requires the caller to know the correct type.
    /// For type-safe retrieval, use module-specific getters.
    #[inline]
    pub fn get<T: Plugin + 'static>(&self, plugin_id: &str) -> Result<Arc<T>, PluginError> {
        self.plugins
            .get(plugin_id)
            .and_then(|p| {
                // Downcast Arc<dyn Plugin> to Arc<T>
                Arc::downcast::<T>(p.clone()).ok()
            })
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))
    }

    /// Check if a plugin is registered.
    #[inline]
    pub fn is_registered(&self, plugin_id: &str) -> bool {
        self.plugins.contains_key(plugin_id)
    }

    /// Get the count of registered plugins.
    #[inline]
    pub fn count(&self) -> usize {
        self.plugins.len()
    }

    /// List all registered plugin IDs.
    #[inline]
    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.iter().map(|k| k.key().clone()).collect()
    }

    /// Unregister a plugin.
    #[inline]
    pub fn unregister(&self, plugin_id: &str) -> Result<(), PluginError> {
        self.plugins
            .remove(plugin_id)
            .ok_or_else(|| PluginError::NotFound(plugin_id.to_string()))?;
        Ok(())
    }

    /// Clear all plugins.
    #[inline]
    pub fn clear(&self) {
        self.plugins.clear();
    }

    /// Load plugins from configuration file.
    ///
    /// This method reads a configuration file and attempts to load
    /// plugins for each module. Note: This is a placeholder for the
    /// actual plugin loading logic, which will be implemented in
    /// later phases when we create the actual plugins.
    pub async fn load_from_config(
        &self,
        config_path: &std::path::PathBuf,
    ) -> Result<(), ConfigError> {
        let config = PluginConfig::from_file(config_path)?;

        // Placeholder: In later phases, this will:
        // 1. For each module in config
        // 2. Load the specified plugins
        // 3. Initialize them with their config
        // 4. Register them in the registry

        tracing::info!("Loaded configuration with {} modules", config.modules.len());

        Ok(())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_empty_registry() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.count(), 0);
        assert!(registry.list_plugins().is_empty());
    }

    #[test]
    fn test_default_creates_empty_registry() {
        let registry = PluginRegistry::default();
        assert_eq!(registry.count(), 0);
        assert!(registry.list_plugins().is_empty());
    }
}
