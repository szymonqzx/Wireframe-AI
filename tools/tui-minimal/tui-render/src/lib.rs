//! UI rendering for minimal TUI
//!
//! Handles terminal rendering and layout

use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::sync::Arc;
use tokio::sync::RwLock;
use tui_core::{PluginManager, RenderContext};

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
    pub sidebar_items: Vec<String>,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            messages: vec![],
            input: String::new(),
            nats_connected: false,
            pending_tasks: 0,
            sidebar_items: vec!["Recent Chat 1".to_string(), "Recent Chat 2".to_string()],
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
    pub async fn render(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        state: RenderState,
        plugin_manager: Arc<RwLock<PluginManager>>,
    ) -> Result<()> {
        let pm = plugin_manager.read().await;
        terminal.draw(|f| {
            Self::draw_ui(f, &state);

            // Allow plugins to render overlays
            let area = f.size();
            let mut ctx = RenderContext { area, frame: f };
            let _ = pm.render_all(&mut ctx);
        })?;
        Ok(())
    }

    /// Draw the UI
    fn draw_ui(f: &mut Frame, state: &RenderState) {
        let size = f.size();

        // OpenCode layout: Sidebar on left, Main area on right
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20), // Sidebar
                Constraint::Percentage(80), // Main area
            ])
            .split(size);

        // Render Sidebar
        let sidebar_block = Block::default()
            .title("History")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let sidebar_text: Vec<Line> = state
            .sidebar_items
            .iter()
            .map(|item| Line::from(item.as_str()))
            .collect();

        let sidebar_paragraph = Paragraph::new(sidebar_text).block(sidebar_block);

        f.render_widget(sidebar_paragraph, chunks[0]);

        // Main area: Messages on top, Input in middle, Status at bottom
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Messages area
                Constraint::Length(4), // Input area
                Constraint::Length(1), // Status bar
            ])
            .split(chunks[1]);

        // Draw messages area
        let messages_block = Block::default()
            .title("Wireframe-AI")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        f.render_widget(messages_block, main_chunks[0]);

        let messages_area = main_chunks[0].inner(&Margin {
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
                        Style::default()
                            .add_modifier(Modifier::BOLD)
                            .fg(Color::DarkGray),
                    ),
                    Span::styled(&msg.content, role_style),
                ])
            })
            .collect();

        let messages_paragraph = Paragraph::new(messages_text)
            .wrap(Wrap { trim: true })
            .scroll((state.messages.len() as u16, 0)); // Auto-scroll to bottom

        f.render_widget(messages_paragraph, messages_area);

        // Draw input area
        let input_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title("Input");

        f.render_widget(input_block, main_chunks[1]);

        let input_area = main_chunks[1].inner(&Margin {
            horizontal: 1,
            vertical: 1,
        });

        let input_paragraph = Paragraph::new(state.input.as_str());
        f.render_widget(input_paragraph, input_area);

        // Draw status bar
        let status_style = Style::default().bg(Color::DarkGray).fg(Color::White);
        let status_text = if state.nats_connected {
            format!(
                " Connected | {} pending | [Ctrl+P] Command Palette | [Ctrl+C] Quit",
                state.pending_tasks
            )
        } else {
            " Disconnected | [Ctrl+P] Command Palette | [Ctrl+C] Quit".to_string()
        };

        let status_paragraph = Paragraph::new(status_text).style(status_style);

        f.render_widget(status_paragraph, main_chunks[2]);
    }

    /// Helper to center a rect
    pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
