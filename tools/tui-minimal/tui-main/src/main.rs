//! Wireframe-AI Minimal TUI
//!
//! A simple, fast terminal UI for Wireframe-AI

mod command_palette;

use anyhow::Result;
use command_palette::CommandPalettePlugin;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::Arc;
use std::time::Instant;
use tui_config::TuiConfig;
use tui_core::{Application, PluginEvent};
use tui_input::{InputBuffer, InputEvent, InputHandler};
use tui_render::{RenderState, Renderer};

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = TuiConfig::load_default()?;

    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create application
    let app = Application::new(config.clone());

    // Register Command Palette Plugin
    {
        let pm_arc = app.plugin_manager();
        let mut pm = pm_arc.write().await;
        pm.register(Box::new(CommandPalettePlugin::new()));
    }

    // Initialize application (connect to NATS)
    if let Err(e) = app.initialize().await {
        eprintln!("Warning: Failed to connect to NATS: {}", e);
    }

    // Create renderer
    let renderer = Renderer::new();

    // Create input handler and buffer
    let mut input_handler = InputHandler::new();
    let mut event_receiver = input_handler.take_receiver().expect("InputHandler should have a receiver upon creation");
    let input_buffer = Arc::new(tokio::sync::Mutex::new(InputBuffer::new()));
    let last_event_time = Arc::new(tokio::sync::Mutex::new(Option::<Instant>::None));

    // Spawn input handler in background
    tokio::task::spawn_blocking(move || {
        let mut handler = input_handler;
        let _ = handler.run();
    });

    let app_state_clone = app.state();

    // Main event loop
    'mainloop: loop {
        // Update render state
        let nats_manager = app_state_clone.read().await.nats_manager.clone();
        let nats_connected = nats_manager.is_connected();
        let pending_tasks = nats_manager.get_pending_tasks().await.len();
        let input_str = input_buffer.lock().await.as_str().to_string();

        let render_state = RenderState {
            messages: vec![],
            input: input_str,
            nats_connected,
            pending_tasks,
            sidebar_items: vec!["Recent Chat 1".to_string(), "Recent Chat 2".to_string()],
        };

        // Render
        renderer
            .render(&mut terminal, render_state, app.plugin_manager())
            .await?;

        // Process all pending input events
        while let Ok(event) = event_receiver.try_recv() {
            if event.is_quit() {
                break 'mainloop;
            }

            // Debounce: skip rapid duplicate events within 50ms
            {
                let mut last_time = last_event_time.lock().await;
                if let Some(ref lt) = *last_time {
                    if lt.elapsed() < std::time::Duration::from_millis(50) {
                        continue;
                    }
                }
                *last_time = Some(Instant::now());
            }

            if let InputEvent::Plugin(ref pe) = event {
                // First pass to plugins
                let mut consumed = false;
                {
                    let pm_arc = app.plugin_manager();
                    let mut pm = pm_arc.write().await;
                    if pm.handle_event(pe).await? {
                        consumed = true;
                    }
                }

                if consumed {
                    continue; // Event consumed by plugin
                }

                // If not consumed, handle default behavior
                let mut buffer = input_buffer.lock().await;
                match pe {
                    PluginEvent::Input(c) => buffer.push_char(*c),
                    PluginEvent::Backspace => {
                        buffer.pop_char();
                    }
                    PluginEvent::Enter => {
                        if !buffer.is_empty() {
                            let input = buffer.as_str().to_string();
                            buffer.clear();
                            // Submit task
                            let state = app_state_clone.read().await;
                            if state.is_nats_connected() {
                                if let Err(e) = state.submit_task(&input).await {
                                    eprintln!("Failed to submit task: {}", e);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    // Shutdown
    app.shutdown().await?;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
