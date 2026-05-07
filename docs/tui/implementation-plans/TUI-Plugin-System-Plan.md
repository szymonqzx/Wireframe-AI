# TUI Plugin System Detailed Implementation Plan

**Generated:** 2025-01-07 | **Scope:** Wireframe-AI TUI Plugin Architecture Extension

## Executive Summary

This document provides a comprehensive analysis of Wireframe-AI's current plugin architecture and a detailed plan for extending it to support TUI-specific functionality. The plan leverages the existing plugin infrastructure while adding TUI-specific traits and capabilities, maintaining consistency with the broader Wireframe-AI ecosystem.

---

## Current Plugin Architecture Analysis

### 1. Core Plugin Infrastructure

#### Base Plugin Trait
**Location:** `sdk/agentic-sdk/src/plugin.rs`

```rust
#[async_trait]
pub trait Plugin: Send + Sync + Any {
    fn plugin_id(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn description(&self) -> &'static str;
    async fn initialize(&mut self, config: &Value) -> Result<(), PluginError>;
    async fn health_check(&self) -> Result<bool, PluginError>;
    async fn shutdown(&mut self) -> Result<(), PluginError>;
}
```

**Key Characteristics:**
- Async lifecycle methods (initialize, health_check, shutdown)
- JSON-based configuration via `serde_json::Value`
- Type-safe downcasting via `PluginRegistry`
- Thread-safe (`Send + Sync`)

#### Plugin Registry
**Location:** `sdk/agentic-sdk/src/plugin_registry.rs`

```rust
pub struct PluginRegistry {
    plugins: dashmap::DashMap<String, Arc<dyn Plugin>>,
}
```

**Capabilities:**
- Concurrent plugin access via `dashmap`
- Type-safe downcasting with `Arc::downcast`
- Plugin lifecycle management
- Configuration-based plugin loading

#### Configuration System
**Location:** `sdk/agentic-sdk/src/config.rs`

**Features:**
- YAML/JSON configuration support
- Environment variable expansion (`${VAR}`)
- JSON schema validation (optional feature)
- Hot-reload via file watching
- Module-specific plugin configuration

### 2. Module-Specific Plugin Traits

#### Current Module Plugin Types

| Module | Plugin Traits | Current Implementations |
|--------|--------------|------------------------|
| Context | `StorageBackend`, `MemoryBackend`, `EnrichmentStrategy` | SQLite storage, FTS5 memory, Env enrichment |
| Orchestrator | `TaskPlanner`, `ExecutionStrategy`, `ResultSynthesizer` | Linear/hierarchical planners, parallel/sequential execution |
| Sandbox | `Tool`, `SecurityPolicy`, `ResourceLimiter` | File/HTTP/Shell tools, whitelist/custom policies, Unix limits |
| Interface | `InputMethod`, `OutputFormatter`, `UIComponent` | CLI input, Markdown output, (UIComponent unused) |

#### Interface Module Analysis
**Location:** `sdk/agentic-sdk/src/plugins/interface.rs`

**Current Traits:**
```rust
#[async_trait]
pub trait InputMethod: Plugin {
    async fn read_input(&self) -> Result<TaskSubmitted, InputError>;
}

#[async_trait]
pub trait OutputFormatter: Plugin {
    async fn format_result(&self, result: &TaskComplete) -> Result<String, FormatError>;
}

#[async_trait]
pub trait UIComponent: Plugin {
    async fn render(&self, state: &UIState) -> Result<(), UIError>;
}
```

**Current Implementations:**
- `input-cli`: Basic stdin input
- `output-markdown`: Markdown formatting
- **No UIComponent implementations** (trait exists but unused)

**Gap Analysis:**
- `UIComponent` trait is too basic for complex TUI needs
- No TUI-specific input handling (multi-line, vim modes, queueing)
- No real-time rendering capabilities
- No widget/component system
- No keyboard shortcut handling
- No theme system integration

### 3. Current TUI Architecture

#### Existing TUI Structure
**Location:** `tools/tui-minimal/`

**Current Crates:**
- `tui-main`: Entry point and event loop
- `tui-core`: Application state and NATS integration
- `tui-config`: TOML-based configuration (separate from plugin system)
- `tui-nats`: NATS message handling
- `tui-input`: Basic keyboard input
- `tui-render`: Ratatui-based rendering

**Current Limitations:**
- No integration with plugin system
- Hardcoded configuration format
- Monolithic architecture
- No extensibility points
- Basic input handling only

---

## TUI Plugin System Design

### 1. Architectural Principles

#### Design Goals
1. **Consistency:** Follow existing Wireframe-AI plugin patterns
2. **Extensibility:** Enable rich TUI functionality through plugins
3. **Performance:** Maintain TUI responsiveness
4. **Security:** No arbitrary code execution (configuration-driven)
5. **Simplicity:** Easy to develop and test plugins

#### Key Decisions
- **Extend existing Interface module** rather than create new module
- **Add TUI-specific traits** to `interface.rs` in SDK
- **Integrate with existing PluginRegistry** for unified management
- **Configuration-driven approach** (no Lua/scripting)
- **Async-first design** for non-blocking TUI operations

### 2. TUI-Specific Plugin Traits

#### Extended Interface Module Traits
**Location:** `sdk/agentic-sdk/src/plugins/interface.rs` (extension)

