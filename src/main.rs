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
        dates::{parse_ymd, today_date},
        editor::edit_in_editor,
        files::{load_or_create_dayfile, resolve_day_file_path, save_dayfile},
        render::{RenderOpts, render, render_migrate, render_summary},
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
    #[arg(short, long, value_parser = parse_ymd, value_name = "YYYY-MM-DD", global = true)]
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
        #[arg(short = 'n', long = "notes")]
        attach_notes: bool,
    },

    #[command(name = "done", about = "Mark an item done by its index")]
    Done { index: usize },

    #[command(name = "undone", about = "Mark an item undone by its index")]
    Undone { index: usize },

    #[command(name = "rm", about = "Remove an item from your list.")]
    Rm { index: usize },

    #[command(name = "edit", about = "Edit an item from your list.")]
    Edit {
        index: usize,
        text: Option<String>,
        /// Add a note to this item, opens in an external editor
        #[arg(short = 'n', long = "notes")]
        attach_notes: bool,
    },

    #[command(name = "show", about = "Show an item by its index.")]
    Show { index: usize },

    #[command(
        name = "migrate",
        about = "Migrate undone items from one date to another."
    )]
    Migrate {
        #[arg(name = "from", short, long, value_parser = parse_ymd, value_name = "YYYY-MM-DD")]
        from_date: Option<NaiveDate>,

        #[arg(name = "to", short, long, value_parser = parse_ymd, value_name = "YYYY-MM-DD")]
        to_date: Option<NaiveDate>,

        /// Perform a dry run to show you what changes will be made.
        #[arg(long = "dry-run")]
        dry_run: bool,
    },
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
            attach_notes,
        }) => run_add(&cli, text, priority.as_deref(), attach_notes),
        Some(Commands::Ls { tags }) => run_ls(&cli, tags),
        Some(Commands::Done { index }) => run_done(&cli, *index, true),
        Some(Commands::Undone { index }) => run_done(&cli, *index, false),
        Some(Commands::Rm { index }) => run_rm(&cli, *index),
        Some(Commands::Edit {
            index,
            text,
            attach_notes,
        }) => run_edit(&cli, *index, text, attach_notes),
        Some(Commands::Show { index }) => run_show(&cli, *index),
        Some(Commands::Migrate {
            from_date,
            to_date,
            dry_run,
        }) => run_migrate(&cli, from_date, to_date, dry_run),
        None => run_ls(&cli, &None),
    }
}

// command handler functions

fn run_add(
    cli: &Cli,
    text: &str,
    priority: Option<&str>,
    attach_notes: &bool,
) -> Result<(), Error> {
    let new_text = sanitise_str(text)?;
    let tags = extract_tags(text);

    let (date, path) = current_day_context(cli)?;
    let mut dayfile = load_or_create_dayfile(&path, date)?;

    let notes = if *attach_notes {
        Some(edit_in_editor("# Notes")?)
    } else {
        None
    };

    dayfile.items.push(Item::new(
        new_text,
        get_item_priority(priority),
        tags,
        notes,
    ));

    save_dayfile(&path, &dayfile)?;

    if let Some(item) = dayfile.items.last() {
        render_summary(
            None,
            item,
            RenderOpts {
                json: cli.json,
                verbose: cli.verbose,
                no_color: cli.no_colour,
                vault_name: None,
                dry_run: false,
            },
        )?;
    }

    Ok(())
}

fn run_ls(cli: &Cli, tags: &Option<Vec<String>>) -> io::Result<()> {
    let (date, path) = current_day_context(cli)?;
    println!("Date: {}", date);
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
            dry_run: false,
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

fn run_edit(cli: &Cli, idx: usize, text: &Option<String>, attach_notes: &bool) -> io::Result<()> {
    let (date, path) = current_day_context(cli)?;
    let mut dayfile = load_or_create_dayfile(&path, date)?;

    let pos = validate_index(idx, dayfile.items.len())?;

    if let Some(item) = dayfile.items.get_mut(pos) {
        if let Some(s) = text {
            item.text = sanitise_str(s)?;
        }

        let notes = if *attach_notes {
            let template = item.notes.as_deref().unwrap_or("# Notes");
            Some(edit_in_editor(&template)?)
        } else {
            None
        };

        item.notes = notes;

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
            Some(idx),
            item,
            RenderOpts {
                json: cli.json,
                verbose: cli.verbose,
                no_color: cli.no_colour,
                vault_name: None,
                dry_run: false,
            },
        )?;
    }

    Ok(())
}

fn run_migrate(
    cli: &Cli,
    from_date: &Option<NaiveDate>,
    to_date: &Option<NaiveDate>,
    dry_run: &bool,
) -> io::Result<()> {
    let from_date = from_date.unwrap_or_else(today_date);
    let to_date = to_date.unwrap_or_else(today_date);

    if from_date == to_date {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Both `--from` and `--to` are the same value, check your input.",
        ));
    }

    let from_date_path = resolve_day_file_path(
        &from_date,
        cli.data_dir.as_deref(),
        cli.verbose,
        cli.vault.as_deref(),
    )?;

    let mut from_dayfile = load_or_create_dayfile(&from_date_path, from_date)?;
    let mut items_to_move: Vec<Item> = Vec::new();

    from_dayfile.items.retain_mut(|i| {
        if i.done_at.is_none() {
            i.migrated_from = Some(from_date);
            items_to_move.push(i.clone());
            false
        } else {
            true
        }
    });

    if !*dry_run {
        save_dayfile(&from_date_path, &from_dayfile)?;
    }

    let to_day_path = resolve_day_file_path(
        &to_date,
        cli.data_dir.as_deref(),
        cli.verbose,
        cli.vault.as_deref(),
    )?;

    let mut to_dayfile = load_or_create_dayfile(&to_day_path, to_date)?;
    to_dayfile.items.append(&mut items_to_move);

    render_migrate(
        &to_dayfile,
        RenderOpts {
            json: cli.json,
            verbose: cli.verbose,
            no_color: cli.no_colour,
            vault_name: None,
            dry_run: *dry_run,
        },
    )?;

    if !*dry_run {
        save_dayfile(&to_day_path, &to_dayfile)?;
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
