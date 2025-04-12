use std::{fs, path::PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    command_types::{Category, Status},
    config::{Config, PROJECT_DIRS},
    errors::Result,
};

pub struct Arx {
    pub store: BookmarkStore,
    pub config: Config,
}

#[derive(Serialize, Deserialize)]
pub struct BookmarkStore {
    pub next_id: usize,
    pub bookmarks: Vec<Bookmark>,
}

impl Default for BookmarkStore {
    fn default() -> Self {
        BookmarkStore { next_id: 1, bookmarks: Vec::new() }
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

// #[derive(Serialize, Deserialize)]
// struct Metadata {
//     prev_save_location: PathBuf,
// }

impl BookmarkStore {
    pub fn load() -> Result<BookmarkStore> {
        // amend_possible_change(&save_location)?;

        if !PROJECT_DIRS.save_location.exists() {
            return Ok(BookmarkStore::default());
        }
        let data = fs::read_to_string(&PROJECT_DIRS.save_location)?;
        let store: BookmarkStore = serde_json::from_str(&data)?;
        Ok(store)
    }

    pub fn save(&mut self) -> Result<()> {
        let data = serde_json::to_string(&self)?;
        if !PROJECT_DIRS.save_location.exists() {
            fs::create_dir_all(
                PROJECT_DIRS
                    .save_location
                    .parent()
                    .expect("Root as save location"),
            )?
        }
        Ok(fs::write(&PROJECT_DIRS.save_location, data)?)
    }
}

// fn amend_possible_change(save_location: &PathBuf) -> Result<()> {
//     if PROJECT_DIRS.metadata.exists() {
//         let data = fs::read_to_string(&PROJECT_DIRS.metadata)?;
//         let metadata: Metadata = toml::from_str(&data)?;
//         if &metadata.prev_save_location != save_location {}
//     } else {
//         fs::create_dir_all(PROJECT_DIRS.metadata.parent().unwrap())?;
//         let metadata = Metadata { prev_save_location: PROJECT_DIRS.data_path.clone() };
//         let data = toml::to_string(&metadata)?;
//         fs::write(&PROJECT_DIRS.metadata, data)?;
//     }
//     Ok(())
// }