```rust
// ============================================================================
// Existing Traits (preserve for backward compatibility)
// ============================================================================

#[async_trait]
pub trait InputMethod: Plugin {
    async fn read_input(&self) -> Result<TaskSubmitted, InputError>;
}

#[async_trait]
pub trait OutputFormatter: Plugin {
    async fn format_result(&self, result: &TaskComplete) -> Result<String, FormatError>;
}

#[async_trait]
pub trait UIComponent: Plugin {
    async fn render(&self, state: &UIState) -> Result<(), UIError>;
}

// ============================================================================
// New TUI-Specific Traits
// ============================================================================

/// TUI input handler with advanced capabilities.
///
/// Extends basic input with multi-line support, keyboard shortcuts,
/// input queueing, and vim-like editing modes.
#[async_trait]
pub trait TuiInputHandler: Plugin {
    /// Handle a keyboard event and return the result.
    async fn handle_key_event(
        &self,
        event: &TuiKeyEvent,
    ) -> Result<TuiInputAction, TuiInputError>;

    /// Get the current input buffer content.
    async fn get_input_buffer(&self) -> Result<String, TuiInputError>;

    /// Set the input buffer content.
    async fn set_input_buffer(&self, content: &str) -> Result<(), TuiInputError>;

    /// Get cursor position in the input buffer.
    async fn get_cursor_position(&self) -> Result<usize, TuiInputError>;

    /// Set cursor position in the input buffer.
    async fn set_cursor_position(&self, pos: usize) -> Result<(), TuiInputError>;

    /// Check if input queue has pending messages.
    async fn has_pending_input(&self) -> Result<bool, TuiInputError>;

    /// Flush pending input queue.
    async fn flush_pending_input(&self) -> Result<Vec<String>, TuiInputError>;

    /// Get input mode (Normal, Insert, Visual, etc.).
    async fn get_input_mode(&self) -> Result<InputMode, TuiInputError>;

    /// Set input mode.
    async fn set_input_mode(&self, mode: InputMode) -> Result<(), TuiInputError>;
}

/// TUI widget for rendering UI components.
///
/// Provides rich widget capabilities for panels, status bars,
/// notifications, and custom UI elements.
#[async_trait]
pub trait TuiWidget: Plugin {
    /// Render the widget to the terminal buffer.
    async fn render(
        &self,
        area: ratatui::layout::Rect,
        buffer: &mut ratatui::buffer::Buffer,
        state: &TuiWidgetState,
    ) -> Result<(), TuiWidgetError>;

    /// Get the widget's preferred size constraints.
    async fn get_size_constraints(&self) -> Result<SizeConstraints, TuiWidgetError>;

    /// Handle a widget-specific event.
    async fn handle_event(
        &self,
        event: &TuiWidgetEvent,
    ) -> Result<Option<TuiWidgetAction>, TuiWidgetError>;

    /// Check if the widget is visible.
    async fn is_visible(&self) -> Result<bool, TuiWidgetError>;

    /// Set widget visibility.
    async fn set_visible(&self, visible: bool) -> Result<(), TuiWidgetError>;

    /// Get widget z-index (for layering).
    async fn get_z_index(&self) -> Result<u8, TuiWidgetError>;
}

/// TUI status bar component.
///
/// Displays real-time status information (connection, agent state,
/// pending tasks, etc.).
#[async_trait]
pub trait TuiStatusBar: Plugin {
    /// Get current status items to display.
    async fn get_status_items(&self) -> Result<Vec<StatusItem>, TuiStatusBarError>;

    /// Update a specific status item.
    async fn update_status_item(
        &self,
        key: &str,
        value: &str,
    ) -> Result<(), TuiStatusBarError>;

    /// Get status bar height.
    async fn get_height(&self) -> Result<u16, TuiStatusBarError>;

    /// Get status bar position (top/bottom).
    async fn get_position(&self) -> Result<StatusBarPosition, TuiStatusBarError>;
}

/// TUI theme provider.
///
/// Manages color schemes, fonts, and visual styling for the TUI.
#[async_trait]
pub trait TuiThemeProvider: Plugin {
    /// Get color for a specific semantic role.
    async fn get_color(&self, role: ColorRole) -> Result<ratatui::style::Color, TuiThemeError>;

    /// Get the current theme mode.
    async fn get_theme_mode(&self) -> Result<ThemeMode, TuiThemeError>;

    /// Set theme mode.
    async fn set_theme_mode(&self, mode: ThemeMode) -> Result<(), TuiThemeError>;

    /// Check if high contrast mode is enabled.
    async fn is_high_contrast(&self) -> Result<bool, TuiThemeError>;

    /// Enable/disable high contrast mode.
    async fn set_high_contrast(&self, enabled: bool) -> Result<(), TuiThemeError>;

    /// Get font configuration.
    async fn get_font_config(&self) -> Result<FontConfig, TuiThemeError>;
}

/// TUI session manager.
///
/// Handles session persistence, auto-save, and session switching.
#[async_trait]
pub trait TuiSessionManager: Plugin {
    /// Create a new session.
    async fn create_session(&self, metadata: SessionMetadata) -> Result<String, TuiSessionError>;

    /// Load a session by ID.
    async fn load_session(&self, session_id: &str) -> Result<ChatSession, TuiSessionError>;

    /// Save current session state.
    async fn save_session(&self, session: &ChatSession) -> Result<(), TuiSessionError>;

    /// List all available sessions.
    async fn list_sessions(&self) -> Result<Vec<SessionInfo>, TuiSessionError>;

    /// Delete a session.
    async fn delete_session(&self, session_id: &str) -> Result<(), TuiSessionError>;

    /// Get current active session ID.
    async fn get_active_session(&self) -> Result<Option<String>, TuiSessionError>;

    /// Set active session.
    async fn set_active_session(&self, session_id: &str) -> Result<(), TuiSessionError>;

    /// Auto-save current session.
    async fn auto_save(&self) -> Result<(), TuiSessionError>;
}

/// TUI command palette.
///
/// Provides searchable command interface with keyboard shortcuts.
#[async_trait]
pub trait TuiCommandPalette: Plugin {
    /// Register a command.
    async fn register_command(&self, command: Command) -> Result<(), TuiCommandError>;

    /// Unregister a command.
    async fn unregister_command(&self, command_id: &str) -> Result<(), TuiCommandError>;

    /// Execute a command by ID.
    async fn execute_command(
        &self,
        command_id: &str,
        args: &Value,
    ) -> Result<Value, TuiCommandError>;

    /// Search for commands matching query.
    async fn search_commands(&self, query: &str) -> Result<Vec<Command>, TuiCommandError>;

    /// Get keyboard shortcut for a command.
    async fn get_shortcut(&self, command_id: &str) -> Result<Option<String>, TuiCommandError>;

    /// Set keyboard shortcut for a command.
    async fn set_shortcut(
        &self,
        command_id: &str,
        shortcut: &str,
    ) -> Result<(), TuiCommandError>;
}

/// TUI notification system.
///
/// Manages toast notifications, alerts, and system messages.
#[async_trait]
pub trait TuiNotificationSystem: Plugin {
    /// Show a notification.
    async fn show_notification(
        &self,
        level: NotificationLevel,
        title: &str,
        message: &str,
    ) -> Result<String, TuiNotificationError>;

    /// Dismiss a notification by ID.
    async fn dismiss_notification(&self, notification_id: &str) -> Result<(), TuiNotificationError>;

    /// Get all active notifications.
    async fn get_active_notifications(&self) -> Result<Vec<Notification>, TuiNotificationError>;

    /// Clear all notifications.
    async fn clear_all(&self) -> Result<(), TuiNotificationError>;

    /// Set notification duration.
    async fn set_duration(&self, duration_ms: u64) -> Result<(), TuiNotificationError>;
}
```

### 3. Supporting Data Types

**Location:** `sdk/agentic-sdk/src/plugins/interface.rs` (extension)

