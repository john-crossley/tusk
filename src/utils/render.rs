use chrono::NaiveDate;
use clap::ValueEnum;
use std::io::{self, Error, Write};

use crate::{
    Cli,
    display::{
        json::JsonRenderer, markdown::MarkdownRenderer, renderer::Renderer,
        terminal::TerminalRenderer,
    },
    models::{dayfile::DayFile, item::Item},
    utils::theme::Theme,
};

#[derive(Debug, Clone, PartialEq, ValueEnum, Copy)]
pub enum RenderOutput {
    Terminal,
    Json,
    #[value(alias = "md")]
    Markdown,
}

#[derive(Clone)]
pub struct RenderOpts {
    pub output: RenderOutput,
    pub verbose: bool,
    pub vault_name: Option<String>,
    pub dry_run: bool,
    pub color: bool,
}

impl Default for RenderOpts {
    fn default() -> Self {
        RenderOpts {
            output: RenderOutput::Terminal,
            verbose: false,
            vault_name: None,
            dry_run: false,
            color: false,
        }
    }
}

impl From<&Cli> for RenderOpts {
    fn from(cli: &Cli) -> Self {
        Self {
            output: cli.output,
            verbose: cli.verbose,
            vault_name: cli.vault.clone(),
            dry_run: false,
            color: !cli.no_colour,
        }
    }
}

pub fn render_migrate(
    to_df: &DayFile,
    from_df: &DayFile,
    items: &[Item],
    opts: RenderOpts,
) -> Result<(), Error> {
    todo!()
    // let mut stdout = io::stdout().lock();

    // if opts.output == RenderOutput::Json {
    //     return as_json(stdout, &to_df);
    // }

    // let theme = Theme::new(true); // opts.color
    // let title = build_title_header(&to_df, opts.vault_name.as_deref(), Some(from_df), &theme);
    // title_underline(&theme, &title, &mut stdout)?;

    // if items.is_empty() {
    //     writeln!(
    //         &mut stdout,
    //         "🦣 {}",
    //         theme.dim(&format!("No tasks to migrate from {}", from_df.date))
    //     )?;

    //     return Ok(());
    // }

    // render_list(&mut stdout, items, &theme, opts.verbose)?;
    // render_migration_count(&mut stdout, items, from_df.date, &theme, opts.dry_run)?;

    // Ok(())
}

fn render_migration_count(
    mut out: impl Write,
    items: &[Item],
    date: NaiveDate,
    theme: &Theme,
    dry_run: bool,
) -> Result<(), Error> {
    let count = items.len();

    if count == 0 {
        return Ok(());
    }

    let item_word = if count == 1 { "item" } else { "items" };
    let details = if dry_run {
        "will be migrated from:"
    } else {
        "migrated:"
    };

    let date_str = date.format("%a %d %b %Y").to_string();

    writeln!(
        out,
        "  ↪ {} {} {} {}",
        theme.info(&count.to_string()),
        item_word,
        details,
        theme.info(&date_str)
    )?;

    Ok(())
}

pub fn render_review_title(
    start: &NaiveDate,
    end: &NaiveDate,
    days: u64,
    opts: &RenderOpts,
) -> io::Result<()> {
    let mut out = io::stdout().lock();

    let theme = Theme::new(true);

    let formatted_start = start.format("%a %d %b %Y").to_string();
    let formatted_end = end.format("%a %d %b %Y").to_string();
    let title = format!("Review: {} → {}", formatted_start, formatted_end);

    writeln!(&mut out, "\n🦣 {}", theme.title(title.as_str()))?;

    let sub = format!("Last {} day(s) (excluding today)", days);
    writeln!(&mut out, "{}", theme.info_em(sub.as_str()))?;

    writeln!(&mut out, "\n{}", theme.title("Summary"))?;
    writeln!(&mut out, "- Total tasks: 18")?;
    writeln!(&mut out, "- Open: 13")?;
    writeln!(&mut out, "- Completed: 5")?;
    writeln!(&mut out, "- Days with activity: 4")?;

    writeln!(&mut out, "---")?;

    Ok(())
}

pub fn render_review_dayfile(df: &DayFile, opts: &RenderOpts) -> io::Result<()> {
    let mut out = io::stdout().lock();
    let theme = Theme::new(true);

    // ## Tue 16 Sep 2025 (12 tasks: 11 open, 1 done)
    // - [ ] Hello, World ahaom! ▽

    writeln!(&mut out, "Tue 16 Sep 2025 (12 tasks: 11 open, 1 done)")?;

    for item in &df.items {
        let is_done = item.done_at.is_some();
        let boxy = theme.checkbox(is_done);

        writeln!(&mut out, "{} {}", boxy, item.text)?;
    }

    Ok(())
}

pub fn make_renderer(opts: &RenderOpts) -> RendererImpl {
    match opts.output {
        RenderOutput::Terminal => RendererImpl::Terminal(TerminalRenderer {
            theme: Theme::new(opts.color),
            vault: opts.vault_name.clone(),
            verbose: opts.verbose,
        }),
        RenderOutput::Json => RendererImpl::Json(JsonRenderer {}),
        RenderOutput::Markdown => RendererImpl::Markdown(MarkdownRenderer {}),
    }
}

pub enum RendererImpl {
    Terminal(TerminalRenderer),
    Json(JsonRenderer),
    Markdown(MarkdownRenderer),
}

impl RendererImpl {
    pub fn render_day(&self, df: &DayFile) -> io::Result<()> {
        match self {
            RendererImpl::Terminal(r) => r.render_day(df),
            RendererImpl::Json(r) => r.render_day(df),
            RendererImpl::Markdown(r) => r.render_day(df),
        }
    }

    pub fn render_summary(&self, index: Option<usize>, item: &Item) -> io::Result<()> {
        match self {
            RendererImpl::Terminal(r) => r.render_summary(index, item),
            RendererImpl::Json(r) => r.render_summary(index, item),
            RendererImpl::Markdown(r) => r.render_summary(index, item),
        }
    }
}
