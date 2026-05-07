//! UI rendering for minimal TUI
//! 
//! Handles terminal rendering and layout

use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Chat message
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Application state for rendering
#[derive(Debug, Clone)]
pub struct RenderState {
    pub messages: Vec<ChatMessage>,
    pub input: String,
    pub nats_connected: bool,
    pub pending_tasks: usize,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            messages: vec![],
            input: String::new(),
            nats_connected: false,
            pending_tasks: 0,
        }
    }
}

/// UI renderer
pub struct Renderer {
    state: Arc<RwLock<RenderState>>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(RenderState::default())),
        }
    }
    
    /// Get state
    pub fn state(&self) -> Arc<RwLock<RenderState>> {
        self.state.clone()
    }
    
    /// Render the UI
    pub async fn render(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, state: RenderState) -> Result<()> {
        terminal.draw(|f| Self::draw_ui(f, state))?;
        Ok(())
    }
    
    /// Draw the UI
    fn draw_ui(f: &mut Frame, state: RenderState) {
        let size = f.area();
        
        // Create layout: messages area on top, input area at bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Min(0), // Messages area
                Constraint::Length(3), // Input area
            ])
            .split(size);
        
        // Draw messages area
        let messages_block = Block::default()
            .title("Wireframe-AI Minimal TUI")
            .borders(Borders::ALL);
        
        f.render_widget(messages_block, chunks[0]);
        
        let messages_area = chunks[0].inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        
        // Render messages
        let messages_text: Vec<Line> = state
            .messages
            .iter()
            .map(|msg| {
                let role_style = match msg.role {
                    MessageRole::User => Style::default().fg(Color::Cyan),
                    MessageRole::Assistant => Style::default().fg(Color::Green),
                    MessageRole::System => Style::default().fg(Color::Yellow),
                };
                
                Line::from(vec![
                    Span::styled(
                        format!("{:?}: ", msg.role),
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(&msg.content, role_style),
                ])
            })
            .collect();
        
        let messages_paragraph = Paragraph::new(messages_text)
            .wrap(Wrap { trim: true })
            .scroll((0, state.messages.len() as u16)); // Auto-scroll to bottom
        
        f.render_widget(messages_paragraph, messages_area);
        
        // Draw input area
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title(if state.nats_connected {
                format!("Input (Connected, {} pending)", state.pending_tasks)
            } else {
                "Input (Disconnected)".to_string()
            });
        
        f.render_widget(input_block, chunks[1]);
        
        let input_area = chunks[1].inner(Margin {
            horizontal: 1,
            vertical: 1,
        });
        
        let input_paragraph = Paragraph::new(state.input.as_str());
        f.render_widget(input_paragraph, input_area);
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_render_state_default() {
        let state = RenderState::default();
        assert!(state.messages.is_empty());
        assert!(state.input.is_empty());
        assert!(!state.nats_connected);
    }
    
    #[test]
    fn test_chat_message() {
        let msg = ChatMessage {
            role: MessageRole::User,
            content: "test".to_string(),
        };
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "test");
    }
}
