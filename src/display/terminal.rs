use chrono::{Days, NaiveDate};
use colored::Colorize;
use std::{
    io::{self, Error, Write},
    usize,
};

use crate::{
    display::renderer::Renderer,
    models::{dayfile::DayFile, item::Item},
    utils::{helpers::{item_count_meta, stats}, render::ActionKind, theme::Theme, tusk_error::TuskError},
};

const DATE_FORMAT: &str = "%a %d %b %Y";
const DATE_WITH_TIME_FORMAT: &str = "%Y-%m-%d %H:%M";

pub struct TerminalRenderer {
    pub theme: Theme,
    pub vault: Option<String>,
    pub verbose: bool,
}

impl Renderer for TerminalRenderer {
    fn render_day(&self, df: &DayFile) -> std::io::Result<()> {
        let mut out = io::stdout().lock();

        let title = self.build_title_header(df.date, None);
        Self::title_underline(&self.theme, &title, &mut out)?;

        if df.items.is_empty() {
            writeln!(
                out,
                "🦣 {}",
                self.theme.dim(&format!("No tasks for {}", df.date))
            )?;

            let hint = r#"tusk add "Drink more water 💦""#;
            writeln!(out, "   Add one with: {}", self.theme.ok(hint))?;

            return Ok(());
        }

        self.render_list(&mut out, &df.items)?;
        self.render_footer(&mut out, df)?;

        Ok(())
    }

    fn render_summary(&self, _date: NaiveDate, index: usize, item: &Item) -> std::io::Result<()> {
        let mut out = io::stdout().lock();

        // Header
        writeln!(
            out,
            "{}  {}",
            self.theme.info(&format!("#{}", index)),
            Self::format_text(&item.text, &self.theme)
        )?;

        // Priority
        writeln!(
            out,
            "    {} {}",
            self.theme.dim("Priority:"),
            self.theme.priority(&item.priority)
        )?;

        // Tags
        if !item.tags.is_empty() {
            let tags = item
                .tags
                .iter()
                .map(|t| format!("#{}", t))
                .collect::<Vec<_>>()
                .join("  ");

            writeln!(out, "    {} {}", self.theme.dim("Tags:"), tags)?;
        }

        // Created
        writeln!(
            out,
            "    {} {}",
            self.theme.dim("Created:"),
            item.created_at.format(DATE_WITH_TIME_FORMAT)
        )?;

        // Done
        let done_s = match &item.done_at {
            Some(ts) => ts.format(DATE_WITH_TIME_FORMAT).to_string(),
            None => "not yet".into(),
        };

        writeln!(out, "    {} {}", self.theme.dim("Done:"), done_s)?;

        if let Some(migrated_from) = item.migrated_from {
            writeln!(
                out,
                "    {} {}",
                self.theme.dim("Migrated from:"),
                migrated_from.format(DATE_FORMAT).to_string()
            )?;
        }

        // Notes
        if let Some(n) = &item.notes {
            writeln!(out, "    {} ", self.theme.dim("Notes:"))?;
            for line in n.lines() {
                writeln!(out, "      {}", line)?;
            }
        }

        Ok(())
    }

    fn render_migrate(
        &self,
        to_date: NaiveDate,
        from_df_original: &DayFile,
        moved_items: &[Item],
        dry_run: bool,
    ) -> Result<(), Error> {
        let mut out = io::stdout().lock();

        let title = self.build_title_header(to_date, Some(from_df_original.date));
        Self::title_underline(&self.theme, &title, &mut out)?;

        if moved_items.is_empty() {
            writeln!(out, "🦣 {}", self.theme.dim("No tasks to migrate."))?;

            return Ok(());
        }

        self.render_list(&mut out, moved_items)?;
        self.render_migratation_count(&mut out, moved_items, from_df_original.date, dry_run)?;

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

        // Build title
        let start_s = start.format(DATE_FORMAT).to_string();
        let end_s = display_end.format(DATE_FORMAT).to_string();

        let raw_title = format!("Review: {start_s} → {end_s}");

        let styled_title = format!(
            "{} {} {} {}",
            self.theme.title("Review:"),
            self.theme.info(&start_s),
            self.theme.dim("→"),
            self.theme.info(&end_s),
        );

        Self::title_underline_styled(&raw_title, &styled_title, &mut out)?;

        // end build

        writeln!(
            &mut out,
            "Last {} {} {}\n",
            self.theme.info(&days.to_string()),
            if days == 1 { "day" } else { "days" },
            self.theme.dim("(excluding today)"),
        )?;

        let count = item_count_meta(dayfiles);

        writeln!(&mut out, "Summary")?;
        writeln!(
            &mut out,
            "  {} {}",
            self.theme.dim("Total:"),
            self.theme.info(&count.total.to_string())
        )?;
        writeln!(
            &mut out,
            "  {} {}",
            self.theme.dim("Open:"),
            self.theme.warn(&count.open.to_string())
        )?;
        writeln!(
            &mut out,
            "  {} {}",
            self.theme.dim("Completed:"),
            self.theme.ok(&count.complete.to_string())
        )?;
        writeln!(
            &mut out,
            "  {} {}",
            self.theme.dim("Active days:"),
            self.theme.info(&dayfiles.len().to_string())
        )?;
        writeln!(&mut out)?;

        for df in dayfiles {
            let total_item_count: usize = df.items.len();
            let total_open_item_count: usize =
                df.items.iter().filter(|i| i.done_at.is_none()).count();

            let title = format!(
                "{} • {} task(s) ({} open, {} done)",
                df.date.format(DATE_FORMAT),
                total_item_count,
                total_open_item_count,
                total_item_count - total_open_item_count
            );

            Self::title_underline(&self.theme, &title, &mut out)?;

            for (index, item) in df.items.iter().enumerate() {
                let is_done = item.done_at.is_some();
                let next_index = (index + 1).to_string();

                let idx = if is_done {
                    self.theme.dim(&next_index)
                } else {
                    self.theme.plain(&next_index)
                };

                let text = if is_done {
                    self.theme.dim(&item.text)
                } else {
                    self.theme.plain(&item.text)
                };

                writeln!(
                    &mut out,
                    "{}. {} {} {}",
                    idx,
                    self.theme.checkbox(is_done),
                    text,
                    self.theme.priority(&item.priority)
                )?;
            }
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
            self.theme.error("error:"),
            e,
            self.theme.dim(format!("(command: {command})"))
        )?;

        if let Some(hint) = e.hint() {
            writeln!(err, "{} {}", self.theme.hint("hint:"), hint)?;
        }

        Ok(())
    }
}

