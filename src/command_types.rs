use std::{fmt, path::PathBuf, str::FromStr};

use clap::{Parser, Subcommand};
use comfy_table::{Color, presets};
use serde::{Deserialize, Serialize};

use crate::errors::{Error, Result};

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
#[command(about = "A cli archive for all your bookmarks that you will totally read")]
pub struct CLI {
    #[command(subcommand)]
    pub command: Subcommands,
}

#[derive(Subcommand, Debug)]
pub enum Subcommands {
    #[clap(about = "add bookmark")]
    Add(AddArgs),

    #[clap(about = "list bookmarks (alias: ls)", alias = "ls")]
    List(ListArgs),

    #[clap(about = "remove bookmark (alias: rm, del, delete)", aliases = ["rm", "del", "delete"])]
    Remove(RemoveArgs),

    #[clap(about = "open bookmark url in browser")]
    Open(OpenArgs),

    #[clap(about = "edit bookmark")]
    Edit(EditArgs),

    #[clap(about = "mark bookmark as done")]
    Done(DoneArgs),

    #[clap(name = "copy-url", about = "copy bookmark url (alias: cp)", alias = "cp")]
    CopyUrl(CopyUrlArgs),

    #[clap(name = "config", about = "configure arx")]
    Config(ConfigArgs),
}

#[derive(Parser, Debug)]
pub struct AddArgs {
    #[arg(help = "title of your bookmark")]
    pub title: String,

    #[arg(short, long, help = "add bookmark url")]
    pub url: Option<String>,

    #[arg(short, long, help = "add bookmark category", value_enum)]
    pub category: Option<Category>,

    #[arg(short, long, num_args=1.., value_delimiter = ' ', help = "add bookmark tags")]
    pub tags: Option<Vec<String>>,

    #[arg(short, long, help = "add note to bookmark")]
    pub notes: Option<String>,

    #[arg(short, long, help = "add the current status of bookmark", value_enum)]
    pub status: Option<Status>,

    #[arg(long, help = "hide bookmark by default")]
    pub hidden: bool,
}

#[derive(Debug, Clone, clap::ValueEnum, Serialize, Deserialize, Default, PartialEq)]
pub enum Status {
    #[default]
    None,
    Pending,
    Done,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::None => write!(f, "━━"),
            Status::Pending => write!(f, "pending"),
            Status::Done => write!(f, "DONE"),
        }
    }
}

#[derive(Debug, Clone, clap::ValueEnum, Serialize, Deserialize, Default, PartialEq)]
pub enum Category {
    Book,
    Article,
    Topic,
    Project,
    Tool,
    Course,
    #[default]
    Other,
}

impl FromStr for Category {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "book" => Ok(Category::Book),
            "article" => Ok(Category::Article),
            "topic" => Ok(Category::Topic),
            "project" => Ok(Category::Project),
            "tool" => Ok(Category::Tool),
            "course" => Ok(Category::Course),
            _ => Err(Error::CategoryParseError(s.to_string())),
        }
    }
}

impl From<&Category> for Color {
    fn from(category: &Category) -> Self {
        match category {
            Category::Book => Color::Green,
            Category::Article => Color::Magenta,
            Category::Topic => Color::Blue,
            Category::Project => Color::Red,
            Category::Tool => Color::AnsiValue(5),
            Category::Course => Color::Yellow,
            Category::Other => Color::White,
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::Book => write!(f, "book"),
            Category::Article => write!(f, "article"),
            Category::Topic => write!(f, "topic"),
            Category::Project => write!(f, "project"),
            Category::Tool => write!(f, "tool"),
            Category::Course => write!(f, "course"),
            Category::Other => write!(f, "other"),
        }
    }
}

#[derive(Debug, Clone, clap::ValueEnum, PartialEq)]
pub enum ListFields {
    Urls,
    Notes,
    Hidden,
}

#[derive(Parser, Debug)]
pub struct ListArgs {
    #[arg(short, long, help = "filter by category")]
    pub category: Option<String>,

    #[arg(short, long, help = "filter by tag")]
    pub tag: Option<String>,

    #[arg(short, long, help = "set page to show")]
    pub page: Option<usize>,

    #[arg(short, long, help = "show all flags (e.g. done/hidden too)")]
    pub all: bool,

    #[arg(value_enum)]
    pub fields: Option<ListFields>,
}

#[derive(Parser, Debug)]
pub struct RemoveArgs {
    #[arg(
        required = true,
        help = "remove bookmark by ID or fuzzy search query e.g. '123' or 'my query'",
        value_name = "ID | query"
    )]
    pub list: Vec<SearchQuery>,
}

