use std::{path::Path};

use anyhow::Result;
use types::{Process, ProcessState};

use crate::{api::cli::{Cli, CliCommand}, library::{chromium_library::ChromiumLibrary, Library}, youtube::get_youtube_video_id, types::Bookmark, process_repository::ProcessRepository};

mod api;
mod library;
mod types;
mod youtube;
mod process_repository;

fn main() -> Result<()> {
    let cli = Cli {};
    let program = cli.run();

    let library: Box<dyn Library> = Box::new(ChromiumLibrary {});

    match program.command {
        CliCommand::Prepare { processes, bookmarks } => {
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
        }
        _ => todo!()
    }

    return Ok(());
}

fn bookmark_to_process(_bookmark: &Bookmark, youtube_id: String) -> Process {
    Process {
        error: None,
        state: ProcessState::Pending,
        youtube_id,
    }
}