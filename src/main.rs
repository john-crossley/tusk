use std::{
    io::{self, Error, Write},
    path::PathBuf,
};

use chrono::{NaiveDate, Utc};
use clap::{Parser, Subcommand};

mod models;
mod utils;

use crate::{
    models::{dayfile::DayFile, item::Item},
    utils::{
        dates::today_date,
        files::{load_or_create_dayfile, resolve_day_file_path, save_dayfile},
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
          value_parser = parse_ymd,          // <— custom parser
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
    #[arg(short, long)]
    verbose: bool,

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

    #[command(name = "done", about = "Mark an item done")]
    Done { id: String },
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
        Some(Commands::Done { id }) => run_done(&cli, id),
    }
}

// command handler functions

fn run_done(cli: &Cli, id: &str) -> io::Result<()> {
    let mut dayfile = get_dayfile(cli)?;

    if let Some(item) = dayfile.items.iter_mut().find(|i| i.id == id) {
        item.done_at = Some(Utc::now());

        let (_, path) = current_day_context(cli);
        save_dayfile(&path, &dayfile)?;

        run_ls(cli)
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("id not found '{}'", id),
        ))
    }
}

fn run_ls(cli: &Cli) -> io::Result<()> {
    let dayfile = get_dayfile(cli)?;
    render(&dayfile, cli)?;

    Ok(())
}

fn run_add(cli: &Cli, text: &str) -> Result<(), Error> {
    if text.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Oops, did you forget to add some text?",
        ));
    }

    let mut dayfile = get_dayfile(&cli)?;

    let next_idx: u32 = (dayfile.items.len() + 1)
        .try_into()
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "I wasn't built for this many items."))?;

    let new_item = Item::new(text.to_owned(), next_idx);

    dayfile.items.push(new_item);

    let (_, path) = current_day_context(cli);

    save_dayfile(&path, &dayfile)?;
    render(&dayfile, cli)?;

    Ok(())
}

// helper functions

fn render(dayfile: &DayFile, cli: &Cli) -> Result<(), Error> {
    let mut stdout = io::stdout().lock();

    if cli.json {
        serde_json::to_writer_pretty(&mut stdout, &dayfile)?;
        writeln!(&mut stdout)?;
    } else {
        if dayfile.items.is_empty() {
            writeln!(
                &mut stdout,
                "No tasks added for {}, try `tusk add \"My new task\"`",
                dayfile.date
            )?;
            return Ok(());
        }

        for (idx, i) in dayfile.items.iter().enumerate() {
            writeln!(
                &mut stdout,
                "{}) {} [{}]\t{}",
                idx + 1,
                i.id,
                item_completion_status(&i),
                i.text
            )?;
        }
    }

    stdout.flush()?;

    Ok(())
}

fn item_completion_status(i: &Item) -> &'static str {
    if i.done_at.is_none() { " " } else { "x" }
}

fn get_dayfile(cli: &Cli) -> Result<DayFile, Error> {
    let (date, path) = current_day_context(cli);
    let dayfile = load_or_create_dayfile(path.as_path(), date)?;

    Ok(dayfile)
}

fn parse_ymd(d: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(d, "%Y-%m-%d")
        .map_err(|_| format!("Invalid date '{d}'. Use YYYY-MM-DD, e.g. 2025-09-14"))
}

fn current_day_context(cli: &Cli) -> (NaiveDate, PathBuf) {
    let date = cli.date.unwrap_or_else(today_date);
    return (
        date,
        resolve_day_file_path(&date, cli.data_dir.as_deref(), cli.verbose),
    );
}
