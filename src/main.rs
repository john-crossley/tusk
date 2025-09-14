use std::{
    io::{self, Error, Write},
    path::PathBuf,
};

use chrono::NaiveDate;
use clap::{Parser, Subcommand};

mod models;
mod utils;

use crate::{
    models::{dayfile::DayFile, item::Item},
    utils::{
        dates::today_date,
        files::{load_or_create_dayfile, resolve_day_file_path},
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
}

fn main() {
    let cli = Cli::parse();

    match cli.command.as_ref() {
        Some(Commands::Ls {}) | None => match run_ls(&cli) {
            Ok(()) => {}
            Err(e) => eprintln!("Error: {}", e),
        },
    }
}

fn run_ls(cli: &Cli) -> Result<(), Error> {
    // 1. Resolve date (use today if None)
    // 2. Resolve data_dir and date file path
    // 3. Load or create empty day file
    // 4. Render as human text or JSON depending on cli.json

    // println!(
    //     "(ls) date={date} json={} no_colour={} data_dir={:?}",
    //     cli.json, cli.no_colour, cli.data_dir
    // )

    let dayfile = get_dayfile(cli)?;

    let mut stdout = io::stdout().lock();

    if cli.json {
        serde_json::to_writer_pretty(&mut stdout, &dayfile)?;
        writeln!(&mut stdout)?;
        stdout.flush()?;
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
                "{}) [{}]\t{}",
                idx,
                item_completion_status(&i),
                i.text
            )?;
        }

        stdout.flush()?;
    }

    Ok(())
}

fn item_completion_status(i: &Item) -> &'static str {
    if i.done_at.is_none() { " " } else { "x" }
}

fn get_dayfile(cli: &Cli) -> Result<DayFile, Error> {
    // let date = cli
    //     .date
    //     .as_deref()
    //     .map(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").expect("Use YYYY-MM-DD"))
    //     .unwrap_or_else(today_date);

    let date = cli.date.unwrap_or_else(today_date);

    let path = resolve_day_file_path(&date, cli.data_dir.as_deref(), cli.verbose);

    load_or_create_dayfile(path.as_path(), date)
}

fn parse_ymd(d: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(d, "%Y-%m-%d")
        .map_err(|_| format!("Invalid date '{d}'. Use YYYY-MM-DD, e.g. 2025-09-14"))
}