```rust
// ============================================================================
// TUI Input Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct TuiKeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyCode {
    Char(char),
    Enter,
    Backspace,
    Delete,
    Esc,
    Home,
    End,
    Left,
    Right,
    Up,
    Down,
    PageUp,
    PageDown,
    Tab,
    F(u8),
    Null,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KeyModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
}

#[derive(Debug, Clone)]
pub enum TuiInputAction {
    None,
    Submit(String),
    QueueInput(String),
    ChangeMode(InputMode),
    Custom(Value),
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Insert,
    Visual,
    Command,
    Replace,
}

#[derive(Debug, Clone, Copy)]
pub enum TuiInputError {
    BufferAccessFailed,
    CursorAccessFailed,
    QueueAccessFailed,
    ModeChangeFailed,
    InvalidInput,
}

// ============================================================================
// TUI Widget Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct TuiWidgetState {
    pub focused: bool,
    pub hovered: bool,
    pub data: Value,
}

#[derive(Debug, Clone)]
pub enum TuiWidgetEvent {
    FocusGained,
    FocusLost,
    MouseClick { x: u16, y: u16 },
    MouseScroll { delta: i16 },
    KeyPress(TuiKeyEvent),
    Custom(Value),
}

#[derive(Debug, Clone)]
pub enum TuiWidgetAction {
    None,
    FocusNext,
    FocusPrevious,
    Close,
    Custom(Value),
}

#[derive(Debug, Clone)]
pub struct SizeConstraints {
    pub min_width: Option<u16>,
    pub max_width: Option<u16>,
    pub min_height: Option<u16>,
    pub max_height: Option<u16>,
}

#[derive(Debug, Clone, Copy)]
pub enum TuiWidgetError {
    RenderFailed,
    SizeConstraintFailed,
    EventHandlingFailed,
    InvalidState,
}

// ============================================================================
// TUI Status Bar Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct StatusItem {
    pub key: String,
    pub label: String,
    pub value: String,
    pub color: String,
    pub icon: char,
    pub priority: u8,
}

#[derive(Debug, Clone, Copy)]
pub enum StatusBarPosition {
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
pub enum TuiStatusBarError {
    ItemNotFound,
    UpdateFailed,
    InvalidPosition,
}

// ============================================================================
// TUI Theme Types
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub enum ColorRole {
    User,
    Assistant,
    System,
    Error,
    Warning,
    Info,
    Success,
    Border,
    Background,
    Foreground,
}

#[derive(Debug, Clone, Copy)]
pub enum ThemeMode {
    Default,
    Dark,
    Light,
    HighContrast,
}

#[derive(Debug, Clone)]
pub struct FontConfig {
    pub family: String,
    pub size: u16,
    pub weight: String,
}

#[derive(Debug, Clone, Copy)]
pub enum TuiThemeError {
    ColorNotFound,
    ThemeChangeFailed,
    InvalidConfig,
}

// ============================================================================
// TUI Session Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub messages: Vec<ChatMessage>,
    pub metadata: SessionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub title: String,
    pub tags: Vec<String>,
    pub agent: String,
    pub model: String,
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub created_at: i64,
    pub message_count: usize,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum TuiSessionError {
    SessionNotFound,
    SaveFailed,
    LoadFailed,
    DeleteFailed,
    InvalidMetadata,
}

// ============================================================================
// TUI Command Palette Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct Command {
    pub id: String,
    pub label: String,
    pub description: String,
    pub category: String,
    pub shortcut: Option<String>,
    pub handler: String, // Plugin ID that handles this command
}

#[derive(Debug, Clone, Copy)]
pub enum TuiCommandError {
    CommandNotFound,
    ExecutionFailed,
    RegistrationFailed,
    ShortcutConflict,
}

// ============================================================================
// TUI Notification Types
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct Notification {
    pub id: String,
    pub level: NotificationLevel,
    pub title: String,
    pub message: String,
    pub created_at: i64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum TuiNotificationError {
    NotificationNotFound,
    DismissFailed,
    InvalidLevel,
}
```

### 4. Plugin Configuration Integration

#### Extended Plugin Configuration
**Location:** `sdk/agentic-sdk/src/config.rs` (extension)

```rust
// Add to ModulePlugins struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulePlugins {
    // Existing fields...
    pub storage: Option<PluginSpec>,
    pub memory: Option<PluginSpec>,
    pub enrichment_pipeline: Vec<EnrichmentStep>,
    pub planner: Option<PluginSpec>,
    pub execution: Option<PluginSpec>,
    pub synthesizer: Option<PluginSpec>,
    pub tools: Vec<PluginSpec>,
    pub security: Option<PluginSpec>,
    pub resources: Option<PluginSpec>,
    pub input: Option<PluginSpec>,
    pub output: Option<PluginSpec>,
    
    // NEW: TUI-specific plugins
    pub tui_input_handler: Option<PluginSpec>,
    pub tui_widget: Vec<PluginSpec>,
    pub tui_status_bar: Option<PluginSpec>,
    pub tui_theme_provider: Option<PluginSpec>,
    pub tui_session_manager: Option<PluginSpec>,
    pub tui_command_palette: Option<PluginSpec>,
    pub tui_notification_system: Option<PluginSpec>,
}
```

#### Example TUI Configuration
**File:** `configs/interface-tui.yaml`

```yaml
modules:
  interface:
    enabled: true
    plugins:
      # Basic I/O (existing)
      input:
        plugin_id: "input-cli"
        config:
          prompt: "> "
      
      output:
        plugin_id: "output-markdown"
        config:
          format: "github"
      
      # NEW: TUI-specific plugins
      tui_input_handler:
        plugin_id: "tui-input-advanced"
        config:
          enable_vim_mode: true
          enable_queueing: true
          history_size: 100
          auto_save_interval: 30
      
      tui_widget:
        - plugin_id: "tui-panel-tools"
          config:
            position: "right"
            width: 30
            order: 1
        - plugin_id: "tui-panel-logs"
          config:
            position: "bottom"
            height: 10
            order: 2
      
      tui_status_bar:
        plugin_id: "tui-status-nats"
        config:
          position: "bottom"
          show_connection: true
          show_pending: true
          show_agent: true
      
      tui_theme_provider:
        plugin_id: "tui-theme-default"
        config:
          mode: "dark"
          high_contrast: false
          custom_colors:
            user: "#00ffff"
            assistant: "#00ff00"
            system: "#ffff00"
      
      tui_session_manager:
        plugin_id: "tui-session-file"
        config:
          session_dir: "~/.wireframe-tui/sessions"
          auto_save: true
          auto_save_interval: 60
          max_sessions: 100
      
      tui_command_palette:
        plugin_id: "tui-commands-default"
        config:
          shortcut: "Ctrl+P"
          fuzzy_search: true
      
      tui_notification_system:
        plugin_id: "tui-notification-toast"
        config:
          duration: 5000
          max_visible: 5
          position: "top-right"
```

