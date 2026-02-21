use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;

/// Immutable dashboard view data.
#[derive(Debug, Clone)]
pub struct DashboardInfo {
    pub version: String,
    pub status_active: bool,
    pub filter_pack_last_updated: String,
}

/// Runs terminal dashboard until user presses `q`.
pub fn run_dashboard(info: DashboardInfo) -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let run_result = loop {
        terminal.draw(|frame| {
            let area = frame.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([
                    Constraint::Length(5),
                    Constraint::Length(4),
                    Constraint::Length(4),
                    Constraint::Length(4),
                    Constraint::Length(2),
                ])
                .split(area);

            let logo = Paragraph::new(vec![
                Line::from(Span::styled(" ARMA ", Style::default().fg(Color::Cyan))),
                Line::from(Span::raw("AI Prompt Guardrail")),
            ])
            .block(Block::default().title("Logo").borders(Borders::ALL));

            let version = Paragraph::new(format!("Current Version: {}", info.version))
                .block(Block::default().title("Version").borders(Borders::ALL));

            let status_text = if info.status_active {
                "Active"
            } else {
                "Inactive"
            };
            let status = Paragraph::new(format!("Status: {status_text}"))
                .block(Block::default().title("Service").borders(Borders::ALL));

            let updated = Paragraph::new(format!(
                "FilterPack Last Updated: {}",
                info.filter_pack_last_updated
            ))
            .block(Block::default().title("FilterPack").borders(Borders::ALL));

            let footer =
                Paragraph::new("Press q to quit").block(Block::default().borders(Borders::NONE));

            frame.render_widget(logo, chunks[0]);
            frame.render_widget(version, chunks[1]);
            frame.render_widget(status, chunks[2]);
            frame.render_widget(updated, chunks[3]);
            frame.render_widget(footer, chunks[4]);
        })?;

        if event::poll(Duration::from_millis(200))?
            && matches!(event::read()?, Event::Key(key) if key.code == KeyCode::Char('q'))
        {
            break Ok(());
        }
    };

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    run_result
}