#[derive(Debug, Clone)]
pub enum SearchQuery {
    Id(usize),
    Query(String),
}

impl FromStr for SearchQuery {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        // Try parsing as u64 first
        if let Ok(id) = s.parse::<usize>() {
            Ok(SearchQuery::Id(id))
        } else {
            // If not a number, treat as query string
            Ok(SearchQuery::Query(s.to_string()))
        }
    }
}

#[derive(Parser, Debug)]
pub struct EditArgs {
    #[arg(
        required = true,
        help = "edit bookmark data",
        long_help = "edit bookmark data by id or fuzzy search query e.g. '123' or 'my query'",
        value_name = "ID | query"
    )]
    pub query: SearchQuery,

    #[arg(long, short)]
    pub title: Option<String>,
    #[arg(long, short)]
    pub url: Option<String>,
    #[arg(long, short)]
    pub notes: Option<String>,
    #[arg(long, short)]
    pub category: Option<Category>,
    #[arg(long, short)]
    pub status: Option<Status>,
    #[arg(long)]
    pub hidden: Option<bool>,
    #[arg(long)]
    pub tags: Option<Vec<String>>,
}

#[derive(Parser, Debug)]
pub struct DoneArgs {
    #[arg(
        required = true,
        help = "mark bookmark as done",
        long_help = "mark bookmark as done by id or fuzzy search query e.g. '123' or 'my query'",
        value_name = "ID | query"
    )]
    pub query: SearchQuery,
}

#[derive(Parser, Debug)]
pub struct OpenArgs {
    #[arg(
        required = true,
        help = "open bookmark url by id or fuzzy search query e.g. '123' or 'my query'",
        value_name = "ID | query"
    )]
    pub query: SearchQuery,
}

#[derive(Parser, Debug)]
pub struct CopyUrlArgs {
    #[arg(
        required = true,
        help = "copy url to clipboard",
        long_help = "copy bookmark url to clipboard / display url for old terminals, specified by id or fuzzy search query e.g. '123' or 'my query'",
        value_name = "ID | query"
    )]
    pub query: SearchQuery,
}

#[derive(Clone, clap::ValueEnum, Debug, Deserialize, Serialize)]
#[clap(rename_all = "snake_case")]
pub enum TableStyle {
    AsciiFull,
    AsciiFullCondensed,
    AsciiNoBorders,
    AsciiBordersOnly,
    AsciiBordersOnlyCondensed,
    AsciiHorizontalOnly,
    AsciiMarkdown,
    Utf8Full,
    Utf8FullCondensed,
    Utf8NoBorders,
    Utf8BordersOnly,
    Utf8HorizontalOnly,
    Nothing,
}

impl TableStyle {
    pub fn to_comfy_style(&self) -> &'static str {
        match self {
            TableStyle::AsciiFull => presets::ASCII_FULL,
            TableStyle::AsciiFullCondensed => presets::ASCII_FULL_CONDENSED,
            TableStyle::AsciiNoBorders => presets::ASCII_NO_BORDERS,
            TableStyle::AsciiBordersOnly => presets::ASCII_BORDERS_ONLY,
            TableStyle::AsciiBordersOnlyCondensed => presets::ASCII_BORDERS_ONLY_CONDENSED,
            TableStyle::AsciiHorizontalOnly => presets::ASCII_HORIZONTAL_ONLY,
            TableStyle::AsciiMarkdown => presets::ASCII_MARKDOWN,
            TableStyle::Utf8Full => presets::UTF8_FULL,
            TableStyle::Utf8FullCondensed => presets::UTF8_FULL_CONDENSED,
            TableStyle::Utf8NoBorders => presets::UTF8_NO_BORDERS,
            TableStyle::Utf8BordersOnly => presets::UTF8_BORDERS_ONLY,
            TableStyle::Utf8HorizontalOnly => presets::UTF8_HORIZONTAL_ONLY,
            TableStyle::Nothing => presets::NOTHING,
        }
    }
}

#[derive(Parser, Debug)]
#[clap(group(
    clap::ArgGroup::new("config-args")
        .required(true)
        .args(&["save-location", "table-style", "page-by"])
))]
pub struct ConfigArgs {
    #[arg(long, short, name = "save-location")]
    pub save_location: Option<PathBuf>,

    #[arg(long, short, name = "table-style")]
    pub table_style: Option<TableStyle>,

    #[arg(long, short, name = "page-by")]
    pub page_by: Option<usize>,
}

impl ConfigArgs {
    pub fn validate(&self) -> Result<()> {
        if self.save_location.is_none() && self.table_style.is_none() && self.page_by.is_none() {
            return Err(Error::NoConfigArgs);
        }
        Ok(())
    }
}
