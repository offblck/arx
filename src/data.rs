use std::{fs, path::PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    command_types::{Category, Status},
    config::PROJECT_DIRS,
    errors::Result,
};

#[derive(Serialize, Deserialize)]
pub struct BookmarkStore {
    pub last_save_location: PathBuf,
    pub next_id: usize,
    pub bookmarks: Vec<Bookmark>,
}

impl Default for BookmarkStore {
    fn default() -> Self {
        BookmarkStore {
            last_save_location: PROJECT_DIRS.data_path.clone(),
            next_id: 1,
            bookmarks: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Bookmark {
    pub id: usize,
    pub title: String,
    pub category: Category,
    pub url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub notes: Option<String>,
    pub status: Status,
    pub hidden: bool,
    pub created_at: DateTime<Utc>,
}

impl BookmarkStore {
    pub fn load(save_location: Option<PathBuf>) -> Result<BookmarkStore> {
        let save_location = match save_location {
            Some(path) => path,
            None => PROJECT_DIRS.data_path.clone(),
        };
        if !save_location.exists() {
            return Ok(BookmarkStore::default());
        }
        let data = fs::read_to_string(&save_location)?;
        let store: BookmarkStore = serde_json::from_str(&data)?;
        if store.last_save_location != save_location {
            if store.last_save_location.exists() {
                let old_data = fs::read_to_string(&store.last_save_location)?;
                let old_store: BookmarkStore = serde_json::from_str(&old_data)?;
            }
        }
        Ok(store)
    }

    pub fn save(&mut self) -> Result<()> {
        let path = &PROJECT_DIRS.data_path;
        let data = serde_json::to_string(&self)?;
        Ok(fs::write(path, data)?)
    }
}
