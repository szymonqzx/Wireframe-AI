# TUI Implementation Plan: Prioritized Opportunities

**Generated:** 2025-01-07 | **Focus:** Wireframe-AI TUI Enhancement

## Executive Summary

Based on comprehensive research of TUI best practices and Wireframe-AI's current architecture, this plan prioritizes the most impactful improvements for the agent chat interface. The plan evaluates implementation complexity, user value, and technical feasibility.

## Priority Analysis

### 1. **HIGH PRIORITY: Message Queueing During AI Turns**
**Impact:** ⭐⭐⭐⭐⭐ | **Complexity:** ⭐⭐ | **Timeline:** 1-2 days

**Problem:** Users cannot type while agent is thinking, creating frustrating wait times.

**Solution:** Implement input queueing system that allows users to continue typing during AI responses.

```rust
// Add to tui-input/src/lib.rs
use std::collections::VecDeque;

pub struct InputQueue {
    pending_input: VecDeque<String>,
    is_ai_thinking: bool,
    current_input: String,
}

impl InputQueue {
    pub fn enqueue(&mut self, input: String) {
        if self.is_ai_thinking {
            self.pending_input.push_back(input);
        } else {
            self.current_input = input;
        }
    }
    
    pub fn flush(&mut self) -> Vec<String> {
        self.is_ai_thinking = false;
        let queued = self.pending_input.drain(..).collect();
        self.current_input.clear();
        queued
    }
}
```

**Implementation Steps:**
1. Add `InputQueue` struct to `tui-input`
2. Modify main event loop to handle AI state
3. Update UI to show queued message count
4. Test with concurrent input scenarios

**Why First:** Immediate user experience improvement with minimal code changes.

---

### 2. **HIGH PRIORITY: Live Status Indicators**
**Impact:** ⭐⭐⭐⭐ | **Complexity:** ⭐⭐⭐ | **Timeline:** 2-3 days

**Problem:** Users have no visibility into agent state, connection status, or progress.

**Solution:** Real-time status bar showing connection, agent state, and activity.

```rust
// Add to tui-status/src/lib.rs
pub struct StatusBar {
    items: Vec<StatusItem>,
}

#[derive(Debug, Clone)]
pub struct StatusItem {
    pub label: String,
    pub value: String,
    pub color: Color,
    pub icon: char,
}

impl StatusBar {
    pub fn from_app_state(state: &AppState) -> Self {
        let mut items = Vec::new();
        
        items.push(StatusItem {
            label: "NATS".to_string(),
            value: if state.is_nats_connected() { "Connected" } else { "Disconnected" }.to_string(),
            color: if state.is_nats_connected() { Color::Green } else { Color::Red },
            icon: if state.is_nats_connected() { '●' } else { '○' },
        });
        
        items.push(StatusItem {
            label: "Agent".to_string(),
            value: state.current_agent.clone(),
            color: Color::Cyan,
            icon: '🤖',
        });
        
        items.push(StatusItem {
            label: "Pending".to_string(),
            value: state.pending_tasks.len().to_string(),
            color: Color::Yellow,
            icon: '⏳',
        });
        
        Self { items }
    }
}
```

**Implementation Steps:**
1. Create `tui-status` crate
2. Add status bar rendering to `tui-render`
3. Integrate with NATS connection state
4. Add agent state tracking
5. Update layout to accommodate status bar

**Why Second:** Provides essential visibility without major architectural changes.

---

### 3. **MEDIUM PRIORITY: Session Persistence**
**Impact:** ⭐⭐⭐⭐ | **Complexity:** ⭐⭐⭐ | **Timeline:** 3-4 days

**Problem:** Chat sessions are lost when TUI restarts, forcing users to start over.

**Solution:** Auto-save chat history to disk with resume functionality.

```rust
// Add to tui-core/src/lib.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub messages: Vec<ChatMessage>,
    pub agent: String,
    pub metadata: SessionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub title: String,
    pub tags: Vec<String>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

impl ChatSession {
    pub fn save_to_disk(&self, path: &PathBuf) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    pub fn load_from_disk(path: &PathBuf) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json)?)
    }
}
```

**Implementation Steps:**
1. Add session serialization to `tui-core`
2. Create session storage directory structure
3. Implement auto-save on message changes
4. Add session picker on startup
5. Add session management commands

**Why Third:** Significant user value but requires more architectural changes.

---

### 4. **LOW PRIORITY: Plugin Architecture (NOT Lua)**
**Impact:** ⭐⭐⭐ | **Complexity:** ⭐⭐⭐⭐⭐ | **Timeline:** 2-3 weeks

**Problem:** TUI is not extensible for custom functionality.

**Solution:** Configuration-driven plugin system using Rust trait objects instead of Lua.

### Why NOT Lua for Wireframe-AI

**Lua Issues:**
- **Security Risk:** Arbitrary code execution in terminal context
- **Complexity:** Adding mlua dependency (~2MB) for limited benefit
- **Debugging:** Mixed Rust/Lua stack traces are hard to debug
- **Performance:** Lua interpreter overhead for simple operations
- **Maintenance:** Lua scripts require separate testing and documentation

### Better Alternative: Configuration-Driven Plugins

