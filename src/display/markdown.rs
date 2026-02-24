use std::io::{self, Write};

use chrono::NaiveDate;

use crate::{
    display::renderer::Renderer,
    models::{dayfile::DayFile, item::Item},
    utils::{helpers::stats, render::ActionKind, tusk_error::TuskError},
};

pub struct MarkdownRenderer;

impl Renderer for MarkdownRenderer {
    fn render_day(&self, df: &DayFile) -> std::io::Result<()> {
        let mut out = io::stdout().lock();
        let title = self.build_title(df.date, None);

        writeln!(out, "{}", title)?;

        if df.items.is_empty() {
            writeln!(
                out,
                "🦣 No tasks for {}",
                format!("No tasks for {}", df.date)
            )?;

            let hint = r#"tusk add "Drink more water 💦""#;
            writeln!(out, "   _Add one with: {}_", hint)?;
        }

        self.render_list(&mut out, &df.items)?;
        self.render_footer(&mut out, df)?;

        Ok(())
    }

    fn render_summary(&self, _date: NaiveDate, _index: usize, _item: &Item) -> std::io::Result<()> {
        todo!()
    }

    fn render_migrate(
        &self,
        _to_date: NaiveDate,
        _from_df_original: &DayFile,
        _moved_items: &[Item],
        _dry_run: bool,
    ) -> std::io::Result<()> {
        todo!()
    }

    fn render_review(
        &self,
        _start: NaiveDate,
        _end: NaiveDate,
        _days: u64,
        _dayfiles: &[DayFile],
    ) -> std::io::Result<()> {
        todo!()
    }

    fn render_action(
        &self,
        _index: usize,
        _date: NaiveDate,
        _action: ActionKind,
        _item: Option<&Item>,
    ) -> std::io::Result<()> {
        todo!()
    }

    fn render_error(&self, _command: &'static str, _e: &TuskError) -> std::io::Result<()> {
        todo!()
    }
}

impl MarkdownRenderer {
    fn build_title(&self, to_date: NaiveDate, migration_date: Option<NaiveDate>) -> String {
        let to_date_s = to_date.format("%a %d %b %Y").to_string();

        let title = if let Some(migration_date) = migration_date {
            let from_date_s = migration_date.format("%a %d %b %Y").to_string();
            format!("# Migration from {} → {}", from_date_s, to_date_s)
        } else {
            format!("# Tasks for {}", to_date_s)
        };

        title
    }

    fn render_list(&self, out: &mut impl Write, items: &[Item]) -> std::io::Result<()> {
        for item in items {
            let is_done = item.done_at.is_some();
            let checkbox = if is_done { "- [x]" } else { "- [ ]" };

            let priority = format!("{}", item.priority);
            write!(out, "{checkbox} {} {}", item.text, priority)?;

            writeln!(out)?;
        }

        Ok(())
    }

    fn render_footer(&self, out: &mut impl Write, dayfile: &DayFile) -> std::io::Result<()> {
        let stats = stats(dayfile);

        writeln!(
            out,
            "\n{} task(s) ({} open, {} done)",
            stats.total, stats.open, stats.completed
        )?;

        Ok(())
    }
}
