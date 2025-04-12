use crate::errors::Result;
use clap::Parser;
use comfy_table::Cell;
use command_types::{CLI, ListFields, Status, Subcommands};
use data::{Arx, BookmarkStore};
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
    let cli = CLI::parse();
    let mut arx = Arx::init()?;
    match cli.command {
        Subcommands::Add(args) => arx.store.add(args)?,
        Subcommands::List(args) => arx.store.list(args, &arx.config)?,
        Subcommands::Remove(query) => arx.store.remove(query)?,
        Subcommands::Edit(query) => arx.store.edit(query)?,
        Subcommands::Done(query) => arx.store.done(query)?,
        Subcommands::Open(query) => arx.store.open(query)?,
        Subcommands::CopyUrl(query) => arx.store.copy_url(query)?,
        Subcommands::Config(args) => arx.store.config(args, &mut arx.config)?,
    }
    Ok(())
}
