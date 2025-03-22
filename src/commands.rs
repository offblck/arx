use std::{fmt, str::FromStr};

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use tabled::Tabled;

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

    #[clap(alias = "ls")]
    List(ListArgs),

    #[clap(aliases = ["rm", "del", "delete"])]
    Remove(RemoveArgs),
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

#[derive(Debug, Clone, clap::ValueEnum, Serialize, Deserialize, Tabled, Default)]
pub enum Category {
    Book,
    Article,
    Topic,
    Project,
    Tool,
    #[default]
    Other,
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
    #[arg(required = true, help = "remove bookmark by id")]
    pub id: SearchQuery,
}

#[derive(Debug, Clone)]
pub enum SearchQuery {
    Id(u64),
    Query(String),
}

impl FromStr for SearchQuery {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try parsing as u64 first
        if let Ok(id) = s.parse::<u64>() {
            Ok(SearchQuery::Id(id))
        } else {
            // If not a number, treat as query string
            Ok(SearchQuery::Query(s.to_string()))
        }
    }
}