```rust
// Add to tui-plugin/src/lib.rs
use dyn_clone::DynClone;
use serde_json::Value;

pub trait Plugin: DynClone + Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self, context: &PluginContext) -> Result<()>;
    fn on_message(&mut self, message: &ChatMessage) -> Result<Option<ChatMessage>>;
    fn on_command(&mut self, command: &str, args: &Value) -> Result<Value>;
    fn render_widget(&self, area: Rect, buf: &mut Buffer) -> Result<()>;
    fn shutdown(&mut self) -> Result<()>;
}

dyn_clone::clone_trait_object!(Plugin);

// Built-in plugins (no external dependencies)
pub struct MessageLoggerPlugin {
    log_file: PathBuf,
}

impl Plugin for MessageLoggerPlugin {
    fn name(&self) -> &str { "message-logger" }
    
    fn on_message(&mut self, message: &ChatMessage) -> Result<Option<ChatMessage>> {
        let log_entry = format!("{}: {:?} - {}\n", 
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            message.role,
            message.content
        );
        std::fs::write(&self.log_file, log_entry)?;
        Ok(None) // Don't modify message
    }
}

pub struct AutoSavePlugin {
    session_dir: PathBuf,
    save_interval: Duration,
}

pub struct ThemePlugin {
    custom_colors: HashMap<String, Color>,
}

pub struct KeyboardShortcutsPlugin {
    custom_bindings: HashMap<String, String>,
}
```

**Configuration-Driven Approach:**
```yaml
# tui-plugins.yaml
plugins:
  - name: message-logger
    enabled: true
    config:
      log_file: "~/.wireframe-tui/messages.log"
  
  - name: auto-save
    enabled: true
    config:
      session_dir: "~/.wireframe-tui/sessions"
      save_interval: 30s
  
  - name: theme
    enabled: false
    config:
      user_color: "#00ffff"
      assistant_color: "#00ff00"
      system_color: "#ffff00"
  
  - name: keyboard-shortcuts
    enabled: true
    config:
      "Ctrl+R": "reload-config"
      "Ctrl+S": "save-session"
      "F9": "toggle-panels"
```

**Benefits of Configuration-Driven:**
- **Secure:** No arbitrary code execution
- **Simple:** YAML configuration is easy to understand
- **Fast:** Rust-native performance
- **Maintainable:** Single language stack
- **Testable:** Standard Rust testing patterns

**Implementation Steps:**
1. Define plugin trait in `tui-plugin`
2. Implement built-in plugins for common features
3. Add plugin configuration loading
4. Create plugin registry and lifecycle management
5. Integrate plugins into main event loop

**Why Last:** High complexity for moderate benefit. Configuration-driven approach is safer and simpler than Lua.

---

## Implementation Roadmap

### Week 1: Foundation (Days 1-5)
- **Day 1-2:** Message queueing implementation
- **Day 3-4:** Live status indicators
- **Day 5:** Testing and integration

### Week 2: Enhancement (Days 6-10)
- **Day 6-8:** Session persistence
- **Day 9-10:** Bug fixes and polish

### Week 3-4: Plugin System (Days 11-20)
- **Day 11-14:** Plugin architecture design
- **Day 15-17:** Built-in plugins implementation
- **Day 18-20:** Configuration system and testing

## Technical Dependencies

### Required Cargo Additions
```toml
# In workspace Cargo.toml
chrono = { version = "0.4", features = ["serde"] }
serde_json = "1.0"
dyn-clone = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
```

### New Crate Structure
```
tools/tui-minimal/
├── tui-main/          # Entry point (existing)
├── tui-core/          # Application state (enhanced)
├── tui-config/        # Configuration (existing)
├── tui-nats/          # NATS integration (existing)
├── tui-input/         # Enhanced input with queueing
├── tui-render/        # Enhanced rendering (existing)
├── tui-status/        # Status bar (new)
├── tui-session/      # Session persistence (new)
└── tui-plugin/        # Plugin system (new)
```

## Success Metrics

### User Experience
- **Response time:** < 100ms for all UI interactions
- **Queue efficiency:** Zero lost messages during AI turns
- **Session recovery:** < 1 second to resume previous session
- **Status visibility:** Real-time updates for all system states

### Technical
- **Memory usage:** < 50MB for typical sessions
- **Startup time:** < 500ms to ready state
- **Plugin load time:** < 100ms per plugin
- **Configuration reload:** < 200ms

### Code Quality
- **Test coverage:** > 90% for new features
- **Documentation:** Complete API docs for all public interfaces
- **Error handling:** Graceful degradation for all failure modes

## Risk Assessment

### Low Risk
- Message queueing: Simple data structure changes
- Status indicators: UI-only changes
- Session persistence: File I/O operations

### Medium Risk
- Plugin architecture: Requires careful API design
- Configuration system: Needs backward compatibility

### Mitigation Strategies
- Incremental rollout with feature flags
- Comprehensive testing before each release
- Fallback mechanisms for all new features
- User documentation and migration guides

## Conclusion

This implementation plan prioritizes user experience improvements while maintaining Wireframe-AI's architectural principles. The phased approach allows for incremental value delivery with minimal risk.

**Key Decision:** Avoid Lua in favor of configuration-driven plugins for better security, performance, and maintainability in the terminal context.

**Next Steps:**
1. Review and approve this plan
2. Begin message queueing implementation
3. Set up testing infrastructure
4. Create developer documentation

---

*This plan will be updated based on implementation feedback and user requirements.*