mod git_compat;
mod tui;
mod vcs;

use anyhow::Result;
use chrono;
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;

use tui::{App, AppMode};

#[derive(Parser)]
#[command(name = "evokervcs")]
#[command(about = "A custom git-like version control system with TUI and git compatibility", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new repository
    Init {
        /// Path to initialize repository (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Add files to the staging area
    Add {
        /// Files to add
        files: Vec<PathBuf>,
    },
    /// Commit staged changes
    Commit {
        /// Commit message
        #[arg(short, long)]
        message: String,
        /// Author name
        #[arg(short, long, default_value = "EvokerVcs User")]
        author: String,
    },
    /// Show repository status
    Status,
    /// Show commit history
    Log {
        /// Number of commits to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    /// Launch interactive TUI
    Tui {
        /// Path to repository (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init { path }) => {
            vcs::operations::init(path)?;
        }
        Some(Commands::Add { files }) => {
            vcs::operations::add(".", files)?;
        }
        Some(Commands::Commit { message, author }) => {
            vcs::operations::commit(".", message, author)?;
        }
        Some(Commands::Status) => {
            let status_lines = vcs::operations::status(".")?;
            for line in status_lines {
                println!("{}", line);
            }
        }
        Some(Commands::Log { limit }) => {
            let log_entries = vcs::operations::log(".", limit)?;
            for entry in log_entries {
                println!("commit {}", entry.id);
                println!("Author: {}", entry.author);
                println!("Date:   {}", chrono::DateTime::<chrono::Utc>::from_timestamp(entry.timestamp, 0)
                    .map(|dt| dt.to_rfc2822())
                    .unwrap_or_else(|| "Unknown".to_string()));
                println!();
                for line in entry.message.lines() {
                    println!("    {}", line);
                }
                println!();
            }
        }
        Some(Commands::Tui { path }) => {
            run_tui(path)?;
        }
        None => {
            // Default to TUI mode
            run_tui(".")?;
        }
    }

    Ok(())
}

fn run_tui<P: Into<PathBuf>>(path: P) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(path.into());
    
    // Initialize data
    let _ = app.refresh_status();
    let _ = app.refresh_log();

    // Run app
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| tui::ui::render(f, app))?;

        if app.should_quit {
            break;
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => app.quit(),
                        KeyCode::Char('1') => {
                            app.switch_mode(AppMode::Status);
                            let _ = app.refresh_status();
                        }
                        KeyCode::Char('2') => {
                            app.switch_mode(AppMode::Log);
                            let _ = app.refresh_log();
                        }
                        KeyCode::Char('3') => {
                            app.switch_mode(AppMode::Staging);
                        }
                        KeyCode::Char('h') => {
                            app.switch_mode(AppMode::Help);
                        }
                        KeyCode::Char('r') => {
                            match app.mode {
                                AppMode::Status => {
                                    let _ = app.refresh_status();
                                    app.set_message("Status refreshed".to_string());
                                }
                                AppMode::Log => {
                                    let _ = app.refresh_log();
                                    app.set_message("Log refreshed".to_string());
                                }
                                _ => {}
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.move_selection_up();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.move_selection_down();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}
