use std::io::{self, Write};

use chrono::{Days, NaiveDate};

use crate::{
    display::{
        renderer::Renderer,
        terminal::{DATE_FORMAT, DATE_WITH_TIME_FORMAT},
    },
    models::{dayfile::DayFile, item::Item},
    utils::{
        helpers::{SummaryStats, item_count_meta},
        render::ActionKind,
        tusk_error::TuskError,
    },
    view::agenda::Agenda,
};

pub struct MarkdownRenderer;

impl Renderer for MarkdownRenderer {
    fn render_agenda(&self, agenda: &Agenda) -> std::io::Result<()> {
        let mut out = io::stdout().lock();

        Self::render_header(&mut out, agenda.date)?;

        if let Some(ff) = &agenda.focusfile {
            writeln!(out, "### Focus Tasks")?;
            self.render_list(&mut out, &ff.items)?;
            writeln!(out)?;
            self.render_footer(&mut out, ff.into())?;
        }

        if let Some(df) = &agenda.dayfile {
            writeln!(out)?;
            writeln!(out, "### Daily Tasks")?;
            self.render_list(&mut out, &df.items)?;
            writeln!(out)?;
            self.render_footer(&mut out, df.into())?;
        }

        writeln!(out)?;
        writeln!(out, "---")?;
        writeln!(out)?;
        self.render_footer(&mut out, agenda.stats())?;

        Ok(())
    }

    fn render_day(&self, df: &DayFile) -> std::io::Result<()> {
        let mut out = io::stdout().lock();

        Self::render_header(&mut out, df.date)?;

        if df.items.is_empty() {
            writeln!(
                out,
                "🦣 No tasks for {}",
                format!("No tasks for {}", df.date)
            )?;

            let hint = r#"tusk add "Drink more water 💦""#;
            writeln!(out, "_Add one with: {}_", hint)?;
        }

        self.render_list(&mut out, &df.items)?;
        writeln!(out)?;
        self.render_footer(&mut out, df.into())?;

        Ok(())
    }

    fn render_summary(
        &self,
        _date: Option<NaiveDate>,
        index: usize,
        item: &Item,
    ) -> std::io::Result<()> {
        let mut out = io::stdout().lock();

        let create_at = item.created_at.format(DATE_WITH_TIME_FORMAT);

        writeln!(out, "# #{} - {}", index, item.text)?;
        writeln!(out)?;

        writeln!(out, "**Status:** {}  ", item.status())?;
        writeln!(out, "**Priority:** {}  ", item.priority)?;
        writeln!(out, "**Created:** {}  ", create_at)?;

        if let Some(done_at) = item.done_at {
            writeln!(
                out,
                "**Completed:** {}  ",
                done_at.format(DATE_WITH_TIME_FORMAT)
            )?;
        }

        if let Some(n) = &item.notes {
            writeln!(out)?;
            writeln!(out, "---")?;
            writeln!(out)?;
            writeln!(out, "## Notes")?;
            writeln!(out)?;
            writeln!(out, "{n}")?;
        }

        Ok(())
    }

    fn render_migrate(
        &self,
        to_date: NaiveDate,
        from_df_original: &DayFile,
        moved_items: &[Item],
        dry_run: bool,
    ) -> std::io::Result<()> {
        let mut out = io::stdout().lock();

        writeln!(out, "# Migration")?;
        let title = self.build_date_line(to_date, Some(from_df_original.date));
        writeln!(out, "{}", title)?;
        writeln!(out)?;

        if moved_items.is_empty() {
            writeln!(out, "> 🦣 No tasks to migrate.")?;

            return Ok(());
        }

        self.render_list(&mut out, moved_items)?;
        self.render_migration_count(&mut out, moved_items, from_df_original.date, dry_run)?;

        Ok(())
    }

