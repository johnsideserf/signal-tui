mod app;
mod config;
mod input;
mod signal;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::App;
use config::Config;
use signal::client::SignalClient;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI args
    let args: Vec<String> = std::env::args().collect();
    let mut config_path: Option<&str> = None;
    let mut account: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-c" | "--config" => {
                if i + 1 < args.len() {
                    config_path = Some(&args[i + 1]);
                    i += 2;
                } else {
                    eprintln!("--config requires a path argument");
                    std::process::exit(1);
                }
            }
            "-a" | "--account" => {
                if i + 1 < args.len() {
                    account = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("--account requires a phone number");
                    std::process::exit(1);
                }
            }
            "--help" => {
                eprintln!("signal-tui - Terminal Signal client");
                eprintln!();
                eprintln!("Usage: signal-tui [OPTIONS]");
                eprintln!();
                eprintln!("Options:");
                eprintln!("  -a, --account <NUMBER>  Phone number (E.164 format)");
                eprintln!("  -c, --config <PATH>     Config file path");
                eprintln!("      --help              Show this help");
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                std::process::exit(1);
            }
        }
    }

    // Load config
    let mut config = Config::load(config_path)?;
    if let Some(acct) = account {
        config.account = acct;
    }

    // Create download directory
    if !config.download_dir.exists() {
        std::fs::create_dir_all(&config.download_dir)?;
    }

    // Spawn signal-cli backend
    let mut signal_client = SignalClient::spawn(&config).await?;

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run the app
    let result = run_app(&mut terminal, &mut signal_client, &config).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    // Shut down signal-cli
    signal_client.shutdown().await?;

    if let Err(e) = result {
        eprintln!("Error: {e:?}");
        std::process::exit(1);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    signal_client: &mut SignalClient,
    config: &Config,
) -> Result<()> {
    let mut app = App::new(config.account.clone());
    app.set_connected();

    loop {
        // Render
        terminal.draw(|frame| ui::draw(frame, &app))?;

        // Poll for events with a short timeout so we stay responsive to signal events
        let has_terminal_event = event::poll(Duration::from_millis(50))?;

        if has_terminal_event {
            if let Event::Key(key) = event::read()? {
                match (key.modifiers, key.code) {
                    // Ctrl+C — quit
                    (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                        app.should_quit = true;
                    }
                    // Tab — next conversation
                    (KeyModifiers::NONE, KeyCode::Tab) => {
                        app.next_conversation();
                    }
                    // Shift+Tab — previous conversation
                    (KeyModifiers::SHIFT, KeyCode::BackTab) => {
                        app.prev_conversation();
                    }
                    // Page Up — scroll up
                    (_, KeyCode::PageUp) => {
                        app.scroll_offset = app.scroll_offset.saturating_add(5);
                    }
                    // Page Down — scroll down
                    (_, KeyCode::PageDown) => {
                        app.scroll_offset = app.scroll_offset.saturating_sub(5);
                    }
                    // Enter — submit input
                    (_, KeyCode::Enter) => {
                        if let Some((recipient, body, is_group)) = app.handle_input() {
                            if let Err(e) =
                                signal_client.send_message(&recipient, &body, is_group).await
                            {
                                app.status_message = format!("send error: {e}");
                            }
                        }
                    }
                    // Backspace
                    (_, KeyCode::Backspace) => {
                        if app.input_cursor > 0 {
                            app.input_cursor -= 1;
                            app.input_buffer.remove(app.input_cursor);
                        }
                    }
                    // Delete
                    (_, KeyCode::Delete) => {
                        if app.input_cursor < app.input_buffer.len() {
                            app.input_buffer.remove(app.input_cursor);
                        }
                    }
                    // Left arrow
                    (_, KeyCode::Left) => {
                        app.input_cursor = app.input_cursor.saturating_sub(1);
                    }
                    // Right arrow
                    (_, KeyCode::Right) => {
                        if app.input_cursor < app.input_buffer.len() {
                            app.input_cursor += 1;
                        }
                    }
                    // Home
                    (_, KeyCode::Home) => {
                        app.input_cursor = 0;
                    }
                    // End
                    (_, KeyCode::End) => {
                        app.input_cursor = app.input_buffer.len();
                    }
                    // Regular character input
                    (_, KeyCode::Char(c)) => {
                        app.input_buffer.insert(app.input_cursor, c);
                        app.input_cursor += 1;
                    }
                    _ => {}
                }
            }
        }

        // Drain signal events (non-blocking)
        while let Ok(event) = signal_client.event_rx.try_recv() {
            app.handle_signal_event(event);
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
