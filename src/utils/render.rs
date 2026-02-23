use chrono::NaiveDate;
use clap::ValueEnum;
use std::io::{self};

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
    pub color: bool,
}

impl From<&Cli> for RenderOpts {
    fn from(cli: &Cli) -> Self {
        Self {
            output: cli.output,
            verbose: cli.verbose,
            vault_name: cli.vault.clone(),
            color: !cli.no_colour,
        }
    }
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

    pub fn render_summary(&self, date: NaiveDate, index: usize, item: &Item) -> io::Result<()> {
        match self {
            RendererImpl::Terminal(r) => r.render_summary(date, index, item),
            RendererImpl::Json(r) => r.render_summary(date, index, item),
            RendererImpl::Markdown(r) => r.render_summary(date, index, item),
        }
    }

    pub fn render_review(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        days: u64,
        dayfiles: &[DayFile],
    ) -> io::Result<()> {
        match self {
            RendererImpl::Terminal(r) => r.render_review(start, end, days, dayfiles),
            RendererImpl::Json(r) => r.render_review(start, end, days, dayfiles),
            RendererImpl::Markdown(r) => r.render_review(start, end, days, dayfiles),
        }
    }

    pub fn render_migrate(
        &self,
        to_date: NaiveDate,
        from_df_original: &DayFile,
        moved_items: &[Item],
        dry_run: bool,
    ) -> io::Result<()> {
        match self {
            RendererImpl::Terminal(r) => r.render_migrate(to_date, from_df_original, moved_items, dry_run),
            RendererImpl::Json(r) => r.render_migrate(to_date, from_df_original, moved_items, dry_run),
            RendererImpl::Markdown(r) => r.render_migrate(to_date, from_df_original, moved_items, dry_run),
        }
    }
}
