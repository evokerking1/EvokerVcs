use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::{App, AppMode};

pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(f.area());

    render_header(f, app, chunks[0]);
    render_content(f, app, chunks[1]);
    render_footer(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let mode_text = match app.mode {
        AppMode::Status => "STATUS",
        AppMode::Log => "LOG",
        AppMode::Branches => "BRANCHES",
        AppMode::Tags => "TAGS",
        AppMode::Staging => "STAGING",
        AppMode::Help => "HELP",
    };

    let compat_mode = if app.git_compat_mode {
        " [GIT COMPAT]"
    } else {
        ""
    };

    let branch = app.get_current_branch();
    
    let title = Paragraph::new(vec![
        Line::from(vec![
            Span::styled(
                "EvokerVcs",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(compat_mode),
            Span::raw(" | "),
            Span::styled(
                mode_text,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | Branch: "),
            Span::styled(
                &branch,
                Style::default().fg(Color::Green),
            ),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL))
    .alignment(Alignment::Left);

    f.render_widget(title, area);
}

fn render_content(f: &mut Frame, app: &App, area: Rect) {
    match app.mode {
        AppMode::Status => render_status(f, app, area),
        AppMode::Log => render_log(f, app, area),
        AppMode::Branches => render_branches(f, app, area),
        AppMode::Tags => render_tags(f, app, area),
        AppMode::Staging => render_staging(f, app, area),
        AppMode::Help => render_help(f, app, area),
    }
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .status_lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(line.clone()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Repository Status").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, area);
}

fn render_log(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .log_entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            let content = format!(
                "{} - {} - {}",
                &entry.id[..7],
                entry.author,
                entry.message.lines().next().unwrap_or("")
            );
            
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Commit History").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, area);
}

fn render_branches(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .branches
        .iter()
        .enumerate()
        .map(|(i, branch)| {
            let is_current = branch == &app.current_branch;
            let prefix = if is_current { "* " } else { "  " };
            let content = format!("{}{}", prefix, branch);
            
            let mut style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            if is_current {
                style = style.fg(Color::Green).add_modifier(Modifier::BOLD);
            }
            
            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Branches").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, area);
}

fn render_tags(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .tags
        .iter()
        .enumerate()
        .map(|(i, tag)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            ListItem::new(tag.clone()).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Tags").borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(list, area);
}

fn render_staging(f: &mut Frame, _app: &App, area: Rect) {
    let text = Paragraph::new("Staging area (not yet implemented)")
        .block(Block::default().title("Staging").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(text, area);
}

fn render_help(f: &mut Frame, _app: &App, area: Rect) {
    let help_text = vec![
        Line::from("Keyboard Shortcuts:"),
        Line::from(""),
        Line::from("  1 - Status view"),
        Line::from("  2 - Commit log view"),
        Line::from("  3 - Branches view"),
        Line::from("  4 - Tags view"),
        Line::from("  5 - Staging area"),
        Line::from("  h - Help (this screen)"),
        Line::from(""),
        Line::from("  ↑/k - Move up"),
        Line::from("  ↓/j - Move down"),
        Line::from(""),
        Line::from("  r - Refresh current view"),
        Line::from("  q - Quit"),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(Block::default().title("Help").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let message = if let Some(msg) = &app.message {
        msg.clone()
    } else {
        "Press 'h' for help, 'q' to quit".to_string()
    };

    let footer = Paragraph::new(message)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan));

    f.render_widget(footer, area);
}
