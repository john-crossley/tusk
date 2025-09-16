use std::{
    io::{self, Error},
    path::PathBuf,
};

use chrono::{NaiveDate, Utc};
use clap::{Parser, Subcommand};

mod models;
mod utils;

use crate::{
    models::item::Item,
    utils::{
        dates::today_date,
        files::{load_or_create_dayfile, resolve_day_file_path, save_dayfile},
        render::render,
    },
};

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Tusk – simple daily todos in your terminal",
    long_about = "Tusk is a lightweight CLI that stores each day's todos in a JSON file. \
                  Add tasks, list them, mark them as done, and export to Markdown with zero friction.",
    subcommand = "ls"
)]
struct Cli {
    /// Target date (YYYY-MM-DD). Defaults to today if omitted.
    #[arg(short, long,
          value_parser = parse_ymd,
          value_name = "YYYY-MM-DD")]
    date: Option<NaiveDate>,

    /// Override the base data directory (default: platform-specific app data dir).
    #[arg(long, value_name = "DIR")]
    data_dir: Option<PathBuf>,

    /// Output results as JSON instead of human-readable text.
    #[arg(short, long)]
    json: bool,

    /// Disable coloured output (useful in scripts or non-TTY environments).
    #[arg(short, long)]
    no_colour: bool,

    /// Enables verbose logging, useful for debugging.
    #[arg(long)]
    verbose: bool,

    /// Specifies which vault to operate in.
    #[arg(short, long)]
    vault: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}
#[derive(Subcommand, Debug)]
enum Commands {
    #[command(name = "ls", about = "List items for the target date")]
    Ls {},

    #[command(name = "add", about = "Add a new item to your day")]
    Add {
        /// The description of the item being added.
        text: String,
    },

    #[command(name = "done", about = "Mark an item done by its index")]
    Done { index: usize },
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = dispatch(&cli) {
        eprintln!("Tusk: {e}");
        std::process::exit(1);
    }
}

fn dispatch(cli: &Cli) -> io::Result<()> {
    match cli.command.as_ref() {
        Some(Commands::Add { text }) => run_add(&cli, text),
        Some(Commands::Ls {}) | None => run_ls(&cli),
        Some(Commands::Done { index }) => run_done(&cli, *index),
    }
}

// command handler functions

fn run_done(cli: &Cli, idx: usize) -> io::Result<()> {
    let (date, path) = current_day_context(cli)?;
    let mut dayfile = load_or_create_dayfile(&path, date)?;

    if idx > 0
        && let Some(item) = dayfile.items.get_mut(idx - 1)
    {
        item.done_at = item.done_at.take().or(Some(Utc::now()));

        save_dayfile(&path, &dayfile)?;

        run_ls(cli)
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("index not found '{}'", idx),
        ))
    }
}

fn run_ls(cli: &Cli) -> io::Result<()> {
    let (date, path) = current_day_context(cli)?;
    let dayfile = load_or_create_dayfile(&path, date)?;
    render(&dayfile, cli.json, cli.verbose)?;

    Ok(())
}

fn run_add(cli: &Cli, text: &str) -> Result<(), Error> {
    if text.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Oops, did you forget to add some text?",
        ));
    }

    let (date, path) = current_day_context(cli)?;
    let mut dayfile = load_or_create_dayfile(&path, date)?;

    let next_idx: u32 = (dayfile.items.len() + 1)
        .try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "I wasn't built for this many items."))?;

    let new_item = Item::new(text.to_owned(), next_idx);

    dayfile.items.push(new_item);

    save_dayfile(&path, &dayfile)?;
    render(&dayfile, cli.json, cli.verbose)?;

    Ok(())
}

// helper functions

fn parse_ymd(d: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(d, "%Y-%m-%d")
        .map_err(|_| format!("Invalid date '{d}'. Use YYYY-MM-DD, e.g. 2025-09-14"))
}

fn current_day_context(cli: &Cli) -> Result<(NaiveDate, PathBuf), Error> {
    let date = cli.date.unwrap_or_else(today_date);

    let path = resolve_day_file_path(
        &date,
        cli.data_dir.as_deref(),
        cli.verbose,
        cli.vault.as_deref(),
    )?;

    return Ok((date, path));
}
