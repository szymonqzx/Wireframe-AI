# TUI Best Practices for Agent Chat Interfaces

**Generated:** 2025-01-07 | **Version:** 1.0 | **Target:** Wireframe-AI TUI Implementation

## Executive Summary

This comprehensive guide synthesizes research on TUI (Terminal User Interface) best practices specifically for building agent chat interfaces. Based on analysis of leading AI agent TUIs (AIChat, Sofos Code, OpenCode), modern terminal applications (k9s, lazygit, helix), and accessibility standards, this document provides actionable patterns for enhancing Wireframe-AI's TUI implementation.

## Table of Contents

1. [Core Architectural Patterns](#core-architectural-patterns)
2. [UX Patterns for Agent Conversations](#ux-patterns-for-agent-conversations)
3. [Advanced Layout and Visualization](#advanced-layout-and-visualization)
4. [Performance Optimization](#performance-optimization)
5. [Security Considerations](#security-considerations)
6. [Accessibility Patterns](#accessibility-patterns)
7. [Implementation Roadmap](#implementation-roadmap)
8. [Code Examples](#code-examples)

---

## Core Architectural Patterns

### 1. Elm Architecture (Model-View-Update)

**Why:** Provides predictable state management and testability for complex chat interfaces.

**Implementation:**
```rust
// Model - Single source of truth
struct Model {
    messages: Vec<ChatMessage>,
    input: InputBuffer,
    mode: Mode,
    nats_state: NatsState,
    theme: Theme,
}

// Update - Pure state transitions
enum Message {
    InputChar(char),
    SubmitMessage,
    NatsMessageReceived(NatsEnvelope),
    ChangeMode(Mode),
}

fn update(model: &mut Model, message: Message) -> Command<Message> {
    match message {
        Message::InputChar(c) => {
            model.input.push_char(c);
            Command::none()
        }
        Message::SubmitMessage => {
            let input = model.input.drain();
            model.messages.push(ChatMessage::user(input.clone()));
            Command::perform(submit_to_nats(input), Message::NatsSubmitted)
        }
        // ...
    }
}
```

### 2. Component-Based Architecture

**Why:** Enables reusable UI components and better separation of concerns.

**Key Components:**
- `ChatComponent` - Message display and history
- `InputComponent` - Multi-line input with vim modes
- `StatusComponent` - Live status indicators
- `PanelComponent` - Floating panels for tools/logs
- `NotificationComponent` - Toast notifications

### 3. Async Event Loop with Channels

**Why:** Prevents UI blocking and enables responsive real-time updates.

```rust
loop {
    tokio::select! {
        Some(event) = input_rx.recv() => handle_input(event),
        Some(msg) = nats_rx.recv() => handle_nats(msg),
        _ = render_rx.recv() => render_frame(),
        _ = tokio::signal::ctrl_c() => break,
    }
}
```

---

## UX Patterns for Agent Conversations

### 1. Streaming Message Display

**Problem:** Agent responses arrive incrementally, causing UI jumps.

**Solution:** Implement streaming display with cursor indicators.

```rust
struct StreamingMessage {
    role: MessageRole,
    content: String,
    is_complete: bool,
    cursor_blink: bool,
}

impl StreamingMessage {
    fn render(&self) -> Line {
        let content = if self.is_complete {
            self.content.clone()
        } else {
            format!("{}█", self.content) // Cursor indicator
        };
        
        Line::from(vec![
            Span::styled(format!("{:?}: ", self.role), Style::default().add_modifier(Modifier::BOLD)),
            Span::styled(content, Style::default().fg(Color::Green)),
        ])
    }
}
```

### 2. Message Queueing During AI Turns

**Problem:** Users cannot type while agent is thinking.

**Solution:** Queue input for immediate processing after response completes.

```rust
struct InputQueue {
    pending_input: VecDeque<String>,
    is_ai_thinking: bool,
}

impl InputQueue {
    fn enqueue(&mut self, input: String) {
        if self.is_ai_thinking {
            self.pending_input.push_back(input);
        } else {
            // Process immediately
        }
    }
    
    fn flush(&mut self) -> Vec<String> {
        self.is_ai_thinking = false;
        self.pending_input.drain(..).collect()
    }
}
```

### 3. Live Status Indicators

**Best Practice:** Real-time connection status, model info, token usage.

```rust
struct StatusLine {
    model: String,
    mode: String,
    tokens_used: usize,
    tokens_limit: usize,
    connection_status: ConnectionStatus,
    current_task: Option<String>,
}
```

### 4. Multi-line Input with Visual Indicators

**Pattern:** Shift+Enter for newlines, Enter to submit.

```rust
struct MultilineInput {
    lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
}

impl MultilineInput {
    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Enter => {
                if self.has_shift_modifier() {
                    self.insert_newline();
                } else if self.lines.len() == 1 {
                    self.submit();
                } else {
                    self.insert_newline();
                }
            }
            // ...
        }
    }
}
```

### 5. Agent State Visualization

**Pattern:** Visual indicators for agent thinking, tool execution, and waiting states.

```rust
enum AgentState {
    Idle,
    Thinking { progress: f32 },
    ExecutingTool { tool_name: String, progress: f32 },
    WaitingForUser,
    Error { message: String },
}
```

---

## Advanced Layout and Visualization

### 1. Dynamic Split View System

**Inspiration:** k9s adaptive layout system

**Features:**
- Adaptive layout based on screen size
- Configurable panel widths
- Support for multiple layout modes

```rust
enum LayoutMode {
    ChatOnly,
    ChatWithLeftPanel,    // Context/files
    ChatWithRightPanel,   // Agent status/tools
    ChatWithBothPanels,
    SplitView,            // Top/bottom split for code + chat
    OverlayMode,          // Full-screen overlay
}
```

### 2. Tab System for Multiple Contexts

**Inspiration:** helix tab management

**Features:**
- Multiple agent sessions
- Keyboard navigation (Ctrl+Tab)
- Session persistence

### 3. Floating Panel System

**Inspiration:** lazygit floating panels

**Panel Types:**
- Tool execution
- Agent logs
- NATS flow visualization
- Schema validator
- Inspector
- Help

### 4. Tool Execution Visualization

**Pattern:** Progress bars, timelines, and execution trees

```rust
struct ToolExecution {
    pub tool_id: String,
    pub tool_name: String,
    pub status: ToolStatus,
    pub progress: u16,
    pub output: String,
    pub error: Option<String>,
}
```

---

## Performance Optimization

### 1. Incremental Rendering

**Problem:** Full re-render on every message causes lag.

**Solution:** Hash-based diff detection to avoid unnecessary redraws.

```rust
struct DiffRenderer {
    last_render: RenderCache,
}

struct RenderCache {
    messages_hash: u64,
    input_hash: u64,
    status_hash: u64,
}

impl DiffRenderer {
    fn should_rerender(&self, new_state: &RenderState) -> bool {
        self.last_render.messages_hash != hash_messages(&new_state.messages)
            || self.last_render.input_hash != hash(&new_state.input)
            || self.last_render.status_hash != hash_status(&new_state)
    }
}
```

### 2. Frame Rate Control

**Best Practice:** Target ~24fps for smooth UI without excessive CPU usage.

```rust
let (render_tx, render_rx) = mpsc::channel::<RenderRequest>(60); // ~24fps
```

### 3. Memory Management

**Patterns:**
- Message compaction for long conversations
- Circular buffers for message history
- Lazy loading of message content

### 4. Async Patterns

**Best Practices:**
- Bounded channels for backpressure
- Separate channels for different concerns
- Non-blocking I/O operations

---

## Security Considerations

### 1. Input Validation and Sanitization

**Critical:** Validate all user input to prevent injection attacks.

```rust
fn sanitize_input(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_ascii_graphic() || *c == ' ')
        .collect()
}
```

### 2. Secure Credential Handling

**Best Practice:** Never log or display sensitive information.

```rust
struct SecureConfig {
    pub api_key: SecretString,
    pub endpoint: String,
}

impl std::fmt::Debug for SecureConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecureConfig")
            .field("endpoint", &self.endpoint)
            .field("api_key", &"[REDACTED]")
            .finish()
    }
}
```

### 3. Terminal Injection Protection

**Pattern:** Escape control sequences in user content.

```rust
fn escape_control_sequences(text: &str) -> String {
    text.replace('\x1b', "\\x1b")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
}
```

### 4. Audit Logging

**Requirement:** Log all agent interactions for security monitoring.

```rust
#[derive(Debug, Serialize)]
struct AuditLog {
    timestamp: chrono::DateTime<chrono::Utc>,
    user_id: String,
    session_id: String,
    action: String,
    agent_response: String,
    tools_used: Vec<String>,
}
```

---

## Accessibility Patterns

### 1. Screen Reader Compatibility

**Best Practices:**
- Use semantic markup for content structure
- Provide alternative text for visual elements
- Ensure keyboard-only navigation

```rust
struct AccessibleMessage {
    content: String,
    role: MessageRole,
    alt_text: String,
}

impl AccessibleMessage {
    fn to_screen_reader_text(&self) -> String {
        format!("{}: {}", self.role, self.alt_text)
    }
}
```

### 2. High Contrast Themes

**Pattern:** Provide high-contrast color schemes for visually impaired users.

```rust
struct HighContrastTheme {
    background: Color::Black,
    foreground: Color::White,
    accent: Color::Yellow,
    success: Color::Green,
    error: Color::Red,
}
```

### 3. Keyboard Navigation

**Requirements:**
- Tab navigation between UI elements
- Clear focus indicators
- Shortcut keys for common actions

### 4. Color Blindness Support

**Pattern:** Use patterns and icons in addition to colors.

```rust
fn status_indicator(status: &Status) -> &'static str {
    match status {
        Status::Success => "✓",
        Status::Warning => "⚠",
        Status::Error => "✗",
        Status::Info => "ℹ",
    }
}
```

---

## Implementation Roadmap

### Phase 1: Core Enhancements (Week 1-2)

1. **Adopt Elm Architecture** in minimal TUI
2. **Implement streaming display** with incremental rendering
3. **Add message queueing** for typing during AI turns
4. **Create component traits** for better organization

### Phase 2: UX Improvements (Week 3-4)

1. **Add session persistence** with auto-save/resume
2. **Implement live status indicators** for agent state
3. **Support multi-line input** with Shift+Enter pattern
4. **Add command palette** for unified command system

### Phase 3: Advanced Features (Week 5-6)

1. **Dynamic layout system** with adaptive panels
2. **Tab system** for multiple sessions
3. **Floating panels** for tools and logs
4. **Notification system** with toast notifications

### Phase 4: Performance & Security (Week 7-8)

1. **Incremental rendering** optimization
2. **Memory management** improvements
3. **Security hardening** and audit logging
4. **Accessibility enhancements**

### Phase 5: Plugin System (Week 9-10)

1. **Lua plugin architecture** for extensibility
2. **Widget registration** system
3. **Theme system** with accessibility support
4. **Documentation and examples**

---

## Code Examples

### Enhanced Input Handler

```rust
// tools/tui-minimal/tui-input/src/lib.rs
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub struct EnhancedInputBuffer {
    lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
    mode: InputMode,
    history: Vec<String>,
    history_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Insert,
    Visual,
    Command,
}

impl EnhancedInputBuffer {
    pub fn handle_key(&mut self, key: KeyEvent) -> InputEvent {
        match self.mode {
            InputMode::Normal => self.handle_normal_mode(key),
            InputMode::Insert => self.handle_insert_mode(key),
            InputMode::Visual => self.handle_visual_mode(key),
            InputMode::Command => self.handle_command_mode(key),
        }
    }
    
    fn handle_insert_mode(&mut self, key: KeyEvent) -> InputEvent {
        match key.code {
            KeyCode::Char(c) => {
                self.insert_char(c);
                InputEvent::InputChanged
            }
            KeyCode::Enter => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    self.insert_newline();
                    InputEvent::InputChanged
                } else if self.lines.len() == 1 {
                    let content = self.lines[0].clone();
                    self.clear();
                    InputEvent::Submit(content)
                } else {
                    self.insert_newline();
                    InputEvent::InputChanged
                }
            }
            KeyCode::Backspace => {
                self.delete_char();
                InputEvent::InputChanged
            }
            KeyCode::Esc => {
                self.mode = InputMode::Normal;
                InputEvent::ModeChanged(InputMode::Normal)
            }
            _ => InputEvent::Unknown,
        }
    }
}
```

### Status Bar Component

```rust
// tools/tui-minimal/tui-status/src/lib.rs
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

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
            label: "Tokens".to_string(),
            value: format!("{}/{}", state.tokens_used, state.token_limit),
            color: if state.tokens_used > state.token_limit * 9 / 10 { Color::Red } else { Color::Green },
            icon: '📊',
        });
        
        Self { items }
    }
    
    pub fn render(&self, area: Rect, frame: &mut ratatui::Frame) {
        let line: Line = self.items
            .iter()
            .flat_map(|item| {
                vec![
                    Span::styled(format!("{} ", item.icon), Style::default().fg(item.color)),
                    Span::styled(&item.label, Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(": "),
                    Span::styled(&item.value, Style::default().fg(item.color)),
                    Span::raw(" | "),
                ]
            })
            .collect();
        
        let paragraph = Paragraph::new(line)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White));
        
        frame.render_widget(paragraph, area);
    }
}
```

### Notification Manager

```rust
// tools/tui-minimal/tui-notification/src/lib.rs
use std::time::{Duration, Instant};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Paragraph};

#[derive(Debug, Clone)]
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
    pub created_at: Instant,
    pub duration: Duration,
}

pub struct NotificationManager {
    notifications: Vec<Notification>,
    max_visible: usize,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: Vec::new(),
            max_visible: 5,
        }
    }
    
    pub fn add(&mut self, level: NotificationLevel, title: String, message: String) {
        let notification = Notification {
            id: uuid::Uuid::new_v4().to_string(),
            level,
            title,
            message,
            created_at: Instant::now(),
            duration: Duration::from_secs(5),
        };
        self.notifications.push(notification);
    }
    
    pub fn render(&self, area: Rect, frame: &mut ratatui::Frame) {
        let notifications = self.visible_notifications();
        
        for (i, notif) in notifications.iter().enumerate() {
            let color = match notif.level {
                NotificationLevel::Info => Color::Blue,
                NotificationLevel::Success => Color::Green,
                NotificationLevel::Warning => Color::Yellow,
                NotificationLevel::Error => Color::Red,
            };
            
            let height = 3;
            let width = 50.min(area.width);
            let y = area.height - height - (i as u16 * (height + 1)) - 1;
            let x = area.width - width - 1;
            
            let notification_area = Rect { x, y, width, height };
            
            let paragraph = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(&notif.title, Style::default().add_modifier(Modifier::BOLD).fg(color)),
                ]),
                Line::from(&notif.message),
            ])
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(color)))
            .alignment(Alignment::Left);
            
            frame.render_widget(paragraph, notification_area);
        }
    }
}
```

---

## Dependencies and Integration

### Required Dependencies

```toml
# Add to workspace Cargo.toml
nucleo = "0.5"           # Fuzzy matching for autocompletion
mlua = "0.9"             # Lua plugin support
libloading = "0.8"       # Dynamic plugin loading
unicode-width = "0.1"    # Unicode width calculation
base64 = "0.22"          # Graphics encoding
dyn-clone = "1.0"        # Trait object cloning
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

### New Crate Structure

```
tools/tui-minimal/
├── tui-main/          # Entry point (existing)
├── tui-core/          # Application state (existing)
├── tui-config/        # Configuration (existing)
├── tui-nats/          # NATS integration (existing)
├── tui-input/         # Enhanced input handling
├── tui-render/        # Enhanced rendering (existing)
├── tui-layout/        # Dynamic layout system (new)
├── tui-tabs/          # Tab management (new)
├── tui-panels/        # Floating panels (new)
├── tui-status/        # Status bar (new)
├── tui-notification/  # Notifications (new)
├── tui-visualization/ # Agent/tool visualization (new)
├── tui-autocomplete/  # Autocompletion engine (new)
├── tui-palette/       # Command palette (new)
├── tui-plugin/        # Plugin system (new)
├── tui-widget/        # Widget registry (new)
├── tui-graphics/      # Terminal graphics (new)
├── tui-platform/      # Platform detection (new)
├── tui-clipboard/     # Clipboard integration (new)
├── tui-accessibility/ # Accessibility features (new)
└── tui-theme/         # Theme system (new)
```

### Integration Steps

1. **Update workspace Cargo.toml** with new crates and dependencies
2. **Implement core patterns** (Elm architecture, components)
3. **Enhance existing modules** with new features
4. **Add new modules** following the established patterns
5. **Integrate with NATS** for real-time features
6. **Add comprehensive tests** for all components
7. **Update documentation** and examples

---

## Testing Strategy

### Unit Tests

- Test individual components in isolation
- Mock NATS connections for reliable testing
- Test state transitions and edge cases
- Verify accessibility features

### Integration Tests

- Test component interactions
- Test NATS message flow
- Test plugin loading and execution
- Test cross-platform compatibility

### Performance Tests

- Benchmark rendering performance
- Test memory usage with large chat histories
- Test concurrent operations
- Profile and optimize bottlenecks

### Accessibility Tests

- Test screen reader compatibility
- Verify keyboard navigation
- Test high contrast themes
- Validate color blindness support

---

## Conclusion

This comprehensive guide provides a roadmap for transforming Wireframe-AI's minimal TUI into a sophisticated, accessible, and performant agent chat interface. By adopting these patterns and best practices, Wireframe-AI can deliver a terminal experience that rivals modern GUI applications while maintaining the efficiency and power of command-line interfaces.

The implementation roadmap prioritizes user experience improvements while ensuring security, performance, and accessibility considerations are addressed from the beginning. The modular architecture allows for incremental development and testing, reducing risk and ensuring quality at each stage.

**Next Steps:**
1. Review and approve this roadmap
2. Begin Phase 1 implementation
3. Establish testing infrastructure
4. Set up CI/CD for automated testing
5. Create developer documentation and examples

---

*This document will be updated as new patterns emerge and the implementation progresses. Contributions and feedback are welcome.*