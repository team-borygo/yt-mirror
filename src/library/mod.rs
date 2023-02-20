use std::path::Path;

use crate::types::Bookmark;

pub mod firefox_library;
pub mod chromium_library;

pub trait Library {
    fn get_bookmarks(path: &Path) -> Result<Vec<Bookmark>, std::io::Error>;
}