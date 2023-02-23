use anyhow::Result;
use rusqlite::Connection;

use crate::types::Bookmark;

use super::Library;

pub struct FirefoxLibrary;

impl Library for FirefoxLibrary {
    fn get_bookmarks(&self, path: &std::path::Path) -> Result<Vec<Bookmark>> {
        let connection = Connection::open(path)?;

        let mut stmt = connection.prepare("
            SELECT moz_bookmarks.title, moz_places.url
            FROM moz_bookmarks
            INNER JOIN moz_places
            ON moz_places.id = moz_bookmarks.fk
        ")?;

        let iter = stmt.query_map([], |row| {
            Ok(Bookmark {
                title: row.get(0)?,
                url: row.get(1)?
            })
        })?;

        Ok(iter.map(|p| p.unwrap()).collect())  
    }
}