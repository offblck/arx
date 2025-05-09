use crate::command_types::{
    AddArgs, ConfigArgs, CopyUrlArgs, DoneArgs, EditArgs, ListArgs, OpenArgs, RemoveArgs,
    SearchQuery,
};
use crate::config::{Config, load_config};
use crate::data::{Arx, Bookmark};
use crate::{
    BookmarkStore, Cell, Error, ListFields, Status,
    errors::Result,
    utils::{copy, fuzz},
};
use comfy_table::{
    Attribute, CellAlignment, Color, ColumnConstraint, Table, Width, presets::UTF8_FULL,
};
use std::fs;
use std::io::{self, Write};
use terminal_link::Link;

impl Arx {
    pub fn init() -> Result<Arx> {
        let config = load_config()?;
        let store = BookmarkStore::load(&config)?;
        Ok(Arx { store, config })
    }
}

impl BookmarkStore {
    pub fn add(&mut self, args: AddArgs) -> Result<()> {
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
        println!("Bookmark with ID #{} successfully added!", self.next_id - 1);
        Ok(())
    }

    pub fn list(&mut self, mut args: ListArgs, config: &Config) -> Result<()> {
        if self.bookmarks.is_empty() {
            println!("You have no bookmarks yet...");
            return Ok(());
        }

        // filter if a field is specified, e.g. only entries with urls/notes/etc.
        self.filter_args(&mut args)?;

        let mut table = Table::new();
        table.load_preset(
            config
                .table_style
                .as_ref()
                .map(|s| s.to_comfy_style())
                .unwrap_or(UTF8_FULL),
        );

        // Initialize headers and calculate column widths
        let mut headers = vec![Cell::new("ID"), Cell::new("name")];
        let column_widths: Vec<(usize, usize)> = match args.fields {
            Some(ListFields::Urls) => {
                headers.push(Cell::new("url"));
                vec![(1, 65), (2, 6)]
            }
            Some(ListFields::Notes) => {
                headers.push(Cell::new("notes"));
                vec![(1, 21), (2, 50)]
            }
            Some(ListFields::Hidden) | None => {
                headers.extend(vec![Cell::new("category"), Cell::new("status")]);
                vec![(1, 50), (2, 10), (3, 10)]
            }
        };

        headers = headers
            .into_iter()
            .map(|cell| cell.fg(Color::Yellow).add_attribute(Attribute::Bold))
            .collect();

        // Build header row w/ column widths
        table.set_header(headers);
        let id_col = table.column_mut(0).expect("id column exists");
        id_col.set_constraint(ColumnConstraint::Absolute(Width::Fixed(5)));
        let name_col = table.column_mut(1).expect("name column exists");
        if args.fields != Some(ListFields::Notes) {
            name_col.set_cell_alignment(CellAlignment::Left);
        } else {
            name_col.set_cell_alignment(CellAlignment::Center);
        }
        for (index, width) in column_widths {
            let column = table.column_mut(index).expect("col should exist");
            column.set_constraint(ColumnConstraint::Absolute(Width::Fixed(width as u16)));
            if index > 1 {
                column.set_cell_alignment(CellAlignment::Center);
            }
        }

        // Calculate rows
        let page = match args.page {
            Some(0) => 1,
            Some(page) => page,
            None => 1,
        };
        let paginate_by = config.page_by.unwrap_or(10);
        if self.bookmarks.len() <= (page - 1) * paginate_by {
            return Err(Error::PageNotFound(page));
        }

        for bookmark in self
            .bookmarks
            .iter_mut()
            .skip((page - 1) * paginate_by)
            .take(page * paginate_by)
        {
            let mut row = vec![
                Cell::new(bookmark.id),
                if bookmark.status == Status::Pending {
                    Cell::new(&bookmark.title)
                        .fg(Color::White)
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Left)
                } else {
                    Cell::new(&bookmark.title).set_alignment(CellAlignment::Left)
                },
            ];
            match args.fields {
                Some(ListFields::Urls) => {
                    row.push(
                        Cell::new(
                            bookmark
                                .url
                                .clone()
                                .map(|_url| "[XX]".to_string())
                                .unwrap_or_else(|| "━━".to_string()),
                        )
                        .fg(Color::DarkBlue),
                    );
                }
                Some(ListFields::Notes) => row.push(
                    Cell::new(bookmark.notes.clone().unwrap_or("-".to_string()))
                        .set_alignment(CellAlignment::Left),
                ),
                Some(ListFields::Hidden) | None => row.extend(vec![
                    Cell::new(&bookmark.category).fg((&bookmark.category).into()),
                    if bookmark.status == Status::Done {
                        Cell::new(&bookmark.status).fg(Color::Green)
                    } else if bookmark.status == Status::Pending {
                        Cell::new(&bookmark.status).add_attribute(Attribute::Italic)
                    } else {
                        Cell::new(&bookmark.status)
                    },
                ]),
            };
            table.add_row(row);
        }

