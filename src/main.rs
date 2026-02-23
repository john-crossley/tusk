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
    models::{
        dayfile::DayFile,
        item::{Item, ItemPriority},
    },
    utils::{
        dates::{parse_ymd, todays_date},
        editor::edit_in_editor,
        files::{
            load_dayfile_if_exists, load_or_create_dayfile, resolve_day_file_path, save_dayfile,
        },
        helpers::{
            current_day_context, extract_tags, sanitise_str, validate_index, warn_dayfile_error,
        },
        render::{ActionKind, RenderOpts, RenderOutput, make_renderer},
        tusk_error::TuskError,
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
        /// Target date (YYYY-MM-DD). Defaults to today if omitted.
        #[arg(short, long, value_parser = parse_ymd, value_name = "YYYY-MM-DD")]
        date: Option<NaiveDate>,

        /// Filter tasks by one or more tags
        #[arg(long = "tag", num_args = 1..)]
        tags: Option<Vec<String>>,
    },

    #[command(name = "add", about = "Add a new item to your day")]
    Add {
        /// Target date (YYYY-MM-DD). Defaults to today if omitted.
        #[arg(short, long, value_parser = parse_ymd, value_name = "YYYY-MM-DD")]
        date: Option<NaiveDate>,

        /// The description of the item being added.
        text: String,

        /// The priority of the item being added.
        #[arg(short = 'p', long = "priority")]
        priority: Option<ItemPriority>,

        /// Add a note to this item, opens in an external editor
        #[arg(short = 'n', long = "notes")]
        attach_notes: bool,
    },

    #[command(name = "done", about = "Mark an item done by its index")]
    Done {
        /// Target date (YYYY-MM-DD). Defaults to today if omitted.
        #[arg(short, long, value_parser = parse_ymd, value_name = "YYYY-MM-DD")]
        date: Option<NaiveDate>,

        index: usize,
    },

    #[command(name = "undone", about = "Mark an item undone by its index")]
    Undone {
        /// Target date (YYYY-MM-DD). Defaults to today if omitted.
        #[arg(short, long, value_parser = parse_ymd, value_name = "YYYY-MM-DD")]
        date: Option<NaiveDate>,

        index: usize,
    },

    #[command(name = "rm", about = "Remove an item from your list.")]
    Rm {
        /// Target date (YYYY-MM-DD). Defaults to today if omitted.
        #[arg(short, long, value_parser = parse_ymd, value_name = "YYYY-MM-DD")]
        date: Option<NaiveDate>,

        index: usize,
    },

    #[command(name = "edit", about = "Edit an item from your list.")]
    Edit {
        /// Target date (YYYY-MM-DD). Defaults to today if omitted.
        #[arg(short, long, value_parser = parse_ymd, value_name = "YYYY-MM-DD")]
        date: Option<NaiveDate>,

        index: usize,
        text: Option<String>,
        /// Add a note to this item, opens in an external editor
        #[arg(short = 'n', long = "notes")]
        attach_notes: bool,
        /// The priority of the item being edited.
        #[arg(short = 'p', long = "priority")]
        priority: Option<ItemPriority>,
    },

    #[command(name = "show", about = "Show an item by its index.")]
    Show {
        /// Target date (YYYY-MM-DD). Defaults to today if omitted.
        #[arg(short, long, value_parser = parse_ymd, value_name = "YYYY-MM-DD")]
        date: Option<NaiveDate>,

        index: usize,
    },

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
    let opts = RenderOpts::from(&cli);
    let renderer = make_renderer(&opts);

    if let Err(e) = dispatch(&cli) {
        if let Err(render_err) = renderer.render_error(command_name(cli.command.as_ref()), &e) {
            eprintln!("Tusk: {e}");
            eprintln!("Failed to render error: {render_err}");
        }
        std::process::exit(1);
    }
}

fn dispatch(cli: &Cli) -> Result<(), TuskError> {
    match cli.command.as_ref() {
        Some(Commands::Add {
            date,
            text,
            priority,
            attach_notes,
        }) => run_add(&cli, *date, text, *priority, attach_notes),
        Some(Commands::Ls { date, tags }) => run_ls(cli, *date, tags.as_deref().unwrap_or(&[])),
        Some(Commands::Done { date, index }) => run_done(&cli, *date, *index, true),
        Some(Commands::Undone { date, index }) => run_done(&cli, *date, *index, false),
        Some(Commands::Rm { date, index }) => run_rm(&cli, *date, *index),
        Some(Commands::Edit {
            date,
            index,
            text,
            attach_notes,
            priority,
        }) => run_edit(&cli, *date, *index, text, attach_notes, *priority),
        Some(Commands::Show { date, index }) => run_show(&cli, *date, *index),
        Some(Commands::Migrate {
            from_date,
            to_date,
            dry_run,
        }) => run_migrate(&cli, from_date, to_date, *dry_run),
        Some(Commands::Review { days }) => run_review(&cli, *days),
        None => run_ls(&cli, None, &[]),
    }
}

// command handler functions

