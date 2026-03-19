use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::{models::item::ItemPriority, utils::{dates::parse_ymd, list_scope::ListScope, render::{RenderOpts, RenderOutput}}};

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
    pub data_dir: Option<PathBuf>,

    /// Specify the terminal output as Terminal, JSON or markdown.
    #[arg(short, long, value_enum, default_value_t = RenderOutput::Terminal)]
    pub output: RenderOutput,

    /// Disable coloured output (useful in scripts or non-TTY environments).
    #[arg(short, long)]
    pub no_colour: bool,

    /// Enables verbose logging, useful for debugging.
    #[arg(long)]
    pub verbose: bool,

    /// Specifies which vault to operate in.
    #[arg(short, long)]
    pub vault: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}
#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(name = "ls", about = "List items for the target date")]
    Ls {
        /// Target date (YYYY-MM-DD). Defaults to today if omitted.
        #[arg(short, long, value_parser = parse_ymd, value_name = "YYYY-MM-DD")]
        date: Option<NaiveDate>,

        /// Filter tasks by one or more tags
        #[arg(long = "tag", num_args = 1..)]
        tags: Vec<String>,

        /// Filter list items by scope
        #[arg(short = 's', long = "scope")]
        scope: Option<ListScope>,
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
pub enum FocusCommands {
    #[command(name = "ls", about = "List long running items.")]
    Ls,

    #[command(name = "add", about = "Add a new long running item.")]
    Add {
        /// The description of the item being added.
        text: String,
    },

    #[command(name = "done", about = "Mark a long running item done by its index")]
    Done {
        /// Target date (YYYY-MM-DD). Defaults to today if omitted.
        #[arg(short, long, value_parser = parse_ymd, value_name = "YYYY-MM-DD")]
        date: Option<NaiveDate>,

        index: usize,
    },

    #[command(name = "undone", about = "Mark a long running item undone by its index")]
    Undone {
        /// Target date (YYYY-MM-DD). Defaults to today if omitted.
        #[arg(short, long, value_parser = parse_ymd, value_name = "YYYY-MM-DD")]
        date: Option<NaiveDate>,

        index: usize,
    },

    #[command(name = "rm", about = "Remove a long running item from your list.")]
    Rm {
        /// Target date (YYYY-MM-DD). Defaults to today if omitted.
        #[arg(short, long, value_parser = parse_ymd, value_name = "YYYY-MM-DD")]
        date: Option<NaiveDate>,
        /// The index of the item to be removed.
        index: usize,
    },
}

pub struct CommandContext {
    pub data_dir: Option<PathBuf>,
    pub vault: Option<String>,
    pub render_opts: RenderOpts,
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