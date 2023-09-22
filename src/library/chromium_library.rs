use std::fs;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::Bookmark;

use super::Library;

pub struct ChromiumLibrary;

impl Library for ChromiumLibrary {
    fn get_bookmarks(&self, path: &std::path::Path) -> Result<Vec<Bookmark>> {
        let data = fs::read_to_string(path)?;
        let json = serde_json::from_str(&data)?;

        return Ok(collect_bookmarks(json));
    }
}

fn parse_bookmarks(entry: &ChromiumBookmark) -> Vec<Bookmark> {
    if let Some(children) = &entry.children {
        children
            .iter()
            .flat_map(|child| parse_bookmarks(child))
            .collect()
    } else {
        if let Some(url) = &entry.url {
            vec![Bookmark {
                title: entry.name.clone(),
                url: url.clone(),
            }]
        } else {
            panic!("Cannot parse bookmark: {}", entry.name);
        }
    }
}

fn collect_bookmarks(core: ChromiumBookmarkCore) -> Vec<Bookmark> {
    return [core.roots.bookmark_bar, core.roots.other, core.roots.synced]
        .iter()
        .flat_map(|b| parse_bookmarks(b))
        .collect();
}

#[derive(Serialize, Deserialize)]
struct ChromiumBookmarkCore {
    roots: ChromiumBookmarkRoots,
}

// @TODO We could differentiate between different bookmark types (folder and url)
#[derive(Serialize, Deserialize)]
struct ChromiumBookmarkRoots {
    bookmark_bar: ChromiumBookmark,
    other: ChromiumBookmark,
    synced: ChromiumBookmark,
}

#[derive(Serialize, Deserialize)]
struct ChromiumBookmark {
    children: Option<Vec<ChromiumBookmark>>,
    url: Option<String>,
    name: String,
}
