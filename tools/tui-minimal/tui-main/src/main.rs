//! Wireframe-AI Minimal TUI
//! 
//! A simple, fast terminal UI for Wireframe-AI

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::Arc;
use tokio::sync::mpsc;
use tui_config::TuiConfig;
use tui_core::Application;
use tui_input::{InputBuffer, InputEvent};
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
    
    // Initialize application (connect to NATS)
    if let Err(e) = app.initialize().await {
        eprintln!("Warning: Failed to connect to NATS: {}", e);
    }
    
    // Create renderer
    let renderer = Renderer::new();
    
    // Create input buffer
    let input_buffer = Arc::new(tokio::sync::Mutex::new(InputBuffer::new()));
    
    // Create channel for input events
    let (event_sender, mut event_receiver) = mpsc::unbounded_channel();
    
    // Spawn input handler in background
    let input_buffer_clone = input_buffer.clone();
    let event_sender_clone = event_sender.clone();
    let app_state_clone = app.state();
    tokio::spawn(async move {
        use crossterm::event::{self, Event, KeyCode, KeyModifiers};
        
        loop {
            if event::poll(std::time::Duration::from_millis(100)).unwrap_or(false) {
                if let Event::Key(key) = event::read().unwrap() {
                    let event = match key.code {
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
                        KeyCode::Esc => InputEvent::CtrlQ,
                        _ => InputEvent::Unknown,
                    };
                    
                    if event == InputEvent::CtrlC || event == InputEvent::CtrlQ {
                        let _ = event_sender_clone.send(event);
                        break;
                    }
                    
                    // Handle input
                    let mut buffer = input_buffer_clone.lock().await;
                    match event {
                        InputEvent::Char(c) => buffer.push_char(c),
                        InputEvent::Backspace => { buffer.pop_char(); }
                        InputEvent::Enter => {
                            if !buffer.is_empty() {
                                let input = buffer.as_str().to_string();
                                buffer.clear();
                                // Submit task
                                let app_state = app_state_clone.read().await;
                                if app_state.is_nats_connected() {
                                    if let Err(e) = app_state.submit_task(&input).await {
                                        eprintln!("Failed to submit task: {}", e);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    
                    let _ = event_sender_clone.send(event);
                }
            }
        }
    });
    
    // Main event loop
    loop {
        // Update render state
        let nats_manager = app.state().read().await.nats_manager.clone();
        let nats_connected = nats_manager.is_connected();
        let pending_tasks = nats_manager.get_pending_tasks().await.len();
        let input_str = input_buffer.lock().await.as_str().to_string();
        
        let render_state = RenderState {
            messages: vec![],
            input: input_str,
            nats_connected,
            pending_tasks,
        };
        
        // Render
        renderer.render(&mut terminal, render_state).await?;
        
        // Check for quit event
        if let Ok(event) = event_receiver.try_recv() {
            if event == InputEvent::CtrlC || event == InputEvent::CtrlQ {
                break;
            }
        }
        
        tokio::time::sleep(std::time::Duration::from_millis(250)).await;
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
