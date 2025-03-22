use std::{fmt, str::FromStr};

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use tabled::Tabled;
use thiserror::Error;

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
#[command(about = "A simple CLI bookmark tracker")]
pub struct CLI {
    #[command(subcommand)]
    pub command: Subcommands,
}

#[derive(Subcommand, Debug)]
pub enum Subcommands {
    Add(AddCommand),

    #[clap(about = "list bookmarks (alias: ls)", alias = "ls")]
    List(ListArgs),

    #[clap(about = "remove bookmark (alias: rm, del, delete)", aliases = ["rm", "del", "delete"])]
    Remove(RemoveArgs),

    #[clap(about = "open bookmark url in browser")]
    Open(OpenArgs),

    #[clap(name = "copy-url", about = "copy bookmark url (alias: cp)", alias = "cp")]
    CopyUrl(CopyUrlArgs),
}

#[derive(Parser, Debug)]
pub struct AddCommand {
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

#[derive(Debug, Clone, clap::ValueEnum, Serialize, Deserialize, Tabled, Default)]
pub enum Status {
    #[default]
    None,
    Pending,
    Done,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::None => write!(f, "-"),
            Status::Pending => write!(f, "pending"),
            Status::Done => write!(f, "done"),
        }
    }
}

#[derive(Debug, Clone, clap::ValueEnum, Serialize, Deserialize, Tabled, Default, PartialEq)]
pub enum Category {
    Book,
    Article,
    Topic,
    Project,
    Tool,
    #[default]
    Other,
}

#[derive(Error, Debug)]
#[error("Invalid category: {0}")]
pub struct CategoryParseError(String);

impl FromStr for Category {
    type Err = CategoryParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "book" => Ok(Category::Book),
            "article" => Ok(Category::Article),
            "topic" => Ok(Category::Topic),
            "project" => Ok(Category::Project),
            "tool" => Ok(Category::Tool),
            _ => Err(CategoryParseError(s.to_string())),
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Category::Book => write!(f, "Book"),
            Category::Article => write!(f, "Article"),
            Category::Topic => write!(f, "Topic"),
            Category::Project => write!(f, "Project"),
            Category::Tool => write!(f, "Tool"),
            Category::Other => write!(f, "Other"),
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
    pub id: SearchQuery,
}

#[derive(Debug, Clone)]
pub enum SearchQuery {
    Id(usize),
    Query(String),
}

impl FromStr for SearchQuery {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
        help = "copy url by id or fuzzy search query e.g. '123' or 'my query'",
        value_name = "ID | query"
    )]
    pub query: SearchQuery,
}
