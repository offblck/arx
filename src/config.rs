use std::{cell::LazyCell, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::errors::{Error, Result};

pub const PROJECT_DIRS: LazyCell<ProjectDirs> =
    LazyCell::new(|| match directories::ProjectDirs::from("dev", "offblck", "arx") {
        Some(proj_dirs) => ProjectDirs {
            data_path: proj_dirs.data_dir().join("bookmarks.json"),
            config_path: proj_dirs.config_dir().join("config.toml"),
            metadata: proj_dirs.data_dir().join(".metadata.toml"),
            error: false,
        },
        None => ProjectDirs { error: true, ..ProjectDirs::default() },
    });

#[derive(Default)]
pub struct ProjectDirs {
    pub data_path: PathBuf,
    pub config_path: PathBuf,
    pub metadata: PathBuf,
    pub error: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub save_location: PathBuf,
}

pub fn init_project_dirs() -> Result<()> {
    // first access to LazyCell inits PROJECT_DIRS
    match PROJECT_DIRS.error {
        true => Err(Error::NoProjectDirs),
        false => Ok(()),
    }
}

pub fn load_config() -> Result<Option<Config>> {
    match PROJECT_DIRS.config_path.exists() {
        true => {
            let data = fs::read_to_string(&PROJECT_DIRS.config_path)?;
            let config = toml::from_str(&data)?;
            Ok(Some(config))
        }
        false => Ok(None),
    }
}
