mod git_compat;
mod tui;
mod vcs;

use anyhow::Result;
use chrono::{DateTime, Utc};
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
    /// Branch operations
    Branch {
        #[command(subcommand)]
        command: Option<BranchCommands>,
    },
    /// Tag operations
    Tag {
        #[command(subcommand)]
        command: Option<TagCommands>,
    },
    /// Switch branches
    Checkout {
        /// Branch name to switch to
        branch: String,
        /// Create new branch
        #[arg(short = 'b', long)]
        new_branch: bool,
    },
    /// Launch interactive TUI
    Tui {
        /// Path to repository (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

#[derive(Subcommand)]
enum BranchCommands {
    /// List all branches
    List,
    /// Create a new branch
    Create {
        /// Branch name
        name: String,
        /// Starting point (commit ID)
        #[arg(short, long)]
        start_point: Option<String>,
    },
    /// Delete a branch
    Delete {
        /// Branch name
        name: String,
        /// Force delete
        #[arg(short, long)]
        force: bool,
    },
    /// Rename a branch
    Rename {
        /// Old branch name
        old_name: String,
        /// New branch name
        new_name: String,
    },
}

#[derive(Subcommand)]
enum TagCommands {
    /// List all tags
    List,
    /// Create a new tag
    Create {
        /// Tag name
        name: String,
        /// Commit ID (defaults to HEAD)
        #[arg(short, long)]
        commit: Option<String>,
        /// Tag message (creates annotated tag)
        #[arg(short, long)]
        message: Option<String>,
    },
    /// Delete a tag
    Delete {
        /// Tag name
        name: String,
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
        Some(Commands::Branch { command }) => {
            let repo = vcs::Repository::open(".")?;
            match command {
                Some(BranchCommands::List) | None => {
                    let branches = repo.list_branches()?;
                    let current = repo.current_branch().ok();
                    for branch in branches {
                        if Some(&branch) == current.as_ref() {
                            println!("* {}", branch);
                        } else {
                            println!("  {}", branch);
                        }
                    }
                }
                Some(BranchCommands::Create { name, start_point }) => {
                    repo.create_branch(&name, start_point.as_deref())?;
                    println!("Created branch '{}'", name);
                }
                Some(BranchCommands::Delete { name, force }) => {
                    repo.delete_branch(&name, force)?;
                    println!("Deleted branch '{}'", name);
                }
                Some(BranchCommands::Rename { old_name, new_name }) => {
                    repo.rename_branch(&old_name, &new_name)?;
                    println!("Renamed branch '{}' to '{}'", old_name, new_name);
                }
            }
        }
        Some(Commands::Tag { command }) => {
            let repo = vcs::Repository::open(".")?;
            match command {
                Some(TagCommands::List) | None => {
                    let tags = repo.list_tags()?;
                    for tag in tags {
                        println!("{}", tag);
                    }
                }
                Some(TagCommands::Create { name, commit, message }) => {
                    repo.create_tag(&name, commit.as_deref(), message.as_deref())?;
                    println!("Created tag '{}'", name);
                }
                Some(TagCommands::Delete { name }) => {
                    repo.delete_tag(&name)?;
                    println!("Deleted tag '{}'", name);
                }
            }
        }
        Some(Commands::Checkout { branch, new_branch }) => {
            let repo = vcs::Repository::open(".")?;
            if new_branch {
                repo.checkout_new_branch(&branch)?;
                println!("Switched to a new branch '{}'", branch);
            } else {
                repo.checkout_branch(&branch)?;
                println!("Switched to branch '{}'", branch);
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
                            app.switch_mode(AppMode::Branches);
                            let _ = app.refresh_branches();
                        }
                        KeyCode::Char('4') => {
                            app.switch_mode(AppMode::Tags);
                            let _ = app.refresh_tags();
                        }
                        KeyCode::Char('5') => {
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
                                AppMode::Branches => {
                                    let _ = app.refresh_branches();
                                    app.set_message("Branches refreshed".to_string());
                                }
                                AppMode::Tags => {
                                    let _ = app.refresh_tags();
                                    app.set_message("Tags refreshed".to_string());
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
