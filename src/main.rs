use std::path::PathBuf;

use chrono::NaiveDate;
use clap::{Parser, Subcommand};

mod models;
mod utils;

use crate::utils::{dates::today_date, files::{load_or_create_dayfile, resolve_day_file_path}};

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
    #[arg(short, long)]
    date: Option<String>,

    /// Override the base data directory (default: platform-specific app data dir).
    #[arg(long, value_name = "DIR")]
    data_dir: Option<PathBuf>,

    /// Output results as JSON instead of human-readable text.
    #[arg(short, long)]
    json: bool,

    /// Disable coloured output (useful in scripts or non-TTY environments).
    #[arg(short, long)]
    no_colour: bool,

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
        Some(Commands::Ls {}) | None => run_ls(&cli),
    }
}

fn run_ls(cli: &Cli) {
    // 1. Resolve date (use today if None)
    // 2. Resolve data_dir and date file path
    // 3. Load or create empty day file
    // 4. Render as human text or JSON depending on cli.json

    let date = cli
        .date
        .as_deref()
        .map(|s| NaiveDate::parse_from_str(s, "%Y-%n-%d").expect("Use YYYY-MM-DD"))
        .unwrap_or_else(today_date);

    let path = resolve_day_file_path(&date, cli.data_dir.as_deref());
    load_or_create_dayfile(path.as_path(), date);

    // println!("{}", day_file.display());

    println!(
        "(ls) date={date} json={} no_colour={} data_dir={:?}",
        cli.json, cli.no_colour, cli.data_dir
    )
}