---

## Implementation Plan

### Phase 1: SDK Extensions (Week 1-2)

#### 1.1 Extend Interface Plugin Traits
**File:** `sdk/agentic-sdk/src/plugins/interface.rs`

**Tasks:**
- [ ] Add TUI-specific trait definitions
- [ ] Add supporting data types
- [ ] Add error types for each trait
- [ ] Update documentation
- [ ] Add comprehensive unit tests

**Dependencies:** None (SDK-only changes)

**Testing:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tui_key_event_serialization() {
        let event = TuiKeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers { shift: false, control: true, alt: false },
            timestamp: 1234567890,
        };
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: TuiKeyEvent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(event.code, deserialized.code);
    }
    
    #[tokio::test]
    async fn test_status_item_creation() {
        let item = StatusItem {
            key: "nats".to_string(),
            label: "NATS".to_string(),
            value: "Connected".to_string(),
            color: "green".to_string(),
            icon: '●',
            priority: 1,
        };
        assert_eq!(item.key, "nats");
    }
}
```

#### 1.2 Extend Plugin Configuration
**File:** `sdk/agentic-sdk/src/config.rs`

**Tasks:**
- [ ] Add TUI plugin fields to `ModulePlugins`
- [ ] Update configuration validation schema
- [ ] Add configuration migration logic
- [ ] Update configuration tests

**Dependencies:** Phase 1.1

**Testing:**
```rust
#[test]
fn test_tui_plugin_config_parsing() {
    let yaml = r#"
modules:
  interface:
    enabled: true
    plugins:
      tui_input_handler:
        plugin_id: "tui-input-advanced"
        config:
          enable_vim_mode: true
"#;
    let config = PluginConfig::from_yaml(yaml).unwrap();
    let tui_input = config.modules["interface"].plugins.tui_input_handler;
    assert!(tui_input.is_some());
    assert_eq!(tui_input.unwrap().plugin_id, "tui-input-advanced");
}
```

#### 1.3 Update Plugin Registry
**File:** `sdk/agentic-sdk/src/plugin_registry.rs`

**Tasks:**
- [ ] Add TUI-specific plugin getter methods
- [ ] Add plugin type validation
- [ ] Update documentation
- [ ] Add integration tests

**Dependencies:** Phase 1.1, 1.2

**Implementation:**
```rust
impl PluginRegistry {
    /// Get TUI input handler plugin.
    pub fn get_tui_input_handler(&self, plugin_id: &str) -> Result<Arc<dyn TuiInputHandler>, PluginError> {
        self.get(plugin_id)
    }
    
    /// Get TUI widget plugin.
    pub fn get_tui_widget(&self, plugin_id: &str) -> Result<Arc<dyn TuiWidget>, PluginError> {
        self.get(plugin_id)
    }
    
    /// Get TUI status bar plugin.
    pub fn get_tui_status_bar(&self, plugin_id: &str) -> Result<Arc<dyn TuiStatusBar>, PluginError> {
        self.get(plugin_id)
    }
    
    /// Get TUI theme provider plugin.
    pub fn get_tui_theme_provider(&self, plugin_id: &str) -> Result<Arc<dyn TuiThemeProvider>, PluginError> {
        self.get(plugin_id)
    }
    
    /// Get TUI session manager plugin.
    pub fn get_tui_session_manager(&self, plugin_id: &str) -> Result<Arc<dyn TuiSessionManager>, PluginError> {
        self.get(plugin_id)
    }
    
    /// Get TUI command palette plugin.
    pub fn get_tui_command_palette(&self, plugin_id: &str) -> Result<Arc<dyn TuiCommandPalette>, PluginError> {
        self.get(plugin_id)
    }
    
    /// Get TUI notification system plugin.
    pub fn get_tui_notification_system(&self, plugin_id: &str) -> Result<Arc<dyn TuiNotificationSystem>, PluginError> {
        self.get(plugin_id)
    }
}
```

### Phase 2: Built-in TUI Plugins (Week 3-5)

#### 2.1 Advanced Input Handler Plugin
**Location:** `plugins/interface/tui-input-advanced/`

**Structure:**
```
plugins/interface/tui-input-advanced/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── buffer.rs
│   ├── modes.rs
│   └── queue.rs
└── tests/
    └── input_tests.rs
```

**Implementation:**
```rust
// src/lib.rs
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::interface::{TuiInputHandler, TuiInputError, TuiInputAction, InputMode, TuiKeyEvent};
use async_trait::async_trait;
use serde_json::Value;

pub struct AdvancedInputHandler {
    buffer: InputBuffer,
    queue: InputQueue,
    mode: InputMode,
    config: InputConfig,
}

#[async_trait]
impl Plugin for AdvancedInputHandler {
    fn plugin_id(&self) -> &'static str {
        "tui-input-advanced"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Advanced TUI input handler with vim modes and queueing"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), PluginError> {
        if let Some(enable_vim) = config.get("enable_vim_mode").and_then(|v| v.as_bool()) {
            self.config.enable_vim_mode = enable_vim;
        }
        if let Some(enable_queueing) = config.get("enable_queueing").and_then(|v| v.as_bool()) {
            self.config.enable_queueing = enable_queueing;
        }
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait]
impl TuiInputHandler for AdvancedInputHandler {
    async fn handle_key_event(&self, event: &TuiKeyEvent) -> Result<TuiInputAction, TuiInputError> {
        match self.mode {
            InputMode::Normal => self.handle_normal_mode(event),
            InputMode::Insert => self.handle_insert_mode(event),
            InputMode::Visual => self.handle_visual_mode(event),
            InputMode::Command => self.handle_command_mode(event),
        }
    }

    async fn get_input_buffer(&self) -> Result<String, TuiInputError> {
        Ok(self.buffer.content())
    }

    async fn set_input_buffer(&self, content: &str) -> Result<(), TuiInputError> {
        self.buffer.set_content(content);
        Ok(())
    }

    async fn get_cursor_position(&self) -> Result<usize, TuiInputError> {
        Ok(self.buffer.cursor_position())
    }

    async fn set_cursor_position(&self, pos: usize) -> Result<(), TuiInputError> {
        self.buffer.set_cursor_position(pos);
        Ok(())
    }

