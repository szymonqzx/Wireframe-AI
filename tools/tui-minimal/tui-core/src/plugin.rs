//! Plugin system for the TUI
//!
//! Provides the Plugin trait and PluginManager

use anyhow::Result;

/// Context provided to plugins for rendering
pub struct RenderContext<'a, 'b> {
    pub frame: &'a mut ratatui::Frame<'b>,
    pub area: ratatui::layout::Rect,
}

/// Input events routed to plugins
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginEvent {
    Input(char),
    Backspace,
    Enter,
    Esc,
    Up,
    Down,
    Left,
    Right,
    Ctrl(char),
    // other events
}

#[async_trait::async_trait]
pub trait Plugin: Send + Sync {
    /// Return the name of the plugin
    fn name(&self) -> &'static str;

    /// Initialize the plugin
    async fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    /// Handle an input event.
    /// Returns true if the event was consumed, false if it should be passed to the next plugin.
    async fn handle_event(&mut self, event: &PluginEvent) -> Result<bool>;

    /// Render the plugin's UI. This is called during the main render loop.
    /// Plugins can render overlays or specific areas.
    fn render(&self, ctx: &mut RenderContext) -> Result<()>;
}

/// Manages registered plugins and routes events/rendering to them
pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    pub fn register(&mut self, plugin: Box<dyn Plugin>) {
        self.plugins.push(plugin);
    }

    pub async fn handle_event(&mut self, event: &PluginEvent) -> Result<bool> {
        // Iterate backwards so overlays (added later) get events first
        for plugin in self.plugins.iter_mut().rev() {
            if plugin.handle_event(event).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn render_all(&self, ctx: &mut RenderContext) -> Result<()> {
        for plugin in &self.plugins {
            plugin.render(ctx)?;
        }
        Ok(())
    }
}
