use crate::errors::Result;
use clap::Parser;
use comfy_table::Cell;
use command_types::{CLI, ListFields, Status, Subcommands};
use config::{init_project_dirs, load_config};
use data::BookmarkStore;
use errors::Error;

mod command_types;
mod commands;
mod config;
mod data;
mod errors;
mod utils;

fn main() {
    if let Err(err) = run() {
        eprintln!("[Error] {}", err);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    init_project_dirs()?;
    let config = load_config()?;
    let cli = CLI::parse();
    // WARNING: extraction will change w/ extended config
    let data_path = config.map(|config| config.save_location);
    let mut store = BookmarkStore::load(data_path)?;
    match cli.command {
        Subcommands::Add(args) => store.add(args)?,
        Subcommands::List(args) => store.list(args)?,
        Subcommands::Remove(query) => store.remove(query)?,
        Subcommands::Edit(query) => store.edit(query)?,
        Subcommands::Done(query) => store.done(query)?,
        Subcommands::Open(query) => store.open(query)?,
        Subcommands::CopyUrl(query) => store.copy_url(query)?,
    }
    Ok(())
}
