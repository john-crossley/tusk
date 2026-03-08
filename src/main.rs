use std::{
    io::{self},
    path::PathBuf,
};

use chrono::{Days, NaiveDate, Utc};
use clap::{Parser, Subcommand};

use crate::{
    models::{
        dayfile::DayFile,
        item::{Item, ItemPriority},
    },
    utils::{
        dates::{parse_ymd, todays_date},
        editor::edit_in_editor,
        files::{load_or_empty, save_dayfile},
        helpers::{extract_tags, sanitise_str, validate_index, warn_dayfile_error},
        render::{ActionKind, RenderOpts, RenderOutput, make_renderer},
        tusk_error::TuskError,
    },
};

mod display;
mod models;
mod store;
mod utils;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Tusk - simple daily todos in your terminal",
    long_about = "Tusk is a lightweight CLI that stores each day's todos in a JSON file. \
                  Add tasks, list them, mark them as done, and export to Markdown with zero friction.",
    subcommand = "ls"
)]
pub struct Cli {
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
        tags: Vec<String>,
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

    /// Manage persistent focus tasks
    #[clap(subcommand)]
    Focus(FocusCommands),
}

#[derive(Subcommand, Debug)]
enum FocusCommands {
    #[command(name = "ls", about = "List long running items.")]
    Ls,

    #[command(name = "add", about = "Add a new long running item.")]
    Add {
        /// The description of the item being added.
        text: String,
    },
}

struct CommandContext {
    data_dir: Option<PathBuf>,
    vault: Option<String>,
    render_opts: RenderOpts,
}

