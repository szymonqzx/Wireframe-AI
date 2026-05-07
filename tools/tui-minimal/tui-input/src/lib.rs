//! Input handling for minimal TUI
//! 
//! Handles keyboard input and event processing

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

/// Input event
#[derive(Debug, Clone, PartialEq)]
pub enum InputEvent {
    /// Character input
    Char(char),
    /// Enter key
    Enter,
    /// Backspace
    Backspace,
    /// Ctrl+C (quit)
    CtrlC,
    /// Ctrl+Q (quit)
    CtrlQ,
    /// Unknown key
    Unknown,
}

/// Input handler
pub struct InputHandler {
    event_sender: mpsc::UnboundedSender<InputEvent>,
}

impl InputHandler {
    /// Create new input handler
    pub fn new() -> Self {
        let (event_sender, _event_receiver) = mpsc::unbounded_channel();
        Self { event_sender }
    }
    
    /// Get event sender
    pub fn sender(&self) -> mpsc::UnboundedSender<InputEvent> {
        self.event_sender.clone()
    }
    
    /// Start reading input events
    pub async fn run(&self) -> Result<()> {
        loop {
            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    let event = self.map_key_event(key);
                    if event == InputEvent::CtrlC || event == InputEvent::CtrlQ {
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
    fn map_key_event(&self, key: KeyEvent) -> InputEvent {
        match key.code {
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'c' => InputEvent::CtrlC,
                        'q' => InputEvent::CtrlQ,
                        _ => InputEvent::Unknown,
                    }
                } else {
                    InputEvent::Char(c)
                }
            }
            KeyCode::Enter => InputEvent::Enter,
            KeyCode::Backspace => InputEvent::Backspace,
            KeyCode::Esc => InputEvent::CtrlQ, // ESC also quits
            _ => InputEvent::Unknown,
        }
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Input buffer for line editing
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
        assert_eq!(event, InputEvent::Char('a'));
        
        // Test Ctrl+C
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        let event = handler.map_key_event(key);
        assert_eq!(event, InputEvent::CtrlC);
        
        // Test Enter
        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        let event = handler.map_key_event(key);
        assert_eq!(event, InputEvent::Enter);
    }
}
