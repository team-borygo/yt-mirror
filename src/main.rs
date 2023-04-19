use std::{
    collections::HashMap,
    path::Path,
    thread::{self},
};

use anyhow::{Error, Result};
use config::config::Config;
use data::NAMES;
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
mod config;
mod data;
mod downloader;
mod library;
mod process_repository;
mod types;
mod ui;
mod youtube;

fn main() -> Result<()> {
    let config = Config::new_from_directory()?;

    let cli = Cli {};
    let program = cli.run();

    match program.command {
        CliCommand::Prepare {} => command_prepare(&config),
        CliCommand::Synchronize { filter, retry } => command_synchronize(&config, filter, retry),
        CliCommand::Failed { short } => command_failed(&config, short),
    }
}

fn command_prepare(config: &Config) -> Result<()> {
    let bookmark_files = config.get_bookmark_files();

    bookmark_files
        .iter()
        .map(|bookmarks| {
            let bookmarks_file = Path::new(&bookmarks).file_name().and_then(|f| f.to_str());

            let library: Box<dyn Library> = match bookmarks_file {
                Some("Bookmarks") | Some("Bookmarks.json") => Box::new(ChromiumLibrary {}),
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

            let mut process_repository = ProcessRepository::new(config.get_process_path())?;

            process_repository.save_many(&process_list)?;

            println!(
                "Bookmarks from {} prepared ({} overall)!",
                bookmarks.display(),
                process_list.len()
            );

            Ok(())
        })
        .collect()
}

fn command_synchronize(config: &Config, filter: Option<String>, retry: bool) -> Result<()> {
    let process_repository = ProcessRepository::new(config.get_process_path())?;

    let processes = {
        let pending = process_repository.get_by_state(ProcessState::Pending)?;

        if retry {
            let failed = process_repository.get_by_state(ProcessState::Failed)?;
            failed.into_iter().chain(pending.into_iter()).collect()
        } else {
            pending
        }
    };

    let process_count = processes.len();

    if process_count == 0 {
        println!("No pending bookmarks to synchronize");
        return Ok(());
    }

    let (process_channel_s, process_channel_r) = crossbeam_channel::unbounded();
    let (message_channel_is, message_channel_r) = crossbeam_channel::unbounded();

    for p in processes {
        process_channel_s.send(p.youtube_id)?;
    }

    let downloader_count = 10;

    let mut handles = vec![];

    for i in 0..downloader_count {
        let downloader = Downloader::new(
            NAMES[i].to_string(),
            process_channel_r.clone(),
            message_channel_is.clone(),
            config.get_target_dir(),
            config.get_tmp_dir(),
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
    let mut progress: (u32, u32) = (0, process_count.try_into()?);

    terminal.draw(|f| draw_ui(f, &downloader_states, &results, &progress))?;

    while let Ok(message) = message_channel_r.recv() {
        if should_quit()? {
            break;
        }

        terminal.draw(|f| draw_ui(f, &downloader_states, &results, &progress))?;

        match message {
            DownloaderMessage::Result(result) => {
                progress.0 += 1;

                let result_clone = result.clone();
                results.insert(0, result_clone);
                results.truncate(40);

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

fn command_failed(config: &Config, short: bool) -> Result<()> {
    let process_repository = ProcessRepository::new(config.get_process_path())?;

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
