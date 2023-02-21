use std::{path::Path};

use anyhow::Result;

use crate::{api::cli::{Cli, CliCommand}, library::{chromium_library::ChromiumLibrary, Library}, youtube::is_youtube_video_url, types::Bookmark};

mod api;
mod library;
mod types;
mod youtube;

fn main() -> Result<()> {
    println!("Hello, world!");

    let cli = Cli {};
    let program = cli.run();

    let library: Box<dyn Library> = Box::new(ChromiumLibrary {});

    match program.command {
        CliCommand::Prepare { processes, bookmarks } => {
            let bookmarks: Vec<Bookmark> = library
                .get_bookmarks(Path::new(&processes)).unwrap()
                .into_iter()
                .filter(|b| is_youtube_video_url(&b.url).unwrap_or(false))
                .collect();

            dbg!(bookmarks);
        }
        _ => todo!()
    }

    return Ok(());
}
