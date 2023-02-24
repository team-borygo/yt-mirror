use std::{
    collections::HashMap,
    path::Path,
    thread::{self},
};

use anyhow::Result;
use downloader::{Downloader, DownloaderState};
use library::firefox_library::FirefoxLibrary;
use types::{Process, ProcessState};
use ui::{close_ui, draw_ui, prepare_ui, should_quit};

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
mod ui;
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

    // This way after all messages are processed it will stop downloader threads
    drop(process_channel_s);

    let mut terminal = prepare_ui()?;

    let mut downloader_states: HashMap<String, DownloaderState> = HashMap::new();
    let mut results: Vec<DownloadResult> = vec![];

    terminal.draw(|f| draw_ui(f, &downloader_states, &results))?;

    while let Ok(message) = message_channel_r.recv() {
        terminal.draw(|f| draw_ui(f, &downloader_states, &results))?;

        if should_quit()? {
            break;
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

    close_ui(terminal)?;

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
