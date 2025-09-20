use std::{
    io::{self, Error},
    path::PathBuf,
};

use chrono::{NaiveDate, Utc};
use clap::{Parser, Subcommand};

mod models;
mod utils;

use crate::{
    models::item::{Item, ItemPriority},
    utils::{
        dates::today_date,
        editor::edit_in_editor,
        files::{load_or_create_dayfile, resolve_day_file_path, save_dayfile},
        render::{RenderOpts, render, render_summary},
    },
};

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Tusk - simple daily todos in your terminal",
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
    Ls {
        /// Filter tasks by one or more tags
        #[arg(long = "tag", num_args = 1..)]
        tags: Option<Vec<String>>,
    },

    #[command(name = "add", about = "Add a new item to your day")]
    Add {
        /// The description of the item being added.
        text: String,

        /// The priority of the item being added.
        #[arg(short = 'p', long = "priority")]
        priority: Option<String>,

        /// Add a note to this item, opens in an external editor
        #[arg(short = 'n', long = "note")]
        note: bool,
    },

    #[command(name = "done", about = "Mark an item done by its index")]
    Done { index: usize },

    #[command(name = "undone", about = "Mark an item undone by its index")]
    Undone { index: usize },

    #[command(name = "rm", about = "Remove an item from your list.")]
    Rm { index: usize },

    #[command(name = "edit", about = "Edit an item from your list.")]
    Edit { index: usize, text: String },

    #[command(name = "show", about = "Show an item by its index.")]
    Show { index: usize },
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
        Some(Commands::Add {
            text,
            priority,
            note,
        }) => run_add(&cli, text, priority.as_deref(), *note),
        Some(Commands::Ls { tags }) => run_ls(&cli, tags),
        Some(Commands::Done { index }) => run_done(&cli, *index, true),
        Some(Commands::Undone { index }) => run_done(&cli, *index, false),
        Some(Commands::Rm { index }) => run_rm(&cli, *index),
        Some(Commands::Edit { index, text }) => run_edit(&cli, *index, text),
        Some(Commands::Show { index }) => run_show(&cli, *index),
        None => run_ls(&cli, &None),
    }
}

// command handler functions

fn run_add(cli: &Cli, text: &str, priority: Option<&str>, note: bool) -> Result<(), Error> {
    let new_text = sanitise_str(text)?;
    let tags = extract_tags(text);

    let (date, path) = current_day_context(cli)?;
    let mut dayfile = load_or_create_dayfile(&path, date)?;

    let next_idx: u32 = (dayfile.items.len() + 1).try_into().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "I wasn't built for this many items.",
        )
    })?;

    let notes = if note {
        Some(edit_in_editor("# Notes")?)
    } else {
        None
    };

    dayfile.items.push(Item::new(
        new_text,
        get_item_priority(priority),
        tags,
        next_idx,
        notes,
    ));

    save_dayfile(&path, &dayfile)?;

    if let Some(item) = dayfile.items.last() {
        render_summary(
            item,
            RenderOpts {
                json: cli.json,
                verbose: cli.verbose,
                no_color: cli.no_colour,
                vault_name: None,
            },
        )?;
    }

    Ok(())
}

fn run_ls(cli: &Cli, tags: &Option<Vec<String>>) -> io::Result<()> {
    let (date, path) = current_day_context(cli)?;
    let mut dayfile = load_or_create_dayfile(&path, date)?;

    if let Some(tags) = tags {
        dayfile.items.retain(|i| {
            tags.iter()
                .all(|t| i.tags.iter().any(|it| it.eq_ignore_ascii_case(t)))
        });
    }

    render(
        &dayfile,
        RenderOpts {
            json: cli.json,
            verbose: cli.verbose,
            no_color: cli.no_colour,
            vault_name: None,
        },
    )?;

    Ok(())
}

fn run_done(cli: &Cli, idx: usize, mark_done: bool) -> io::Result<()> {
    let (date, path) = current_day_context(cli)?;
    let mut dayfile = load_or_create_dayfile(&path, date)?;

    let pos = validate_index(idx, dayfile.items.len())?;
    let item = &mut dayfile.items[pos];
    item.done_at = if mark_done {
        item.done_at.take().or(Some(Utc::now()))
    } else {
        None
    };
    save_dayfile(&path, &dayfile)?;

    Ok(())
}

fn run_rm(cli: &Cli, idx: usize) -> io::Result<()> {
    let (date, path) = current_day_context(cli)?;
    let mut dayfile = load_or_create_dayfile(&path, date)?;

    let pos = validate_index(idx, dayfile.items.len())?;
    let _ = &mut dayfile.items.remove(pos);
    save_dayfile(&path, &dayfile)?;

    Ok(())
}

fn run_edit(cli: &Cli, idx: usize, text: &str) -> io::Result<()> {
    let new_text = sanitise_str(text)?;
    let (date, path) = current_day_context(cli)?;
    let mut dayfile = load_or_create_dayfile(&path, date)?;

    let pos = validate_index(idx, dayfile.items.len())?;

    if let Some(item) = dayfile.items.get_mut(pos) {
        item.text = new_text;
        save_dayfile(&path, &dayfile)?;
    }

    Ok(())
}

fn run_show(cli: &Cli, idx: usize) -> io::Result<()> {
    let (date, path) = current_day_context(cli)?;
    let mut dayfile = load_or_create_dayfile(&path, date)?;
    let pos = validate_index(idx, dayfile.items.len())?;

    if let Some(item) = dayfile.items.get_mut(pos) {
        render_summary(
            item,
            RenderOpts {
                json: cli.json,
                verbose: cli.verbose,
                no_color: cli.no_colour,
                vault_name: None,
            },
        )?;
    }

    Ok(())
}

// helper functions

fn validate_index(i: usize, len: usize) -> io::Result<usize> {
    if i == 0 || i > len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("item at index {} does not exist.", i),
        ));
    }

    Ok(i - 1)
}

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

fn sanitise_str(text: &str) -> io::Result<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Oops, did you forget to add some text?",
        ))
    } else {
        Ok(trimmed.to_owned())
    }
}

fn get_item_priority(priority: Option<&str>) -> ItemPriority {
    // if let Some(priority) = priority {
    //     match priority.to_lowercase().as_str() {
    //         "high" => ItemPriority::High,
    //         "med" | "medium" => ItemPriority::Medium,
    //         _ => ItemPriority::Low
    //     }
    // } else {
    //     ItemPriority::Low
    // }

    match priority.map(|p| p.to_lowercase()) {
        Some(ref p) if p == "high" => ItemPriority::High,
        Some(ref p) if p == "med" || p == "medium" => ItemPriority::Medium,
        _ => ItemPriority::Low,
    }
}

fn extract_tags(s: &str) -> Vec<String> {
    s.split_whitespace()
        .filter_map(|w| w.strip_prefix('#').map(|t| t.to_string()))
        .collect()
}