impl From<&Cli> for CommandContext {
    fn from(cli: &Cli) -> Self {
        Self {
            data_dir: cli.data_dir.clone(),
            vault: cli.vault.clone(),
            render_opts: RenderOpts::from(cli),
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let cmd_ctx = CommandContext::from(&cli);
    let renderer = make_renderer(&cmd_ctx.render_opts);
    let cmd_name = command_name(cli.command.as_ref());

    if let Err(e) = dispatch(cli, cmd_ctx) {
        if let Err(render_err) = renderer.render_error(cmd_name, &e) {
            eprintln!("Tusk: {e}");
            eprintln!("Failed to render error: {render_err}");
        }
        std::process::exit(1);
    }
}

fn dispatch(cli: Cli, ctx: CommandContext) -> Result<(), TuskError> {
    match cli.command {
        Some(Commands::Add {
            date,
            text,
            priority,
            attach_notes,
        }) => run_add(date, text, priority, attach_notes, ctx),
        Some(Commands::Ls { date, tags }) => run_ls(date, tags, ctx),
        Some(Commands::Done { date, index }) => run_done(date, index, true, ctx),
        Some(Commands::Undone { date, index }) => run_done(date, index, false, ctx),
        Some(Commands::Rm { date, index }) => run_rm(date, index, ctx),
        Some(Commands::Edit {
            date,
            index,
            text,
            attach_notes,
            priority,
        }) => run_edit(date, index, text, attach_notes, priority, ctx),
        Some(Commands::Show { date, index }) => run_show(date, index, ctx),
        Some(Commands::Migrate {
            from_date,
            to_date,
            dry_run,
        }) => run_migrate(from_date, to_date, dry_run, ctx),
        Some(Commands::Review { days }) => run_review(days, ctx),
        Some(Commands::Focus(focus_commands)) => dispatch_focus(focus_commands, ctx),
        None => run_ls(None, vec![], ctx),
    }
}

fn dispatch_focus(commands: FocusCommands, ctx: CommandContext) -> Result<(), TuskError> {
    match commands {
        FocusCommands::Ls {} => run_ls(None, vec![], ctx),
        FocusCommands::Add { text } => run_add(None, text, None, false, ctx),
    }
}

// command handler functions

fn run_add(
    date: Option<NaiveDate>,
    text: String,
    priority: Option<ItemPriority>,
    attach_notes: bool,
    ctx: CommandContext,
) -> Result<(), TuskError> {
    let new_text = sanitise_str(&text)?;
    let tags = extract_tags(&text);
    let mut df = load_or_empty(&ctx, date)?;

    let notes = if attach_notes {
        Some(edit_in_editor("")?)
    } else {
        None
    };

    df.items.push(Item::new(
        new_text,
        priority.unwrap_or(ItemPriority::Low),
        tags,
        notes,
    ));

    save_dayfile(&ctx, &df)?;

    if let Some(item) = df.items.last() {
        let renderer = make_renderer(&ctx.render_opts);
        renderer.render_summary(df.date, df.items.len(), item)?;
    }

    Ok(())
}

fn run_ls(
    date: Option<NaiveDate>,
    tags: Vec<String>,
    ctx: CommandContext,
) -> Result<(), TuskError> {
    let df = load_or_empty(&ctx, date)?;
    let renderer = make_renderer(&ctx.render_opts);

    let render_df = if tags.is_empty() {
        df
    } else {
        df.filtered_by_tags(&tags)
    };

    renderer.render_day(&render_df)?;

    Ok(())
}

fn run_done(
    date: Option<NaiveDate>,
    index: usize,
    mark_done: bool,
    ctx: CommandContext,
) -> Result<(), TuskError> {
    let mut df = load_or_empty(&ctx, date)?;
    let pos = validate_index(index, df.items.len())?;

    {
        let item = &mut df.items[pos];
        item.done_at = if mark_done {
            item.done_at.take().or(Some(Utc::now()))
        } else {
            None
        };
    }

    let item = &df.items[pos];
    save_dayfile(&ctx, &df)?;

    let action = if mark_done {
        ActionKind::Done
    } else {
        ActionKind::Undone
    };

    let renderer = make_renderer(&ctx.render_opts);
    renderer.render_action(index, df.date, action, Some(&item))?;

    Ok(())
}

fn run_rm(date: Option<NaiveDate>, index: usize, ctx: CommandContext) -> Result<(), TuskError> {
    let mut df = load_or_empty(&ctx, date)?;

    let pos = validate_index(index, df.items.len())?;
    let item = df.items.remove(pos);
    save_dayfile(&ctx, &df)?;

    let renderer = make_renderer(&ctx.render_opts);
    renderer.render_action(index, df.date, ActionKind::Removed, Some(&item))?;

    Ok(())
}

fn run_edit(
    date: Option<NaiveDate>,
    index: usize,
    text: Option<String>,
    attach_notes: bool,
    priority: Option<ItemPriority>,
    ctx: CommandContext,
) -> Result<(), TuskError> {
    let mut df = load_or_empty(&ctx, date)?;
    let pos = validate_index(index, df.items.len())?;

    if let Some(item) = df.items.get_mut(pos) {
        if let Some(s) = text {
            item.text = sanitise_str(&s)?;
        }

        let notes = if attach_notes {
            let template = item.notes.as_deref().unwrap_or("");
            Some(edit_in_editor(&template)?)
        } else {
            None
        };

        if notes.is_some() {
            item.notes = notes;
        }

        if let Some(p) = priority {
            item.priority = p;
        }

        save_dayfile(&ctx, &df)?;
    }

    Ok(())
}

fn run_show(date: Option<NaiveDate>, index: usize, ctx: CommandContext) -> Result<(), TuskError> {
    let mut df = load_or_empty(&ctx, date)?;
    let pos = validate_index(index, df.items.len())?;

    if let Some(item) = df.items.get_mut(pos) {
        let renderer = make_renderer(&ctx.render_opts);
        renderer.render_summary(df.date, index, item)?;
    }

    Ok(())
}

fn run_migrate(
    from_date: Option<NaiveDate>,
    to_date: Option<NaiveDate>,
    dry_run: bool,
    ctx: CommandContext,
) -> Result<(), TuskError> {
    let from_date = from_date.unwrap_or_else(todays_date);
    let to_date = to_date.unwrap_or_else(todays_date);

    if from_date == to_date {
        return Err(TuskError::InvalidInput {
            message: "Both `--from` and `--to` are the same value, check your input.".to_string(),
        });
    }

    let mut from_df = load_or_empty(&ctx, Some(from_date))?;
    let from_df_before = from_df.clone();

    let mut to_df = load_or_empty(&ctx, Some(to_date))?;
    let renderer = make_renderer(&ctx.render_opts);

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

        save_dayfile(&ctx, &from_df)?;
        save_dayfile(&ctx, &to_df)?;

        renderer.render_migrate(to_df.date, &from_df_before, &moved_items, false)?;
    }

    Ok(())
}

fn run_review(days: Option<u64>, ctx: CommandContext) -> Result<(), TuskError> {
    let days = days.unwrap_or(1);

    if days > 365 {
        return Err(TuskError::InvalidInput {
            message: "Review can't be more than 365 days".to_string(),
        });
    }

    let today = todays_date();

    let start = today
        .checked_sub_days(Days::new(days))
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "data underflow"))?;

    let end = today;

    let mut dayfiles: Vec<DayFile> = Vec::new();

    for d in start.iter_days().take_while(|d| *d < end) {
        match load_or_empty(&ctx, Some(d)) {
            Ok(df) => {
                if !df.items.is_empty() {
                    dayfiles.push(df);
                }
            }
            Err(e) => warn_dayfile_error(d, &e, ctx.render_opts.verbose),
        }
    }

    let renderer = make_renderer(&ctx.render_opts);
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
        Some(Commands::Focus(focus_cmd)) => match focus_cmd {
            FocusCommands::Ls { .. } => "focus ls",
            FocusCommands::Add { .. } => "focus add",
        },
        None => "ls",
    }
}
