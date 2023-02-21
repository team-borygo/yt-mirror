use anyhow::Result;

use crate::types::Bookmark;

use super::Library;

pub struct FirefoxLibrary;

impl Library for FirefoxLibrary {
    fn get_bookmarks(&self, path: &std::path::Path) -> Result<Vec<Bookmark>> {
        todo!()
    }
}