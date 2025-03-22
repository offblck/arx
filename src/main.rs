use crate::errors::Result;
use chrono::{DateTime, Utc};
use clap::Parser;
use commands::{AddCommand, CLI, Category, ListArgs, ListFields, Status, Subcommands};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tabled::{
    Tabled,
    builder::Builder,
    settings::{
        Alignment, Color, Padding, Style, Width,
        formatting::TrimStrategy,
        object::{Columns, FirstRow, Object, Rows},
        width::MinWidth,
    },
};
use terminal_link::Link;

mod commands;
mod errors;

#[derive(Serialize, Deserialize, Default)]
struct BookmarkStore {
    next_id: usize,
    bookmarks: Vec<Bookmark>,
}

#[derive(Serialize, Deserialize)]
struct Bookmark {
    id: usize,
    title: String,
    category: Category,
    url: Option<String>,
    tags: Option<Vec<String>>,
    notes: Option<String>,
    status: Status,
    hidden: bool,
    created_at: DateTime<Utc>,
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

    fn list(&mut self, mut args: ListArgs) -> Result<()> {
        if self.bookmarks.is_empty() {
            println!("You have no bookmarks yet...");
            return Ok(());
        }

        self.filter_args(&mut args)?;

        let mut builder = Builder::default();

        // Initialize headers and calculate column widths
        let default_headers = vec!["id".to_string(), "name".to_string()];
        let (headers, column_widths): (Vec<String>, Vec<(usize, usize)>) = match args.fields {
            Some(ListFields::Urls) => {
                // Headers: id, name, url
                let mut headers = default_headers;
                headers.extend(vec!["url".to_string()]);
                let widths = vec![(1, 63), (2, 4)];
                (headers, widths)
            }
            Some(ListFields::Notes) => {
                let mut headers = default_headers;
                headers.extend(vec!["notes".to_string()]);
                // Widths: name (index 1) = 50
                let widths = vec![(1, 19), (2, 48)];
                (headers, widths)
            }
            Some(ListFields::Hidden) | None => {
                let mut headers = default_headers;
                headers.extend(vec!["category".to_string(), "status".to_string()]);
                let widths = vec![(1, 48), (2, 8), (3, 8)];
                (headers, widths)
            }
        };
        builder.push_record(headers.clone());

        for bookmark in &self.bookmarks {
            let row = match args.fields {
                Some(ListFields::Urls) => {
                    vec![
                        bookmark.id.to_string(),
                        bookmark.title.clone(),
                        bookmark
                            .url
                            .clone()
                            .map(|_url| "[XX]".to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    ]
                }
                Some(ListFields::Notes) => vec![
                    bookmark.id.to_string(),
                    bookmark.title.clone(),
                    bookmark.notes.clone().unwrap_or("-".to_string()),
                ],
                Some(ListFields::Hidden) | None => vec![
                    bookmark.id.to_string(),
                    bookmark.title.clone(),
                    bookmark.category.to_string(),
                    bookmark.status.to_string(),
                ],
            };
            builder.push_record(row);
        }

        let mut table = builder.build();

        table
            .with(Style::ascii())
            .with(TrimStrategy::Horizontal)
            .with(Alignment::center());

        table
            .modify(Columns::single(0), Padding::zero())
            .modify(Columns::single(0), MinWidth::new(5))
            .modify(Columns::single(1).intersect(Rows::new(1..)), Alignment::left())
            .modify(FirstRow, Color::BG_WHITE)
            .modify(FirstRow, Color::FG_RED);

        for (index, width) in column_widths {
            table.modify(Columns::single(index), MinWidth::new(width));
            // if index == 1 {
            table.modify(Columns::single(index), Width::truncate(width));
            // } else {
            //     table.modify(Columns::single(index), Width::wrap(width));
            // }
        }
        let mut table = table.to_string();
        if args.fields == Some(ListFields::Urls) {
            for bookmark in &self.bookmarks {
                if let Some(url) = &bookmark.url {
                    let link = Link::new("LINK", url).to_string();
                    table = table.replace("[XX]", &link);
                }
            }
        }

        println!("{table}");
        Ok(())
    }

    fn filter_args(&mut self, args: &mut ListArgs) -> Result<()> {
        match args.fields {
            Some(ListFields::Urls) => self.bookmarks.retain(|b| b.url.is_some()),
            Some(ListFields::Notes) => self.bookmarks.retain(|b| b.notes.is_some()),
            Some(ListFields::Hidden) => self.bookmarks.retain(|b| b.hidden),
            None => {}
        }

        if let Some(category) = &args.category {
            let category = category.parse()?;
            self.bookmarks.retain(|b| b.category == category);
        }

        if let Some(tag) = &args.tag {
            self.bookmarks.retain(|b| b.tags.is_some());
            self.bookmarks
                .retain(|b| b.tags.as_ref().unwrap().contains(tag));
        }

        Ok(())
    }
}

fn get_data_file_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("xyz", "arx", "offblck")
        .ok_or_else(|| "Could not determine app data directory on system")?;
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
        Subcommands::List(args) => store.list(args)?,
        Subcommands::Remove(args) => {}
        Subcommands::Open(args) => {}
        Subcommands::CopyUrl(args) => {}
    }
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
