use std::{path::Path};

use anyhow::Result;
use downloader::download_yt;
use types::{Process, ProcessState};

use crate::{api::cli::{Cli, CliCommand}, library::{chromium_library::ChromiumLibrary, Library}, youtube::get_youtube_video_id, types::Bookmark, process_repository::ProcessRepository};

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
    todo!()
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