use std::path::Path;

use anyhow::Result;

use crate::types::Bookmark;

pub mod firefox_library;
pub mod chromium_library;

pub trait Library {
    fn get_bookmarks(&self, path: &Path) -> Result<Vec<Bookmark>>;
}