impl TerminalRenderer {
    fn build_title_header(&self, to_date: NaiveDate, migration_date: Option<NaiveDate>) -> String {
        let date_str = to_date.format("%a %d %b %Y").to_string();

        let mut title = if let Some(date) = migration_date {
            let from_date_str = date.format("%a %d %b %Y").to_string();
            format!(
                "Migration from {} → {}",
                self.theme.info(&from_date_str),
                self.theme.info(&date_str)
            )
        } else {
            format!("Tasks for {}", date_str)
        };

        if let Some(v) = &self.vault {
            title.push_str(&format!(" • vault: {}", v));
        }

        title
    }

    fn render_list(&self, out: &mut impl Write, items: &[Item]) -> Result<(), Error> {
        let width = items.len().to_string().len();

        for (idx, i) in items.iter().enumerate() {
            let n = idx + 1;
            let is_done = i.done_at.is_some();
            let boxy = self.theme.checkbox(is_done);

            let short_id = if self.verbose {
                let id = format!("({})", Self::abbrev_id(&i.id, 6));
                self.theme.dim(&id).to_string()
            } else {
                String::new()
            };

            let spacer = if short_id.is_empty() { "" } else { " " };
            let line = format!(
                "{n:>width$}. {boxy} {short_id}{spacer}{}",
                Self::format_text(&i.text, &self.theme),
                width = width
            );

            let prio = format!(" {}", self.theme.priority(&i.priority));

            if is_done {
                write!(out, "{}{prio}", self.theme.dim(&line))?;
            } else {
                write!(out, "{line}{prio}")?;
            }

            if let Some(migrated_from) = i.migrated_from {
                let date_str = migrated_from.format("%a, %d %b").to_string();
                write!(out, "  ↪ {}", self.theme.dim(&date_str))?;
            }

            writeln!(out)?;
        }

        Ok(())
    }

    fn render_footer(&self, out: &mut impl Write, dayfile: &DayFile) -> Result<(), Error> {
        let stats = stats(dayfile);

        writeln!(
            out,
            "\n{} task(s) ({} open, {} done)",
            &self.theme.info(stats.total),
            &self.theme.warn(stats.open),
            &self.theme.ok(stats.completed)
        )?;

        Ok(())
    }

    fn render_migratation_count(
        &self,
        out: &mut impl Write,
        items: &[Item],
        date: NaiveDate,
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
            self.theme.info(&count.to_string()),
            item_word,
            details,
            self.theme.info(&date_str)
        )?;

        Ok(())
    }

    // Utils

    fn title_underline(theme: &Theme, title: &str, out: &mut impl Write) -> Result<(), Error> {
        writeln!(out)?;
        writeln!(out, "{}", theme.title(title))?;
        let underline = "_".repeat(title.chars().count());
        writeln!(out, "{underline}")?;

        Ok(())
    }

    fn title_underline_styled(
        raw_title: &str,
        styled_title: &str,
        out: &mut impl Write,
    ) -> io::Result<()> {
        writeln!(out)?;
        writeln!(out, "{styled_title}")?;

        let underline_len = raw_title.chars().count();
        writeln!(out, "{}", "─".repeat(underline_len))?;
        Ok(())
    }

    fn abbrev_id(id: &str, len: usize) -> String {
        id.chars().take(len).collect()
    }

    fn format_text(s: &str, theme: &Theme) -> String {
        s.split_whitespace()
            .map(|w| {
                if w.starts_with("#") {
                    theme.info(w).to_string()
                } else {
                    w.normal().to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}
