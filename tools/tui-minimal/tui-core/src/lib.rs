//! Core application logic for minimal TUI
//! 
//! Coordinates between configuration, NATS, and rendering

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;
use tui_config::TuiConfig;
use tui_nats::{TuiNatsManager, TaskComplete};

/// Application state
#[derive(Debug)]
pub struct AppState {
    pub config: TuiConfig,
    pub nats_manager: Arc<TuiNatsManager>,
}

impl AppState {
    pub fn new(config: TuiConfig) -> Self {
        Self {
            config,
            nats_manager: Arc::new(TuiNatsManager::new()),
        }
    }
    
    /// Connect to NATS
    pub async fn connect_nats(&self) -> Result<()> {
        self.nats_manager.connect(&self.config.nats_url).await
    }
    
    /// Check if NATS is connected
    pub fn is_nats_connected(&self) -> bool {
        self.nats_manager.is_connected()
    }
    
    /// Submit a task to NATS
    pub async fn submit_task(&self, user_input: &str) -> Result<String> {
        self.nats_manager.submit_task(user_input).await
    }
    
    /// Handle task completion
    pub async fn handle_task_complete(&self, complete: TaskComplete) {
        // In a real implementation, this would update the UI state
        // For now, just remove from pending tasks
        self.nats_manager.complete_task(&complete.session_id).await;
    }
}

/// Application
pub struct Application {
    state: Arc<RwLock<AppState>>,
}

impl Application {
    pub fn new(config: TuiConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(AppState::new(config))),
        }
    }
    
    /// Get state
    pub fn state(&self) -> Arc<RwLock<AppState>> {
        self.state.clone()
    }
    
    /// Initialize application
    pub async fn initialize(&self) -> Result<()> {
        let state = self.state.read().await;
        state.connect_nats().await?;
        Ok(())
    }
    
    /// Shutdown application
    pub async fn shutdown(&self) -> Result<()> {
        // Cleanup logic here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_app_state_creation() {
        let config = TuiConfig::default();
        let state = AppState::new(config);
        assert!(!state.is_nats_connected());
    }
    
    #[tokio::test]
    async fn test_application_creation() {
        let config = TuiConfig::default();
        let app = Application::new(config);
        assert!(app.state().read().await.config.nats_url == "nats://localhost:4222");
    }
}