    async fn has_pending_input(&self) -> Result<bool, TuiInputError> {
        Ok(!self.queue.is_empty())
    }

    async fn flush_pending_input(&self) -> Result<Vec<String>, TuiInputError> {
        Ok(self.queue.flush())
    }

    async fn get_input_mode(&self) -> Result<InputMode, TuiInputError> {
        Ok(self.mode)
    }

    async fn set_input_mode(&self, mode: InputMode) -> Result<(), TuiInputError> {
        self.mode = mode;
        Ok(())
    }
}
```

**Features:**
- Multi-line input with Shift+Enter
- Vim-like editing modes (Normal, Insert, Visual)
- Input queueing during AI turns
- History navigation (Up/Down arrows)
- Word operations (Ctrl+W, Ctrl+K, Ctrl+U)
- Cursor movement (Home, End, arrows)

#### 2.2 Status Bar Plugin
**Location:** `plugins/interface/tui-status-nats/`

**Implementation:**
```rust
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::interface::{TuiStatusBar, StatusItem, StatusBarPosition, TuiStatusBarError};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct NatsStatusBar {
    nats_manager: Arc<dyn NatsManager>,
    items: Arc<RwLock<Vec<StatusItem>>>,
    position: StatusBarPosition,
}

#[async_trait]
impl Plugin for NatsStatusBar {
    fn plugin_id(&self) -> &'static str {
        "tui-status-nats"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "NATS-aware status bar for TUI"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), PluginError> {
        // Parse configuration
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait]
impl TuiStatusBar for NatsStatusBar {
    async fn get_status_items(&self) -> Result<Vec<StatusItem>, TuiStatusBarError> {
        let items = self.items.read().await;
        Ok(items.clone())
    }

    async fn update_status_item(&self, key: &str, value: &str) -> Result<(), TuiStatusBarError> {
        let mut items = self.items.write().await;
        if let Some(item) = items.iter_mut().find(|i| i.key == key) {
            item.value = value.to_string();
        }
        Ok(())
    }

    async fn get_height(&self) -> Result<u16, TuiStatusBarError> {
        Ok(1)
    }

    async fn get_position(&self) -> Result<StatusBarPosition, TuiStatusBarError> {
        Ok(self.position)
    }
}
```

#### 2.3 Session Manager Plugin
**Location:** `plugins/interface/tui-session-file/`

**Implementation:**
```rust
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::interface::{TuiSessionManager, ChatSession, SessionMetadata, SessionInfo, TuiSessionError};
use async_trait::async_trait;
use std::path::PathBuf;

pub struct FileSessionManager {
    session_dir: PathBuf,
    auto_save: bool,
    auto_save_interval: u64,
}

#[async_trait]
impl Plugin for FileSessionManager {
    fn plugin_id(&self) -> &'static str {
        "tui-session-file"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "File-based session persistence for TUI"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), PluginError> {
        // Initialize session directory
        std::fs::create_dir_all(&self.session_dir)?;
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(self.session_dir.exists())
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        // Ensure final save
        Ok(())
    }
}

#[async_trait]
impl TuiSessionManager for FileSessionManager {
    async fn create_session(&self, metadata: SessionMetadata) -> Result<String, TuiSessionError> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let session = ChatSession {
            id: session_id.clone(),
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            messages: vec![],
            metadata,
        };
        self.save_session(&session).await?;
        Ok(session_id)
    }

    async fn load_session(&self, session_id: &str) -> Result<ChatSession, TuiSessionError> {
        let path = self.session_dir.join(format!("{}.json", session_id));
        let content = std::fs::read_to_string(&path)
            .map_err(|e| TuiSessionError::LoadFailed(e.to_string()))?;
        let session: ChatSession = serde_json::from_str(&content)
            .map_err(|e| TuiSessionError::LoadFailed(e.to_string()))?;
        Ok(session)
    }

    async fn save_session(&self, session: &ChatSession) -> Result<(), TuiSessionError> {
        let path = self.session_dir.join(format!("{}.json", session.id));
        let content = serde_json::to_string_pretty(session)
            .map_err(|e| TuiSessionError::SaveFailed(e.to_string()))?;
        std::fs::write(&path, content)
            .map_err(|e| TuiSessionError::SaveFailed(e.to_string()))?;
        Ok(())
    }

    async fn list_sessions(&self) -> Result<Vec<SessionInfo>, TuiSessionError> {
        let mut sessions = vec![];
        for entry in std::fs::read_dir(&self.session_dir)
            .map_err(|e| TuiSessionError::LoadFailed(e.to_string()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| TuiSessionError::LoadFailed(e.to_string()))?;
                let session: ChatSession = serde_json::from_str(&content)
                    .map_err(|e| TuiSessionError::LoadFailed(e.to_string()))?;
                sessions.push(SessionInfo {
                    id: session.id,
                    title: session.metadata.title,
                    created_at: session.created_at,
                    message_count: session.messages.len(),
                    tags: session.metadata.tags,
                });
            }
        }
        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(sessions)
    }

    async fn delete_session(&self, session_id: &str) -> Result<(), TuiSessionError> {
        let path = self.session_dir.join(format!("{}.json", session_id));
        std::fs::remove_file(&path)
            .map_err(|e| TuiSessionError::DeleteFailed(e.to_string()))?;
        Ok(())
    }

    async fn get_active_session(&self) -> Result<Option<String>, TuiSessionError> {
        // Read from active session file
        let active_path = self.session_dir.join("active.json");
        if active_path.exists() {
            let content = std::fs::read_to_string(&active_path)
                .map_err(|e| TuiSessionError::LoadFailed(e.to_string()))?;
            Ok(Some(content))
        } else {
            Ok(None)
        }
    }

    async fn set_active_session(&self, session_id: &str) -> Result<(), TuiSessionError> {
        let active_path = self.session_dir.join("active.json");
        std::fs::write(&active_path, session_id)
            .map_err(|e| TuiSessionError::SaveFailed(e.to_string()))?;
        Ok(())
    }

    async fn auto_save(&self) -> Result<(), TuiSessionError> {
        // Triggered by timer
        Ok(())
    }
}
```

#### 2.4 Theme Provider Plugin
**Location:** `plugins/interface/tui-theme-default/`

**Implementation:**
```rust
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::interface::{TuiThemeProvider, ColorRole, ThemeMode, FontConfig, TuiThemeError};
use async_trait::async_trait;
use ratatui::style::Color;

pub struct DefaultThemeProvider {
    mode: ThemeMode,
    high_contrast: bool,
    custom_colors: std::collections::HashMap<ColorRole, String>,
}

