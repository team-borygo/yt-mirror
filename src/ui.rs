use std::{
    collections::HashMap,
    io::{self, Stdout},
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame, Terminal,
};

use crate::downloader::{DownloadResult, DownloaderState};

pub fn prepare_ui() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    Ok(terminal)
}

pub fn close_ui(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

pub fn should_quit() -> Result<bool> {
    if poll(Duration::from_millis(100))? {
        let event = read()?;

        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => Ok(true),
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('c'),
                ..
            }) => Ok(true),
            _ => Ok(false),
        }
    } else {
        Ok(false)
    }
}

pub fn draw_ui<B: Backend>(
    f: &mut Frame<B>,
    downloader_states: &HashMap<String, DownloaderState>,
    results: &Vec<DownloadResult>,
    progress: &(u32, u32),
) -> () {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(root[1]);

    let progress_block = Gauge::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Download progress"),
        )
        .gauge_style(Style::default().fg(Color::White).bg(Color::Black))
        .label(format!("{} / {}", progress.0, progress.1))
        .ratio(progress.0 as f64 / progress.1 as f64);

    let actors_column_text: Vec<Spans> = downloader_states
        .values()
        .map(|state| match state {
            DownloaderState::Downloading {
                downloader_id,
                youtube_id,
            } => format!("{}: Downloading {}", downloader_id, youtube_id).to_string(),
            DownloaderState::Finished { downloader_id } => {
                format!("{}: Finished", downloader_id).to_string()
            }
            DownloaderState::Waiting { downloader_id } => {
                format!("{}: Waiting", downloader_id).to_string()
            }
            DownloaderState::Crashed { downloader_id } => {
                format!("{}: Crashed", downloader_id).to_string()
            }
        })
        .map(|str| Spans::from(Span::raw(str)))
        .collect();

    let results_column_text: Vec<ListItem> = results
        .iter()
        .map(|result| match result {
            DownloadResult::DownloadSkipped {
                downloader_id,
                youtube_id,
            } => {
                format!("{} skipped {}", downloader_id, youtube_id)
            }
            DownloadResult::DownloadFailed {
                downloader_id,
                youtube_id,
                error_message,
            } => {
                format!(
                    "{} failed {} because {}",
                    downloader_id, youtube_id, error_message
                )
            }
            DownloadResult::DownloadFinished {
                downloader_id,
                youtube_id,
            } => {
                format!("{} finished {}", downloader_id, youtube_id)
            }
        })
        .map(|str| ListItem::new(vec![Spans::from(Span::raw(str))]))
        .collect();

    let actor_block = Paragraph::new(actors_column_text)
        .block(Block::default().title("Downloaders").borders(Borders::ALL));

    let results_block = List::new(results_column_text)
        .block(Block::default().title("Results").borders(Borders::ALL));

    f.render_widget(progress_block, root[0]);
    f.render_widget(results_block, layout[0]);
    f.render_widget(actor_block, layout[1]);
}