    fn render_review(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        days: u64,
        dayfiles: &[DayFile],
    ) -> std::io::Result<()> {
        let mut out = io::stdout().lock();

        let display_end = end
            .checked_sub_days(Days::new(1))
            .expect("end should always be at least one day after start");

        let start_s = start.format(DATE_FORMAT).to_string();
        let end_s = display_end.format(DATE_FORMAT).to_string();

        writeln!(out, "# Review: {} → {}", start_s, end_s)?;

        writeln!(
            &mut out,
            "> Last {} {} {}\n",
            days,
            if days == 1 { "day" } else { "days" },
            "(excluding today)",
        )?;

        let count = item_count_meta(dayfiles);

        writeln!(&mut out, "## Summary")?;
        writeln!(&mut out, "{} {}", "- **Total:**", count.total)?;
        writeln!(&mut out, "{} {}", "- **Open:**", count.open)?;
        writeln!(&mut out, "{} {}", "- **Completed:**", count.complete)?;
        writeln!(&mut out, "{} {}", "- **Active days:**", dayfiles.len())?;
        writeln!(&mut out)?;

        for df in dayfiles {
            let total_item_count: usize = df.items.len();

            let total_open_item_count: usize =
                df.items.iter().filter(|i| i.done_at.is_none()).count();

            writeln!(out)?;
            let title = format!("## {}", df.date.format(DATE_FORMAT));
            writeln!(out, "{}", title)?;

            writeln!(
                out,
                "**{} tasks** - **{} open** - **{} done**\n",
                total_item_count,
                total_open_item_count,
                total_item_count - total_open_item_count
            )?;

            for item in &df.items {
                let is_done = item.done_at.is_some();
                let text = if is_done {
                    format!("~~{}~~", item.text)
                } else {
                    format!("{}", item.text)
                };

                writeln!(
                    &mut out,
                    "{} {text} {}",
                    if is_done { "- [x]" } else { "- [ ]" },
                    format_args!("*({})*", item.priority)
                )?;
            }
            writeln!(out)?;
        }

        Ok(())
    }

    fn render_action(
        &self,
        _index: usize,
        _date: NaiveDate,
        _action: ActionKind,
        _item: Option<&Item>,
    ) -> std::io::Result<()> {
        Ok(())
    }

    fn render_error(&self, command: &'static str, e: &TuskError) -> std::io::Result<()> {
        let mut err = io::stderr().lock();

        writeln!(
            err,
            "{} {} {}",
            "error:",
            e,
            format!("(command: {command})")
        )?;

        if let Some(hint) = e.hint() {
            writeln!(err, "hint: {}", hint)?;
        }

        Ok(())
    }
}

impl MarkdownRenderer {

    fn render_header(out: &mut impl Write, date: NaiveDate) -> std::io::Result<()> {
        writeln!(out, "# Tasks")?;
        writeln!(out)?;
        writeln!(out, "## {}", date.format("%a %d %b %Y"))?;
        writeln!(out)
    }

    fn build_date_line(&self, to_date: NaiveDate, migration_date: Option<NaiveDate>) -> String {
        if let Some(migration_date) = migration_date {
            let from_date_s = migration_date.format("%a %d %b %Y");
            format!(
                "From **{}** → **{}**",
                from_date_s,
                to_date.format("%a %d %b %Y")
            )
        } else {
            format!("On **{}**", to_date.format("%a %d %b %Y"))
        }
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

    fn render_footer(&self, out: &mut impl Write, stats: SummaryStats) -> std::io::Result<()> {
        writeln!(
            out,
            "> **{} task(s)** ({} open, {} done)",
            stats.total, stats.open, stats.completed
        )?;

        Ok(())
    }

    fn render_migration_count(
        &self,
        out: &mut impl Write,
        items: &[Item],
        date: NaiveDate,
        dry_run: bool,
    ) -> std::io::Result<()> {
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

        writeln!(out)?;

        writeln!(
            out,
            "> {} {} {} {}",
            count,
            item_word,
            details,
            date.format("%a %d %b %Y")
        )?;

        Ok(())
    }
}
