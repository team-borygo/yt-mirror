use std::{
    collections::HashMap,
    io::{self, Stdout},
    path::Path,
    thread::{self},
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    event::{poll, read, DisableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use downloader::{Downloader, DownloaderState};
use library::firefox_library::FirefoxLibrary;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph},
    Frame, Terminal,
};
use types::{Process, ProcessState};

use crate::{
    api::cli::{Cli, CliCommand},
    downloader::{DownloadResult, DownloaderMessage},
    library::{chromium_library::ChromiumLibrary, Library},
    process_repository::ProcessRepository,
    types::Bookmark,
    youtube::get_youtube_video_id,
};

mod api;
mod downloader;
mod library;
mod process_repository;
mod types;
mod youtube;

fn main() -> Result<()> {
    let cli = Cli {};
    let program = cli.run();

    match program.command {
        CliCommand::Prepare {
            processes,
            bookmarks,
        } => command_prepare(processes, bookmarks),
        CliCommand::Synchronize {
            processes,
            target,
            tmp,
            filter,
        } => command_synchronize(processes, target, tmp, filter),
        CliCommand::Failed { processes, short } => command_failed(processes, short),
    }
}

fn draw_ui<B: Backend>(
    f: &mut Frame<B>,
    downloader_states: &HashMap<String, DownloaderState>,
    results: &Vec<DownloadResult>,
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
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(Style::default().fg(Color::White).bg(Color::Black))
        .percent(20);

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

fn command_prepare(processes: String, bookmarks: String) -> Result<()> {
    let bookmarks_file = Path::new(&bookmarks).file_name().and_then(|f| f.to_str());

    let library: Box<dyn Library> = match bookmarks_file {
        Some("Bookmarks") => Box::new(ChromiumLibrary {}),
        Some("Bookmarks.json") => Box::new(ChromiumLibrary {}),
        Some("places.sqlite") => Box::new(FirefoxLibrary {}),
        _ => todo!(),
    };

    let process_list: Vec<Process> = library
        .get_bookmarks(Path::new(&bookmarks))?
        .into_iter()
        .filter_map(|b| {
            let video_id = get_youtube_video_id(&b.url).unwrap_or(None);

            video_id.and_then(|id| Some(bookmark_to_process(&b, id)))
        })
        .collect();

    let mut process_repository = ProcessRepository::new(&processes)?;

    process_repository.save_many(&process_list)?;

    Ok(())
}

fn command_synchronize(
    processes: String,
    target: String,
    tmp: String,
    filter: Option<String>,
) -> Result<()> {
    let process_repository = ProcessRepository::new(&processes)?;

    let pending = process_repository.get_by_state(ProcessState::Pending)?;

    let (process_channel_s, process_channel_r) = crossbeam_channel::unbounded();
    let (message_channel_is, message_channel_r) = crossbeam_channel::unbounded();

    for p in pending {
        process_channel_s.send(p.youtube_id)?;
    }

    let downloader_count = 10;

    let mut handles = vec![];

    for _ in 0..downloader_count {
        let downloader = Downloader::new(
            process_channel_r.clone(),
            message_channel_is.clone(),
            target.clone(),
            tmp.clone(),
            filter.clone(),
        );

        let handle = thread::spawn(move || {
            downloader.start();
        });

        handles.push(handle);
    }

    // This way after all messages are processed it will stop downloaders
    drop(process_channel_s);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut downloader_states: HashMap<String, DownloaderState> = HashMap::new();
    let mut results: Vec<DownloadResult> = vec![];

    terminal.draw(|f| draw_ui(f, &downloader_states, &results))?;

    while let Ok(message) = message_channel_r.recv() {
        terminal.draw(|f| draw_ui(f, &downloader_states, &results))?;

        if poll(Duration::from_millis(0))? {
            let event = read()?;

            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                }) => break,
                Event::Key(KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: KeyCode::Char('c'),
                    ..
                }) => break,
                _ => {}
            }
        }

        match message {
            DownloaderMessage::Result(result) => {
                let result_clone = result.clone();
                results.insert(0, result_clone);

                match result {
                    DownloadResult::DownloadFailed {
                        youtube_id,
                        error_message,
                        ..
                    } => process_repository.fail(&youtube_id, &error_message),
                    DownloadResult::DownloadFinished { youtube_id, .. } => {
                        process_repository.finish(&youtube_id);
                    }
                    DownloadResult::DownloadSkipped { youtube_id, .. } => {
                        process_repository.skip(&youtube_id);
                    }
                }
            }
            DownloaderMessage::State(state) => {
                let state_clone = state.clone();
                match state {
                    DownloaderState::Downloading { downloader_id, .. } => {
                        downloader_states.insert(downloader_id.clone(), state_clone);
                    }
                    DownloaderState::Waiting { downloader_id } => {
                        downloader_states.insert(downloader_id.clone(), state_clone);
                    }
                    DownloaderState::Finished { downloader_id } => {
                        downloader_states.insert(downloader_id.clone(), state_clone);
                    }
                    DownloaderState::Crashed { downloader_id } => {
                        downloader_states.insert(downloader_id.clone(), state_clone);
                    }
                }
            }
        }

        // This is not the best solution to quit this loop
        // but I don't know why the channel disconnects
        let is_workers_done = handles.iter().all(|h| h.is_finished());
        let is_channel_empty = message_channel_r.is_empty();
        if is_workers_done && is_channel_empty {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn command_failed(processes: String, short: bool) -> Result<()> {
    let process_repository = ProcessRepository::new(&processes)?;

    let pending = process_repository.get_by_state(ProcessState::Failed)?;

    for process in pending {
        if short {
            println!("{}", process.youtube_id);
        } else {
            println!(
                "{} | {}",
                process.youtube_id,
                process.error.unwrap_or("".to_string())
            );
        }
    }

    Ok(())
}

fn bookmark_to_process(_bookmark: &Bookmark, youtube_id: String) -> Process {
    Process {
        error: None,
        state: ProcessState::Pending,
        youtube_id,
    }
}
