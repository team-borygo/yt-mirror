use std::{path::Path, thread::{self}, time};

use anyhow::Result;
use downloader::{Downloader};
use types::{Process, ProcessState};

use crate::{api::cli::{Cli, CliCommand}, library::{chromium_library::ChromiumLibrary, Library}, youtube::get_youtube_video_id, types::Bookmark, process_repository::ProcessRepository, downloader::DownloadStatus};

mod api;
mod library;
mod types;
mod youtube;
mod process_repository;
mod downloader;

fn main() -> Result<()> {
    let cli = Cli {};
    let program = cli.run();

    match program.command {
        CliCommand::Prepare { processes, bookmarks } => command_prepare(
            processes,
            bookmarks
        ),
        CliCommand::Synchronize { processes, target, tmp, filter } => command_synchronize(
            processes,
            target,
            tmp,
            filter
        ),
        CliCommand::Failed { processes, short } => command_failed(
            processes,
            short
        ),
    }
}

fn command_prepare(
    processes: String,
    bookmarks: String
) -> Result<()> {
    // TODO: add support for different bookmarks
    // "Bookmarks" -> loadBookmarks $ CR.ChromeRepository bookmarksPath
    // "Bookmarks.json" -> loadBookmarks $ CR.ChromeRepository bookmarksPath
    // "places.sqlite" -> loadBookmarks $ FR.FirefoxRepository bookmarksPath
    let library: Box<dyn Library> = Box::new(ChromiumLibrary {});

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
    let (result_channel_s, result_channel_r) = crossbeam_channel::unbounded();

    for p in pending {
        process_channel_s.send(p.youtube_id)?;
    }

    let downloader_count = 2;

    let mut handles = vec![];

    for _ in 0..downloader_count {
        let downloader = Downloader::new(
            process_channel_r.clone(),
            result_channel_s.clone(),
            target.clone(),
            tmp.clone(),
            filter.clone()
        );
        let handle = thread::spawn(move || {
            downloader.start();
        });

        handles.push(handle);
    }

    // This way after all messages are processed it will stop downloaders
    drop(process_channel_s);

    while let Ok(result) = result_channel_r.recv() {
        match &result.status {
            DownloadStatus::DownloadFailed { youtube_id, error_message } => {
                process_repository.fail(&youtube_id, &error_message)
            }
            DownloadStatus::DownloadFinished { youtube_id } => {
                process_repository.finish(&youtube_id);
            }
            DownloadStatus::DownloadSkipped { youtube_id } => {
                process_repository.skip(&youtube_id);
            }
        }

        dbg!(result);

        // This is not the best solution to quit this loop
        // but I don't know why the channel disconnects
        let is_workers_done = handles.iter().all(|h| h.is_finished());
        let is_channel_empty = result_channel_r.is_empty();
        if is_workers_done && is_channel_empty {
            break;
        }
    }

    Ok(())
}

fn command_failed(
    processes: String,
    short: bool
) -> Result<()> {
    todo!()
}

fn bookmark_to_process(_bookmark: &Bookmark, youtube_id: String) -> Process {
    Process {
        error: None,
        state: ProcessState::Pending,
        youtube_id,
    }
}