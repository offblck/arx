use std::{fs, path::PathBuf, sync::LazyLock};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::{
    command_types::TableStyle,
    errors::{Error, Result},
};

#[derive(Default)]
pub struct DefaultDirs {
    pub save_location: PathBuf,
    pub config_path: PathBuf,
    error: bool,
}

pub static PROJECT_DIRS: LazyLock<DefaultDirs> =
    LazyLock::new(|| match ProjectDirs::from("dev", "offblck", "arx") {
        Some(dirs) => DefaultDirs {
            save_location: dirs.data_dir().join("bookmarks.json"),
            config_path: dirs.config_dir().join("config.toml"),
            error: false,
        },
        None => DefaultDirs { error: true, ..DefaultDirs::default() },
    });

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default = "default_save_location")]
    pub save_location: PathBuf,

    pub table_style: Option<TableStyle>,

    pub page_by: Option<usize>,
}

pub fn load_config() -> Result<Config> {
    let default_dirs = LazyLock::force(&PROJECT_DIRS);
    if default_dirs.error {
        return Err(Error::NoProjectDirs);
    }
    match default_dirs.config_path.exists() {
        true => {
            let data = fs::read_to_string(&default_dirs.config_path)?;
            let config: Config = toml::from_str(&data)?;
            Ok(config)
        }
        false => Ok(Config {
            save_location: default_dirs.save_location.clone(),
            ..Config::default()
        }),
    }
}

pub fn default_save_location() -> PathBuf {
    PROJECT_DIRS.save_location.clone()
}