        let mut table = table.to_string();
        if args.fields == Some(ListFields::Urls) {
            let mut lines: Vec<String> = table.lines().map(String::from).collect();
            for (id, bookmark) in self.bookmarks.iter().enumerate() {
                if let Some(url) = &bookmark.url {
                    let link = Link::new("LINK", url).to_string();
                    let line_id = 3 + 2 * id;
                    lines[line_id] = lines[line_id].replace("[XX]", &link);
                }
            }
            table = lines.join("\n");
        }
        if table.lines().count() <= 3 {
            println!("No bookmarks found.");
            return Ok(());
        }

        println!("{table}");
        println!(
            "Showing page {page} out of {} (specify with -p <num>)",
            self.bookmarks.len().div_ceil(paginate_by)
        );
        Ok(())
    }

    pub fn remove(&mut self, args: RemoveArgs) -> Result<()> {
        for arg in args.list {
            match arg {
                SearchQuery::Id(id) => match self.bookmarks.iter().position(|e| e.id == id) {
                    Some(id) => {
                        let title = &mut self.bookmarks[id].title.clone();
                        if title.len() > 24 {
                            title.truncate(21);
                            title.push_str("...");
                        }
                        let _ = self.bookmarks.remove(id);
                        println!("Successfully removed #{id} - {}", title);
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
                    }
                }
            };
        }
        self.normalize();
        self.save()?;
        Ok(())
    }

    pub fn edit(&mut self, args: EditArgs) -> Result<()> {
        if args.category.is_none()
            && args.hidden.is_none()
            && args.notes.is_none()
            && args.status.is_none()
            && args.tags.is_none()
            && args.title.is_none()
            && args.url.is_none()
        {
            return Err(Error::NoEditSpecified);
        }
        let id = match args.query {
            SearchQuery::Id(id) => self
                .bookmarks
                .iter()
                .find_map(|b| if b.id == id { Some(b.id) } else { None })
                .ok_or(Error::IDNotFound(id))?,
            SearchQuery::Query(query) => fuzz(&query, &self.bookmarks),
        };

        let bookmark = &mut self.bookmarks[id];
        if let Some(category) = args.category {
            bookmark.category = category;
        } else if let Some(hidden) = args.hidden {
            bookmark.hidden = hidden;
        } else if let Some(notes) = args.notes {
            bookmark.notes = Some(notes);
        } else if let Some(status) = args.status {
            bookmark.status = status;
        } else if let Some(tags) = args.tags {
            bookmark.tags = Some(tags);
        } else if let Some(title) = args.title {
            bookmark.title = title;
        } else if let Some(url) = args.url {
            bookmark.url = Some(url);
        }
        self.save()?;
        Ok(())
    }

    pub fn done(&mut self, args: DoneArgs) -> Result<()> {
        match args.query {
            SearchQuery::Id(id) => match self.bookmarks.iter_mut().find(|b| b.id == id) {
                Some(bookmark) => bookmark.status = Status::Done,
                None => return Err(Error::IDNotFound(id)),
            },
            SearchQuery::Query(query) => {
                let id = fuzz(&query, &self.bookmarks);
                self.bookmarks[id].status = Status::Done;
            }
        };

        self.save()?;
        Ok(())
    }

    pub fn open(&self, args: OpenArgs) -> Result<()> {
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

    pub fn copy_url(&self, args: CopyUrlArgs) -> Result<()> {
        match args.query {
            SearchQuery::Id(id) => match self.bookmarks.iter().find(|b| b.id == id) {
                Some(bookmark) => match &bookmark.url {
                    Some(url) => copy(url.to_owned())?,
                    None => return Err(Error::NoUrl(id)),
                },
                None => return Err(Error::IDNotFound(id)),
            },
            SearchQuery::Query(query) => {
                let id = fuzz(&query, &self.bookmarks);
                match &self.bookmarks[id].url {
                    Some(url) => copy(url.to_owned())?,
                    None => return Err(Error::NoUrl(id)),
                }
            }
        }
        Ok(())
    }

    pub fn config(&self, args: ConfigArgs, config: &mut Config) -> Result<()> {
        args.validate()?;
        if let Some(mut path) = args.save_location {
            if path.is_dir() {
                path = path.join("bookmarks.json");
            }
            fs::rename(&config.save_location, &path)?;
            config.save_location = path;
        }
        if let Some(num) = args.page_by {
            if num == 0 {
                return Err(Error::ZeroPagination);
            }
            config.page_by = Some(num);
        }
        if let Some(style) = args.table_style {
            config.table_style = Some(style);
        }
        self.save_config(config)?;
        Ok(())
    }
}

mod utils {
    use crate::{
        command_types::{ListArgs, ListFields, Status},
        data::BookmarkStore,
        errors::Result,
    };

    impl BookmarkStore {
        pub fn filter_args(&mut self, args: &mut ListArgs) -> Result<()> {
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

            if !args.all {
                if let Some(ListFields::Hidden) = args.fields {
                    self.bookmarks.retain(|b| b.status != Status::Done);
                } else {
                    self.bookmarks
                        .retain(|b| b.status != Status::Done && !b.hidden);
                }
            }

            Ok(())
        }

        pub fn normalize(&mut self) {
            for id in 0..self.bookmarks.len() {
                if self.bookmarks[id].id != id {
                    self.bookmarks[id].id = id;
                }
            }
            self.next_id = self.bookmarks.len();
        }
    }
}
