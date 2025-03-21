use crate::errors::Result;
use chrono::{DateTime, Utc};
use clap::Parser;
use commands::{AddCommand, CLI, Category, ListArgs, Status, Subcommands};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tabled::{
    Table, Tabled,
    settings::{Alignment, Color, Style, object::FirstRow},
};

mod commands;
mod errors;

#[derive(Serialize, Deserialize, Default)]
struct BookmarkStore {
    next_id: usize,
    bookmarks: Vec<Bookmark>,
}

#[derive(Serialize, Deserialize, Tabled)]
struct Bookmark {
    id: usize,
    title: String,
    #[tabled(skip)]
    category: Category,
    #[tabled(display = "display")]
    url: Option<String>,
    #[tabled(display = "display_tags")]
    tags: Option<Vec<String>>,
    #[tabled(display = "display")]
    notes: Option<String>,
    status: Status,
    hidden: bool,
    created_at: DateTime<Utc>,
}

fn display_tags(tags: &Option<Vec<String>>) -> String {
    match tags {
        Some(tags) => tags.join(", "),
        None => "-".to_string(),
    }
}

fn display(option: &Option<String>) -> String {
    match option {
        Some(option) => option.clone(),
        None => "-".to_string(),
    }
}

impl BookmarkStore {
    fn load() -> Result<BookmarkStore> {
        let path = get_data_file_path()?;
        if !path.exists() {
            return Ok(BookmarkStore::default());
        }
        let data =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read bookmarks file: {e}"))?;
        Ok(serde_json::from_str(&data).map_err(|e| format!("Failed to parse bookmarks: {e}"))?)
    }

    fn save(&mut self) -> Result<()> {
        let path = get_data_file_path()?;
        let data = serde_json::to_string(&self)
            .map_err(|e| format!("Failed to serialize bookmarks: {e}"))?;
        Ok(fs::write(&path, data).map_err(|e| format!("Failed to write bookmakrs to disk: {e}"))?)
    }

    fn add(&mut self, args: AddCommand) -> Result<()> {
        let id = self.next_id;
        let new_bookmark = Bookmark {
            id,
            title: args.title,
            url: args.url,
            category: args.category.unwrap_or_default(),
            tags: args.tags,
            notes: args.notes,
            status: args.status.unwrap_or_default(),
            hidden: args.hidden,
            created_at: chrono::Utc::now(),
        };
        self.bookmarks.push(new_bookmark);
        self.next_id += 1;
        self.save()?;
        Ok(())
    }

    fn list(&mut self, args: ListArgs) {
        let mut table = Table::new(&self.bookmarks);
        table.with(Style::rounded());

        table
            .modify(FirstRow, Alignment::center())
            .modify(FirstRow, Color::BG_WHITE)
            .modify(FirstRow, Color::FG_RED);

        println!("{table}");
    }
}

fn get_data_file_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("xyz", "arx", "offblck")
        .ok_or_else(|| String::from("Could not determine app data directory on system"))?;
    let data_dir = proj_dirs.data_dir();
    if !data_dir.exists() {
        fs::create_dir_all(data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;
    }
    Ok(data_dir.join("bookmarks.json"))
}

fn run() -> Result<()> {
    let cli = CLI::parse();
    let mut store = BookmarkStore::load()?;
    match cli.command {
        Subcommands::Add(args) => store.add(args)?,
        Subcommands::List(args) => store.list(args),
        Subcommands::Remove(args) => {}
    }
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
