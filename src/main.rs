use crate::errors::Result;
use chrono::{DateTime, Utc};
use clap::Parser;
use commands::{
    AddCommand, CLI, Category, ListArgs, ListFields, OpenArgs, RemoveArgs, SearchQuery, Status,
    Subcommands,
};
use directories::ProjectDirs;
use errors::Error;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};
use sublime_fuzzy::best_match;
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
        let data = fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&data)?)
    }

    fn save(&mut self) -> Result<()> {
        let path = get_data_file_path()?;
        let data = serde_json::to_string(&self)?;
        Ok(fs::write(&path, data)?)
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

        // Calculate rows
        for bookmark in &self.bookmarks {
            let mut rows = vec![bookmark.id.to_string(), bookmark.title.clone()];
            match args.fields {
                Some(ListFields::Urls) => {
                    rows.extend(vec![
                        bookmark
                            .url
                            .clone()
                            .map(|_url| "[XX]".to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    ]);
                }
                Some(ListFields::Notes) => {
                    rows.extend(vec![bookmark.notes.clone().unwrap_or("-".to_string())])
                }
                Some(ListFields::Hidden) | None => {
                    rows.extend(vec![bookmark.category.to_string(), bookmark.status.to_string()])
                }
            };
            builder.push_record(rows);
        }

        let mut table = builder.build();

        // Styling shite
        table
            .with(Style::modern())
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

    fn remove(&mut self, args: RemoveArgs) -> Result<()> {
        match args.id {
            SearchQuery::Id(id) => match self.bookmarks.iter().position(|e| e.id == id) {
                Some(id) => {
                    let title = &mut self.bookmarks[id].title.clone();
                    if title.len() > 24 {
                        title.truncate(21);
                        title.push_str("...");
                    }
                    let _ = self.bookmarks.remove(id);
                    println!("Successfully removed #{id} - {}", title);
                    self.normalize();
                }
                None => {
                    return Err(Error::IDNotFound(id));
                }
            },
            SearchQuery::Query(query) => {
                let id = fuzz(&query, &self.bookmarks);
                let title = &mut self.bookmarks[id].title.clone();
                if title.len() > 24 {
                    title.truncate(21);
                    title.push_str("...");
                }
                print!("Confirm removing '{}' from your bookmarks [y/n] ", title);
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim().to_lowercase();
                if input == "y" {
                    let _ = self.bookmarks.remove(id);
                    self.normalize();
                }
            }
        };
        self.save()?;
        Ok(())
    }

    fn open(&self, args: OpenArgs) -> Result<()> {
        match args.query {
            SearchQuery::Id(id) => match self.bookmarks.iter().find(|b| b.id == id) {
                Some(bookmark) => match &bookmark.url {
                    Some(url) => open::that(url)?,
                    None => return Err(Error::NoUrl(id)),
                },
                None => return Err(Error::IDNotFound(id)),
            },
            SearchQuery::Query(query) => {
                let id = fuzz(&query, &self.bookmarks);
                match &self.bookmarks[id].url {
                    Some(url) => open::that(url)?,
                    None => return Err(Error::NoUrl(id)),
                }
            }
        }
        Ok(())
    }

    fn normalize(&mut self) {
        for id in 0..self.bookmarks.len() {
            if self.bookmarks[id].id != id {
                self.bookmarks[id].id = id;
            }
        }
        self.next_id = self.bookmarks.len();
    }
}

fn fuzz<'a>(query: &str, store: &Vec<Bookmark>) -> usize {
    let (id, _) = store
        .iter()
        .filter_map(|i| best_match(query, &i.title))
        .enumerate()
        .max_by(|(id, a), (id2, b)| a.score().cmp(&b.score()))
        .unwrap();
    id
}

fn get_data_file_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("xyz", "arx", "offblck").ok_or(Error::NoProjectDirs)?;
    let data_dir = proj_dirs.data_dir();
    if !data_dir.exists() {
        fs::create_dir_all(data_dir)?;
    }
    Ok(data_dir.join("bookmarks.json"))
}

fn run() -> Result<()> {
    let cli = CLI::parse();
    let mut store = BookmarkStore::load()?;
    match cli.command {
        Subcommands::Add(args) => store.add(args)?,
        Subcommands::List(args) => store.list(args)?,
        Subcommands::Remove(query) => store.remove(query)?,
        Subcommands::Open(query) => store.open(query)?,
        Subcommands::CopyUrl(args) => {}
    }
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("[Error] {}", err);
        std::process::exit(1);
    }
}