#[async_trait]
impl Plugin for DefaultThemeProvider {
    fn plugin_id(&self) -> &'static str {
        "tui-theme-default"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Default theme provider for TUI"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), PluginError> {
        // Parse theme configuration
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait]
impl TuiThemeProvider for DefaultThemeProvider {
    async fn get_color(&self, role: ColorRole) -> Result<Color, TuiThemeError> {
        let color = match role {
            ColorRole::User => Color::Cyan,
            ColorRole::Assistant => Color::Green,
            ColorRole::System => Color::Yellow,
            ColorRole::Error => Color::Red,
            ColorRole::Warning => Color::Yellow,
            ColorRole::Info => Color::Blue,
            ColorRole::Success => Color::Green,
            ColorRole::Border => Color::Gray,
            ColorRole::Background => Color::Black,
            ColorRole::Foreground => Color::White,
        };
        Ok(color)
    }

    async fn get_theme_mode(&self) -> Result<ThemeMode, TuiThemeError> {
        Ok(self.mode)
    }

    async fn set_theme_mode(&self, mode: ThemeMode) -> Result<(), TuiThemeError> {
        self.mode = mode;
        Ok(())
    }

    async fn is_high_contrast(&self) -> Result<bool, TuiThemeError> {
        Ok(self.high_contrast)
    }

    async fn set_high_contrast(&self, enabled: bool) -> Result<(), TuiThemeError> {
        self.high_contrast = enabled;
        Ok(())
    }

    async fn get_font_config(&self) -> Result<FontConfig, TuiThemeError> {
        Ok(FontConfig {
            family: "monospace".to_string(),
            size: 14,
            weight: "normal".to_string(),
        })
    }
}
```

#### 2.5 Command Palette Plugin
**Location:** `plugins/interface/tui-commands-default/`

**Implementation:**
```rust
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::interface::{TuiCommandPalette, Command, TuiCommandError};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

pub struct DefaultCommandPalette {
    commands: HashMap<String, Command>,
    shortcuts: HashMap<String, String>,
}

#[async_trait]
impl Plugin for DefaultCommandPalette {
    fn plugin_id(&self) -> &'static str {
        "tui-commands-default"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Default command palette for TUI"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), PluginError> {
        // Register default commands
        self.register_default_commands().await;
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

impl DefaultCommandPalette {
    async fn register_default_commands(&mut self) {
        let commands = vec![
            Command {
                id: "new-session".to_string(),
                label: "New Session".to_string(),
                description: "Create a new chat session".to_string(),
                category: "Session".to_string(),
                shortcut: Some("Ctrl+N".to_string()),
                handler: "tui-session-file".to_string(),
            },
            Command {
                id: "save-session".to_string(),
                label: "Save Session".to_string(),
                description: "Save current session".to_string(),
                category: "Session".to_string(),
                shortcut: Some("Ctrl+S".to_string()),
                handler: "tui-session-file".to_string(),
            },
            Command {
                id: "toggle-panels".to_string(),
                label: "Toggle Panels".to_string(),
                description: "Show/hide side panels".to_string(),
                category: "View".to_string(),
                shortcut: Some("F9".to_string()),
                handler: "tui-widget".to_string(),
            },
        ];
        
        for cmd in commands {
            let _ = self.register_command(cmd).await;
        }
    }
}

#[async_trait]
impl TuiCommandPalette for DefaultCommandPalette {
    async fn register_command(&self, command: Command) -> Result<(), TuiCommandError> {
        self.commands.insert(command.id.clone(), command);
        Ok(())
    }

    async fn unregister_command(&self, command_id: &str) -> Result<(), TuiCommandError> {
        self.commands.remove(command_id);
        Ok(())
    }

    async fn execute_command(&self, command_id: &str, args: &Value) -> Result<Value, TuiCommandError> {
        // Execute command via plugin handler
        Ok(serde_json::json!({"status": "executed"}))
    }

