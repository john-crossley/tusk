use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod models;

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
    #[arg(long, value_name = "FILE")]
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
    
    match cli.command.as_ref().unwrap_or(&Commands::Ls {}) {
        Commands::Ls {} => run_ls(&cli),
    }
}

fn run_ls(cli: &Cli) {
    // 1. Resolve date (use today if None)
    // 2. Resolve data_dir and date file path
    // 3. Load or create empty day file
    // 4. Render as human text or JSON depending on cli.json

    let date = cli.date.as_deref().unwrap_or("<today>");
    println!("(ls) date={date} json={} no_colour={} data_dir={:?}", cli.json, cli.no_colour, cli.data_dir)
}
