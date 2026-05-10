//! Input handling for minimal TUI
//!
//! Handles keyboard input and event processing

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;
use tui_core::PluginEvent;

/// Extended Input event that includes PluginEvent mapping
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    Plugin(PluginEvent),
    /// Ctrl+C (quit)
    CtrlC,
    /// Ctrl+Q (quit)
    CtrlQ,
    /// Unknown key
    Unknown,
}

impl InputEvent {
    pub fn is_quit(&self) -> bool {
        matches!(self, InputEvent::CtrlC | InputEvent::CtrlQ)
    }
}

/// Input handler
pub struct InputHandler {
    event_sender: mpsc::UnboundedSender<InputEvent>,
    event_receiver: Option<mpsc::UnboundedReceiver<InputEvent>>,
}

impl InputHandler {
    /// Create new input handler
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        Self {
            event_sender,
            event_receiver: Some(event_receiver),
        }
    }

    /// Get event sender
    pub fn sender(&self) -> mpsc::UnboundedSender<InputEvent> {
        self.event_sender.clone()
    }

    /// Take the event receiver
    pub fn take_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<InputEvent>> {
        self.event_receiver.take()
    }

    /// Start reading input events
    pub fn run(&mut self) -> Result<()> {
        loop {
            if event::poll(std::time::Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? {
                    let event = self.map_key_event(key);
                    if event.is_quit() {
                        // Send quit event and break
                        let _ = self.event_sender.send(event);
                        break;
                    }
                    let _ = self.event_sender.send(event);
                }
            }
        }
        Ok(())
    }

    /// Map crossterm key event to input event
    pub fn map_key_event(&self, key: KeyEvent) -> InputEvent {
        match key.code {
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'c' => InputEvent::CtrlC,
                        'q' => InputEvent::CtrlQ,
                        _ => InputEvent::Plugin(PluginEvent::Ctrl(c)),
                    }
                } else {
                    InputEvent::Plugin(PluginEvent::Input(c))
                }
            }
            KeyCode::Enter => InputEvent::Plugin(PluginEvent::Enter),
            KeyCode::Backspace => InputEvent::Plugin(PluginEvent::Backspace),
            KeyCode::Esc => InputEvent::Plugin(PluginEvent::Esc),
            KeyCode::Up => InputEvent::Plugin(PluginEvent::Up),
            KeyCode::Down => InputEvent::Plugin(PluginEvent::Down),
            KeyCode::Left => InputEvent::Plugin(PluginEvent::Left),
            KeyCode::Right => InputEvent::Plugin(PluginEvent::Right),
            _ => InputEvent::Unknown,
        }
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Input buffer for multi-line editing
#[derive(Debug, Clone, Default)]
pub struct InputBuffer {
    buffer: String,
}

impl InputBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get current buffer content
    pub fn as_str(&self) -> &str {
        &self.buffer
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Add character to buffer
    pub fn push_char(&mut self, c: char) {
        self.buffer.push(c);
    }

    /// Add a newline to the buffer
    pub fn push_newline(&mut self) {
        self.buffer.push('\n');
    }

    /// Remove last character
    pub fn pop_char(&mut self) -> Option<char> {
        self.buffer.pop()
    }

    /// Clear buffer
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    /// Get buffer length
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_buffer() {
        let mut buffer = InputBuffer::new();
        assert!(buffer.is_empty());

        buffer.push_char('a');
        buffer.push_char('b');
        assert_eq!(buffer.as_str(), "ab");
        assert_eq!(buffer.len(), 2);

        buffer.pop_char();
        assert_eq!(buffer.as_str(), "a");

        buffer.clear();
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_key_mapping() {
        let handler = InputHandler::new();

        // Test character
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::empty());
        let event = handler.map_key_event(key);
        assert_eq!(event, InputEvent::Plugin(PluginEvent::Input('a')));

        // Test Ctrl+C
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let event = handler.map_key_event(key);
        assert_eq!(event, InputEvent::CtrlC);

        // Test Enter
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        let event = handler.map_key_event(key);
        assert_eq!(event, InputEvent::Plugin(PluginEvent::Enter));
    }
}