    async fn search_commands(&self, query: &str) -> Result<Vec<Command>, TuiCommandError> {
        let query_lower = query.to_lowercase();
        let results: Vec<Command> = self.commands
            .values()
            .filter(|cmd| {
                cmd.label.to_lowercase().contains(&query_lower)
                    || cmd.description.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect();
        Ok(results)
    }

    async fn get_shortcut(&self, command_id: &str) -> Result<Option<String>, TuiCommandError> {
        Ok(self.commands.get(command_id).and_then(|cmd| cmd.shortcut.clone()))
    }

    async fn set_shortcut(&self, command_id: &str, shortcut: &str) -> Result<(), TuiCommandError> {
        if let Some(cmd) = self.commands.get_mut(command_id) {
            cmd.shortcut = Some(shortcut.to_string());
        }
        Ok(())
    }
}
```

#### 2.6 Notification System Plugin
**Location:** `plugins/interface/tui-notification-toast/`

**Implementation:**
```rust
use agentic_sdk::plugin::Plugin;
use agentic_sdk::plugins::interface::{TuiNotificationSystem, Notification, NotificationLevel, TuiNotificationError};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ToastNotificationSystem {
    notifications: Arc<RwLock<Vec<Notification>>>,
    duration_ms: u64,
    max_visible: usize,
}

#[async_trait]
impl Plugin for ToastNotificationSystem {
    fn plugin_id(&self) -> &'static str {
        "tui-notification-toast"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Toast notification system for TUI"
    }

    async fn initialize(&mut self, config: &Value) -> Result<(), PluginError> {
        if let Some(duration) = config.get("duration").and_then(|v| v.as_u64()) {
            self.duration_ms = duration;
        }
        if let Some(max) = config.get("max_visible").and_then(|v| v.as_usize()) {
            self.max_visible = max;
        }
        Ok(())
    }

    async fn health_check(&self) -> Result<bool, PluginError> {
        Ok(true)
    }

    async fn shutdown(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}

#[async_trait]
impl TuiNotificationSystem for ToastNotificationSystem {
    async fn show_notification(
        &self,
        level: NotificationLevel,
        title: &str,
        message: &str,
    ) -> Result<String, TuiNotificationError> {
        let notification = Notification {
            id: uuid::Uuid::new_v4().to_string(),
            level,
            title: title.to_string(),
            message: message.to_string(),
            created_at: chrono::Utc::now().timestamp(),
            duration_ms: self.duration_ms,
        };
        
        let mut notifications = self.notifications.write().await;
        notifications.push(notification);
        Ok(notification.id)
    }

    async fn dismiss_notification(&self, notification_id: &str) -> Result<(), TuiNotificationError> {
        let mut notifications = self.notifications.write().await;
        notifications.retain(|n| n.id != notification_id);
        Ok(())
    }

    async fn get_active_notifications(&self) -> Result<Vec<Notification>, TuiNotificationError> {
        let notifications = self.notifications.read().await;
        Ok(notifications.clone())
    }

    async fn clear_all(&self) -> Result<(), TuiNotificationError> {
        let mut notifications = self.notifications.write().await;
        notifications.clear();
        Ok(())
    }

    async fn set_duration(&self, duration_ms: u64) -> Result<(), TuiNotificationError> {
        self.duration_ms = duration_ms;
        Ok(())
    }
}
```

### Phase 3: TUI Integration (Week 6-7)

#### 3.1 Update TUI Core
**File:** `tools/tui-minimal/tui-core/src/lib.rs`

**Changes:**
- Integrate with `PluginRegistry`
- Load TUI plugins from configuration
- Manage plugin lifecycle
- Provide plugin access to TUI components

**Implementation:**
```rust
use agentic_sdk::plugin::PluginRegistry;
use agentic_sdk::plugins::interface::{
    TuiInputHandler, TuiStatusBar, TuiThemeProvider, TuiSessionManager,
    TuiCommandPalette, TuiNotificationSystem
};

pub struct AppState {
    pub config: TuiConfig,
    pub nats_manager: Arc<TuiNatsManager>,
    // NEW: Plugin integration
    pub plugin_registry: Arc<PluginRegistry>,
    pub input_handler: Option<Arc<dyn TuiInputHandler>>,
    pub status_bar: Option<Arc<dyn TuiStatusBar>>,
    pub theme_provider: Option<Arc<dyn TuiThemeProvider>>,
    pub session_manager: Option<Arc<dyn TuiSessionManager>>,
    pub command_palette: Option<Arc<dyn TuiCommandPalette>>,
    pub notification_system: Option<Arc<dyn TuiNotificationSystem>>,
}

impl AppState {
    pub fn new(config: TuiConfig) -> Self {
        let plugin_registry = Arc::new(PluginRegistry::new());
        Self {
            config,
            nats_manager: Arc::new(TuiNatsManager::new()),
            plugin_registry,
            input_handler: None,
            status_bar: None,
            theme_provider: None,
            session_manager: None,
            command_palette: None,
            notification_system: None,
        }
    }
    
    pub async fn load_plugins(&mut self) -> Result<()> {
        let plugin_config = PluginConfig::from_file(&PathBuf::from("configs/interface-tui.yaml"))?;
        
        // Load TUI plugins
        if let Some(input_spec) = plugin_config.modules["interface"].plugins.tui_input_handler.clone() {
            // Load and initialize input handler plugin
            // self.input_handler = Some(load_plugin(&input_spec).await?);
        }
        
        // Load other TUI plugins similarly...
        
        Ok(())
    }
}
```

#### 3.2 Update TUI Render
**File:** `tools/tui-minimal/tui-render/src/lib.rs`

**Changes:**
- Use `TuiThemeProvider` for colors
- Use `TuiStatusBar` for status display
- Use `TuiWidget` for custom widgets
- Use `TuiNotificationSystem` for notifications

**Implementation:**
```rust
pub struct Renderer {
    state: Arc<RwLock<RenderState>>,
    // NEW: Plugin references
    theme_provider: Option<Arc<dyn TuiThemeProvider>>,
    status_bar: Option<Arc<dyn TuiStatusBar>>,
    widgets: Vec<Arc<dyn TuiWidget>>,
    notification_system: Option<Arc<dyn TuiNotificationSystem>>,
}

impl Renderer {
    pub fn new_with_plugins(
        state: Arc<RwLock<RenderState>>,
        theme_provider: Option<Arc<dyn TuiThemeProvider>>,
        status_bar: Option<Arc<dyn TuiStatusBar>>,
        widgets: Vec<Arc<dyn TuiWidget>>,
        notification_system: Option<Arc<dyn TuiNotificationSystem>>,
    ) -> Self {
        Self {
            state,
            theme_provider,
            status_bar,
            widgets,
            notification_system,
        }
    }
    
    pub async fn render(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, state: RenderState) -> Result<()> {
        terminal.draw(|f| {
            self.draw_ui(f, state).await;
        })?;
        Ok(())
    }
    
    async fn draw_ui(&self, f: &mut Frame, state: RenderState) {
        let size = f.area();
        
        // Get colors from theme provider
        let user_color = if let Some(theme) = &self.theme_provider {
            theme.get_color(ColorRole::User).await.unwrap_or(Color::Cyan)
        } else {
            Color::Cyan
        };
        
        // Draw status bar
        if let Some(status) = &self.status_bar {
            let status_items = status.get_status_items().await.unwrap_or_default();
            self.draw_status_bar(f, status_items);
        }
        
        // Draw widgets
        for widget in &self.widgets {
            if widget.is_visible().await.unwrap_or(false) {
                let constraints = widget.get_size_constraints().await.unwrap_or_default();
                let widget_area = self.calculate_widget_area(size, &constraints);
                let mut buffer = Buffer::filled(widget_area, Cell::default());
                widget.render(widget_area, &mut buffer, &state).await.unwrap();
                f.render_widget(Clear, widget_area); // Clear area
            }
        }
        
        // Draw notifications
        if let Some(notifications) = &self.notification_system {
            let active = notifications.get_active_notifications().await.unwrap_or_default();
            self.draw_notifications(f, active);
        }
    }
}
```

#### 3.3 Update TUI Input
**File:** `tools/tui-minimal/tui-input/src/lib.rs`

**Changes:**
- Delegate to `TuiInputHandler` plugin
- Remove hardcoded input logic
- Support plugin-based input modes

**Implementation:**
```rust
pub struct InputHandler {
    // NEW: Plugin-based input handling
    input_plugin: Option<Arc<dyn TuiInputHandler>>,
    event_sender: mpsc::UnboundedSender<InputEvent>,
}

impl InputHandler {
    pub fn new_with_plugin(input_plugin: Option<Arc<dyn TuiInputHandler>>) -> Self {
        let (event_sender, _event_receiver) = mpsc::unbounded_channel();
        Self {
            input_plugin,
            event_sender,
        }
    }
    
    pub async fn handle_key_event(&self, key: KeyEvent) -> InputEvent {
        if let Some(plugin) = &self.input_plugin {
            let tui_event = TuiKeyEvent {
                code: self.map_key_code(key.code),
                modifiers: self.map_key_modifiers(key.modifiers),
                timestamp: chrono::Utc::now().timestamp(),
            };
            
            match plugin.handle_key_event(&tui_event).await {
                Ok(TuiInputAction::Submit(input)) => InputEvent::Submit(input),
                Ok(TuiInputAction::QueueInput(input)) => InputEvent::Queue(input),
                Ok(TuiInputAction::ChangeMode(mode)) => InputEvent::ModeChange(mode),
                Ok(TuiInputAction::None) => InputEvent::None,
                Err(_) => InputEvent::Unknown,
            }
        } else {
            // Fallback to basic handling
            self.map_key_event_basic(key)
        }
    }
}
```

### Phase 4: Testing and Documentation (Week 8)

#### 4.1 Integration Tests
**Location:** `tools/tui-minimal/tests/plugin_integration_tests.rs`

```rust
#[tokio::test]
async fn test_plugin_loading() {
    let config = TuiConfig::default();
    let mut app = Application::new(config);
    app.load_plugins().await.unwrap();
    
    // Verify plugins are loaded
    assert!(app.state().plugin_registry.count() > 0);
}

#[tokio::test]
async fn test_input_handler_plugin() {
    let plugin = AdvancedInputHandler::new();
    let event = TuiKeyEvent {
        code: KeyCode::Char('a'),
        modifiers: KeyModifiers { shift: false, control: false, alt: false },
        timestamp: 0,
    };
    
    let action = plugin.handle_key_event(&event).await.unwrap();
    assert!(!matches!(action, TuiInputAction::None));
}

#[tokio::test]
async fn test_session_persistence() {
    let manager = FileSessionManager::new(PathBuf::from("/tmp/test-sessions"));
    let metadata = SessionMetadata {
        title: "Test Session".to_string(),
        tags: vec!["test".to_string()],
        agent: "test-agent".to_string(),
        model: "test-model".to_string(),
    };
    
    let session_id = manager.create_session(metadata).await.unwrap();
    let session = manager.load_session(&session_id).await.unwrap();
    assert_eq!(session.id, session_id);
    
    manager.delete_session(&session_id).await.unwrap();
}
```

#### 4.2 Documentation
**Files to create:**
- `docs/TUI-Plugin-Development.md` - Plugin development guide
- `docs/TUI-Plugin-API.md` - API reference
- `plugins/interface/README.md` - Plugin directory documentation
- Example plugin templates

---

## Migration Strategy

### Current TUI → Plugin-Based TUI

#### Step 1: Add Plugin Support (Non-Breaking)
- Keep existing TUI functionality
- Add plugin system alongside
- Feature flag for plugin mode

#### Step 2: Migrate Features Incrementally
- Migrate input handling to plugin
- Migrate status bar to plugin
- Migrate session management to plugin
- Keep fallback to original implementation

#### Step 3: Deprecate Old Implementation
- Mark old code as deprecated
- Update documentation
- Provide migration guide

#### Step 4: Remove Old Implementation
- Remove deprecated code
- Clean up unused dependencies
- Finalize plugin-based architecture

### Backward Compatibility

**Configuration Migration:**
```yaml
# Old tui-config.toml
nats_url = "nats://localhost:4222"
tick_rate_ms = 250
providers = []

# New configs/interface-tui.yaml (plugin-based)
modules:
  interface:
    enabled: true
    plugins:
      tui_input_handler:
        plugin_id: "tui-input-advanced"
        config:
          nats_url: "nats://localhost:4222"
          tick_rate_ms: 250
```

**Migration Script:**
```rust
pub fn migrate_config(old_config: TuiConfig) -> PluginConfig {
    PluginConfig {
        modules: {
            let mut map = HashMap::new();
            map.insert("interface".to_string(), ModuleConfig {
                enabled: true,
                plugins: ModulePlugins {
                    tui_input_handler: Some(PluginSpec {
                        plugin_id: "tui-input-advanced".to_string(),
                        config: serde_json::json!({
                            "nats_url": old_config.nats_url,
                            "tick_rate_ms": old_config.tick_rate_ms,
                        }),
                        order: 0,
                    }),
                    ..Default::default()
                },
            });
            map
        },
    }
}
```

---

## Performance Considerations

### Plugin Loading Performance
- **Lazy Loading:** Load plugins on-demand
- **Caching:** Cache plugin instances
- **Async Initialization:** Non-blocking plugin startup

### Runtime Performance
- **Zero-Cost Abstractions:** Use trait objects efficiently
- **Minimize Arc Cloning:** Use references where possible
- **Batch Updates:** Group plugin operations

### Memory Management
- **Plugin Lifecycle:** Proper shutdown and cleanup
- **Resource Limits:** Enforce memory limits per plugin
- **Leak Detection:** Monitor plugin memory usage

---

## Security Considerations

### Plugin Sandboxing
- **Configuration-Only:** No arbitrary code execution
- **Validation:** Strict configuration validation
- **Resource Limits:** CPU, memory, I/O limits per plugin

### Input Validation
- **Sanitization:** Validate all plugin inputs
- **Escaping:** Escape terminal control sequences
- **Length Limits:** Enforce input length limits

### Audit Logging
- **Plugin Actions:** Log all plugin operations
- **Configuration Changes:** Track configuration modifications
- **Error Reporting:** Secure error reporting

---

## Success Metrics

### Plugin System Metrics
- **Plugin Load Time:** < 100ms per plugin
- **Plugin Call Overhead:** < 1μs per call
- **Memory Overhead:** < 5MB per plugin
- **Plugin Count:** Support 10+ concurrent plugins

### TUI Performance Metrics
- **Frame Rate:** > 24fps with plugins
- **Input Latency:** < 10ms with plugin input
- **Startup Time:** < 500ms with all plugins
- **Memory Usage:** < 100MB total

### Developer Experience Metrics
- **Plugin Development Time:** < 1 day for simple plugins
- **Documentation Coverage:** 100% API documentation
- **Test Coverage:** > 90% for all plugins
- **Example Plugins:** 5+ example plugins

---

## Conclusion

This detailed plan extends Wireframe-AI's existing plugin architecture to support rich TUI functionality while maintaining consistency with the broader ecosystem. The configuration-driven approach provides security and simplicity, while the trait-based design enables extensibility and performance.

**Key Advantages:**
- **Consistency:** Follows existing Wireframe-AI patterns
- **Security:** No arbitrary code execution
- **Performance:** Async-first design with minimal overhead
- **Extensibility:** Rich plugin traits for TUI capabilities
- **Maintainability:** Single language stack (Rust)

**Next Steps:**
1. Review and approve this plan
2. Begin Phase 1: SDK extensions
3. Set up testing infrastructure
4. Create developer documentation
5. Implement built-in plugins
6. Integrate with existing TUI

---

*This plan will be updated based on implementation feedback and requirements.*