fn run_add(
    cli: &Cli,
    date: Option<NaiveDate>,
    text: &str,
    priority: Option<ItemPriority>,
    attach_notes: &bool,
) -> Result<(), TuskError> {
    let new_text = sanitise_str(text)?;
    let tags = extract_tags(text);

    let (date, path) = current_day_context(cli, date)?;
    let mut df = load_or_create_dayfile(&path, date)?;

    let notes = if *attach_notes {
        Some(edit_in_editor("# Notes")?)
    } else {
        None
    };

    df.items.push(Item::new(
        new_text,
        priority.unwrap_or(ItemPriority::Low),
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

fn run_ls(cli: &Cli, date: Option<NaiveDate>, tags: &[String]) -> Result<(), TuskError> {
    let (date, path) = current_day_context(cli, date)?;
    let mut dayfile = load_or_create_dayfile(&path, date)?;

    if !tags.is_empty() {
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

fn run_done(
    cli: &Cli,
    date: Option<NaiveDate>,
    idx: usize,
    mark_done: bool,
) -> Result<(), TuskError> {
    let (date, path) = current_day_context(cli, date)?;
    let mut dayfile = load_or_create_dayfile(&path, date)?;

    let pos = validate_index(idx, dayfile.items.len())?;

    {
        let item = &mut dayfile.items[pos];
        item.done_at = if mark_done {
            item.done_at.take().or(Some(Utc::now()))
        } else {
            None
        };
    }

    let item = &dayfile.items[pos];
    save_dayfile(&path, &dayfile)?;

    let action = if mark_done {
        ActionKind::Done
    } else {
        ActionKind::Undone
    };

    let renderer = make_renderer(&cli.into());
    renderer.render_action(idx, date, action, Some(&item))?;

    Ok(())
}

fn run_rm(cli: &Cli, date: Option<NaiveDate>, idx: usize) -> Result<(), TuskError> {
    let (date, path) = current_day_context(cli, date)?;
    let mut dayfile = load_or_create_dayfile(&path, date)?;

    let pos = validate_index(idx, dayfile.items.len())?;
    let item = dayfile.items.remove(pos);
    save_dayfile(&path, &dayfile)?;

    let renderer = make_renderer(&cli.into());
    renderer.render_action(idx, date, ActionKind::Removed, Some(&item))?;

    Ok(())
}

fn run_edit(
    cli: &Cli,
    date: Option<NaiveDate>,
    idx: usize,
    text: &Option<String>,
    attach_notes: &bool,
    priority: Option<ItemPriority>,
) -> Result<(), TuskError> {
    let (date, path) = current_day_context(cli, date)?;
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

        if let Some(p) = priority {
            item.priority = p;
        }

        save_dayfile(&path, &dayfile)?;
    }

    Ok(())
}

fn run_show(cli: &Cli, date: Option<NaiveDate>, idx: usize) -> Result<(), TuskError> {
    let (date, path) = current_day_context(cli, date)?;
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
) -> Result<(), TuskError> {
    let from_date = from_date.unwrap_or_else(todays_date);
    let to_date = to_date.unwrap_or_else(todays_date);

    if from_date == to_date {
        return Err(TuskError::InvalidInput {
            message: "Both `--from` and `--to` are the same value, check your input.".to_string(),
        });
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
    let from_df_before = from_df.clone();

    let mut to_df = load_or_create_dayfile(&to_df_path, to_date)?;
    let opts: RenderOpts = cli.into();

    let renderer = make_renderer(&opts);

    if dry_run {
        let mut pending_items = from_df.migratable_items();

        for i in pending_items.iter_mut() {
            i.migrated_from = Some(from_date);
        }

        renderer.render_migrate(to_df.date, &from_df_before, &pending_items, true)?;
    } else {
        let (mut to_move, to_keep): (Vec<Item>, Vec<Item>) =
            from_df.items.into_iter().partition(|i| i.done_at.is_none());

        for i in &mut to_move {
            i.migrated_from = Some(from_date);
        }

        from_df.items = to_keep;
        let moved_items = to_move.clone();
        to_df.items.extend(to_move);

        save_dayfile(&from_df_path, &from_df)?;
        save_dayfile(&to_df_path, &to_df)?;

        renderer.render_migrate(to_df.date, &from_df_before, &moved_items, false)?;
    }

    Ok(())
}

fn run_review(cli: &Cli, days: Option<u64>) -> Result<(), TuskError> {
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
    renderer.render_review(start, end, days, &dayfiles)?;

    Ok(())
}

fn command_name(cmd: Option<&Commands>) -> &'static str {
    match cmd {
        Some(Commands::Add { .. }) => "add",
        Some(Commands::Ls { .. }) => "ls",
        Some(Commands::Done { .. }) => "done",
        Some(Commands::Undone { .. }) => "undone",
        Some(Commands::Rm { .. }) => "rm",
        Some(Commands::Edit { .. }) => "edit",
        Some(Commands::Show { .. }) => "show",
        Some(Commands::Migrate { .. }) => "migrate",
        Some(Commands::Review { .. }) => "review",
        None => "ls",
    }
}
