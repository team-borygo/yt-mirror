use crate::types::Bookmark;

use super::Library;

pub struct FirefoxLibrary;

impl Library for FirefoxLibrary {
    fn get_bookmarks(path: &std::path::Path) -> Result<Vec<Bookmark>, std::io::Error> {
        todo!()
    }
}