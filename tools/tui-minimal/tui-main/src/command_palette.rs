use anyhow::Result;
use async_trait::async_trait;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};
use tui_core::{Plugin, PluginEvent, RenderContext};
use tui_render::Renderer;

pub struct CommandPalettePlugin {
    pub active: bool,
    pub input: String,
    pub commands: Vec<String>,
    pub selected_index: usize,
}

impl CommandPalettePlugin {
    pub fn new() -> Self {
        Self {
            active: false,
            input: String::new(),
            commands: vec![
                "Connect to NATS".to_string(),
                "Disconnect NATS".to_string(),
                "Clear History".to_string(),
                "Quit".to_string(),
            ],
            selected_index: 0,
        }
    }

    fn filtered_commands(&self) -> Vec<&String> {
        if self.input.is_empty() {
            self.commands.iter().collect()
        } else {
            self.commands
                .iter()
                .filter(|c| c.to_lowercase().contains(&self.input.to_lowercase()))
                .collect()
        }
    }
}

#[async_trait]
impl Plugin for CommandPalettePlugin {
    fn name(&self) -> &'static str {
        "CommandPalette"
    }

    async fn handle_event(&mut self, event: &PluginEvent) -> Result<bool> {
        // Toggle palette
        if let PluginEvent::Ctrl('p') = event {
            self.active = !self.active;
            if !self.active {
                self.input.clear();
                self.selected_index = 0;
            }
            return Ok(true); // Consume event
        }

        if !self.active {
            return Ok(false); // Let other plugins handle it
        }

        match event {
            PluginEvent::Esc => {
                self.active = false;
                self.input.clear();
                self.selected_index = 0;
                Ok(true)
            }
            PluginEvent::Input(c) => {
                self.input.push(*c);
                self.selected_index = 0;
                Ok(true)
            }
            PluginEvent::Backspace => {
                self.input.pop();
                self.selected_index = 0;
                Ok(true)
            }
            PluginEvent::Up => {
                let count = self.filtered_commands().len();
                if count > 0 {
                    self.selected_index = self.selected_index.saturating_sub(1);
                }
                Ok(true)
            }
            PluginEvent::Down => {
                let count = self.filtered_commands().len();
                if count > 0 && self.selected_index < count - 1 {
                    self.selected_index += 1;
                }
                Ok(true)
            }
            PluginEvent::Enter => {
                let filtered = self.filtered_commands();
                if let Some(_cmd) = filtered.get(self.selected_index) {
                    // Here we'd execute the command, for now just close
                    // In a real app we might pass a channel to send commands to the main loop
                    self.active = false;
                    self.input.clear();
                    self.selected_index = 0;
                }
                Ok(true)
            }
            _ => Ok(true), // Consume all other events while active so they don't leak
        }
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<()> {
        if !self.active {
            return Ok(());
        }

        let area = Renderer::centered_rect(60, 40, ctx.area);

        // Clear background
        ctx.frame.render_widget(Clear, area);

        let block = Block::default()
            .title("Command Palette")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let inner_area = block.inner(area);
        ctx.frame.render_widget(block, area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(inner_area);

        // Input
        let input_block = Block::default().borders(Borders::BOTTOM);
        let input_paragraph = Paragraph::new(format!("> {}", self.input)).block(input_block);
        ctx.frame.render_widget(input_paragraph, chunks[0]);

        // List
        let filtered = self.filtered_commands();
        let items: Vec<ListItem> = filtered
            .iter()
            .enumerate()
            .map(|(i, cmd)| {
                let style = if i == self.selected_index {
                    Style::default().bg(Color::DarkGray).fg(Color::White)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(vec![Span::styled((*cmd).clone(), style)]))
            })
            .collect();

        let list = List::new(items);
        ctx.frame.render_widget(list, chunks[1]);

        Ok(())
    }
}
