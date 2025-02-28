use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tabled::{
    Table, Tabled,
    builder::Builder,
    settings::{Alignment, Color, Style, object::FirstRow, style::HorizontalLine},
};

type Result<T> = core::result::Result<T, Error>;
type Error = Box<dyn std::error::Error>;

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
    url: Option<String>,
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
        Ok(
            fs::write(&path, data)
                .map_err(|e| format!("Failed to write bookmakrs to disk: {e}"))?,
        )
    }
    fn add(&mut self, args: AddCommand) -> Result<()> {
        let id = self.next_id;
        let new_bookmark = Bookmark {
            id,
            title: args.title,
            url: args.url,
        };
        self.bookmarks.push(new_bookmark);
        self.next_id += 1;
        self.save()?;
        Ok(())
    }
    fn list(&mut self) {
        let mut builder = Builder::default();
        builder.push_record(["ID", "Title"]);

        for (index, bookmark) in self.bookmarks.iter().enumerate() {
            builder.push_record([index.to_string(), bookmark.title.to_string()]);
        }

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

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
#[command(about = "A simple CLI bookmark tracker")]
struct CLI {
    #[command(subcommand)]
    command: Subcommands,
}

#[derive(Subcommand, Debug)]
enum Subcommands {
    Add(AddCommand),
    List,
}

#[derive(Parser, Debug)]
struct AddCommand {
    #[arg(help = "title of your bookmark")]
    title: String,

    #[arg(help = "add bookmark url")]
    url: Option<String>,
}

fn main() -> Result<()> {
    let cli = CLI::parse();
    let mut store = BookmarkStore::load()?;
    match cli.command {
        Subcommands::Add(args) => store.add(args)?,
        Subcommands::List => store.list(),
    }
    Ok(())
}
