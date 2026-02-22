use std::{
    io::{self, Error},
    path::PathBuf,
};

use chrono::{Days, NaiveDate, Utc};
use clap::{Parser, Subcommand};

mod display;
mod models;
mod utils;

use crate::{
    models::{dayfile::DayFile, item::Item},
    utils::{
        dates::{parse_ymd, todays_date},
        editor::edit_in_editor,
        files::{
            load_dayfile_if_exists, load_or_create_dayfile, resolve_day_file_path, save_dayfile,
        },
        helpers::{
            current_day_context, extract_tags, get_item_priority, sanitise_str, validate_index,
            warn_dayfile_error,
        },
        render::{RenderOpts, RenderOutput, make_renderer},
    },
};

///
/// Tasks to complete
/// 1. Remove the date variable from the global args scope.
/// 2. Can the from, to dates accept "today", "tomorrow", "yesterday"?
/// 3. Show notes when `t show <INDEX>`
///

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

    /// Specify the terminal output as Terminal, JSON or markdown.
    #[arg(short, long, value_enum, default_value_t = RenderOutput::Terminal)]
    output: RenderOutput,

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
        /// The priority of the item being edited.
        #[arg(short = 'p', long = "priority")]
        priority: Option<String>,
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

    #[command(
        name = "review",
        about = "Grabs a slice of tasks within a specified time period."
    )]
    Review {
        #[arg(name = "days", long)]
        days: Option<u64>,
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
            priority,
        }) => run_edit(&cli, *index, text, attach_notes, priority.as_deref()),
        Some(Commands::Show { index }) => run_show(&cli, *index),
        Some(Commands::Migrate {
            from_date,
            to_date,
            dry_run,
        }) => run_migrate(&cli, from_date, to_date, *dry_run),
        Some(Commands::Review { days }) => run_review(&cli, *days),
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
    let mut df = load_or_create_dayfile(&path, date)?;

    let notes = if *attach_notes {
        Some(edit_in_editor("# Notes")?)
    } else {
        None
    };

    df.items.push(Item::new(
        new_text,
        get_item_priority(priority),
        tags,
        notes,
    ));

    save_dayfile(&path, &df)?;

    if let Some(item) = df.items.last() {
        let opts: RenderOpts = cli.into();
        let renderer = make_renderer(&opts);

        renderer.render_summary(df.date, df.items.len(), item)?;
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

    let opts: RenderOpts = cli.into();
    let renderer = make_renderer(&opts);

    renderer.render_day(&dayfile)?;

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

fn run_edit(
    cli: &Cli,
    idx: usize,
    text: &Option<String>,
    attach_notes: &bool,
    priority: Option<&str>,
) -> io::Result<()> {
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

        if priority.is_some() {
            item.priority = get_item_priority(priority);
        }

        save_dayfile(&path, &dayfile)?;
    }

    Ok(())
}

fn run_show(cli: &Cli, idx: usize) -> io::Result<()> {
    let (date, path) = current_day_context(cli)?;
    let mut df = load_or_create_dayfile(&path, date)?;
    let pos = validate_index(idx, df.items.len())?;

    if let Some(item) = df.items.get_mut(pos) {
        let opts: RenderOpts = cli.into();
        let renderer = make_renderer(&opts);
        renderer.render_summary(df.date, idx, item)?;
    }

    Ok(())
}

fn run_migrate(
    cli: &Cli,
    from_date: &Option<NaiveDate>,
    to_date: &Option<NaiveDate>,
    dry_run: bool,
) -> io::Result<()> {
    let from_date = from_date.unwrap_or_else(todays_date);
    let to_date = to_date.unwrap_or_else(todays_date);

    if from_date == to_date {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Both `--from` and `--to` are the same value, check your input.",
        ));
    }

    let from_df_path = resolve_day_file_path(
        &from_date,
        cli.data_dir.as_deref(),
        cli.verbose,
        cli.vault.as_deref(),
    )?;

    let to_df_path = resolve_day_file_path(
        &to_date,
        cli.data_dir.as_deref(),
        cli.verbose,
        cli.vault.as_deref(),
    )?;

    let mut from_df = load_or_create_dayfile(&from_df_path, from_date)?;
    let mut to_df = load_or_create_dayfile(&to_df_path, to_date)?;
    let pending_items = &from_df.migratable_items();
    let opts: RenderOpts = cli.into();

    let renderer = make_renderer(&opts);

    if dry_run {
        let mut preview = to_df.clone();
        preview.items.extend_from_slice(&pending_items);
        renderer.render_migrate(&preview, &from_df, pending_items, true)?;
    } else {
        let (mut to_move, to_keep): (Vec<Item>, Vec<Item>) =
            from_df.items.into_iter().partition(|i| i.done_at.is_none());

        for i in &mut to_move {
            i.migrated_from = Some(from_date);
        }

        from_df.items = to_keep;
        to_df.items.extend(to_move);

        save_dayfile(&from_df_path, &from_df)?;
        save_dayfile(&to_df_path, &to_df)?;

        renderer.render_migrate(&to_df, &from_df, pending_items, false)?;
    }

    Ok(())
}

fn run_review(cli: &Cli, days: Option<u64>) -> io::Result<()> {
    let days = days.unwrap_or(1);
    let today = todays_date();

    let start = today
        .checked_sub_days(Days::new(days))
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "data underflow"))?;

    let end = today;
    let opts: RenderOpts = cli.into();

    let mut dayfiles: Vec<DayFile> = Vec::new();

    for d in start.iter_days().take_while(|d| *d < end) {
        let path = resolve_day_file_path(
            &d,
            cli.data_dir.as_deref(),
            opts.verbose,
            opts.vault_name.as_deref(),
        )?;

        match load_dayfile_if_exists(&path) {
            Ok(df) => {
                if !df.items.is_empty() {
                    dayfiles.push(df);
                }
            }
            Err(e) => warn_dayfile_error(d, &path, &e, cli.verbose),
        }
    }

    let renderer = make_renderer(&opts);
    renderer.render_review(&start, &end, days, &dayfiles)?;

    Ok(())
}
