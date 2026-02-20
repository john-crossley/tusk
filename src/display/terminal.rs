use chrono::NaiveDate;
use colored::Colorize;
use std::io::{self, Error, Write};

use crate::{
    display::renderer::Renderer,
    models::{dayfile::DayFile, item::Item},
    utils::theme::Theme,
};

pub struct TerminalRenderer {
    pub theme: Theme,
    pub vault: Option<String>,
    pub verbose: bool,
}

impl Renderer for TerminalRenderer {
    fn render_day(&self, df: &DayFile) -> Result<(), Error> {
        let mut out = io::stdout().lock();

        let title = self.build_title_header(df, None);
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

    fn render_summary(&self, index: Option<usize>, item: &Item) -> Result<(), Error> {
        let mut out = io::stdout().lock();
        let index = index.map(|i| i.to_string()).unwrap_or(item.id.to_string());

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
            item.created_at.format("%Y-%m-%d %H:%M")
        )?;

        // Done
        let done_s = match &item.done_at {
            Some(ts) => ts.format("%Y-%m-%d %H:%M").to_string(),
            None => "not yet".into(),
        };

        writeln!(out, "    {} {}", self.theme.dim("Done:"), done_s)?;

        if let Some(migrated_from) = item.migrated_from {
            writeln!(
                out,
                "    {} {}",
                self.theme.dim("Migrated from:"),
                migrated_from.format("%Y-%m-%d").to_string()
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
        to_df: &DayFile,
        from_df: &DayFile,
        items: &[Item],
        dry_run: bool,
    ) -> Result<(), Error> {
        let mut out = io::stdout().lock();

        let title = self.build_title_header(to_df, Some(from_df));
        Self::title_underline(&self.theme, &title, &mut out)?;

        if items.is_empty() {
            writeln!(
                out,
                "🦣 {}",
                self.theme
                    .dim(&format!("No tasks to migrate from {} 🐘", from_df.date))
            )?;

            return Ok(());
        }

        self.render_list(&mut out, items)?;
        self.render_migratation_count(&mut out, items, from_df.date, dry_run)?;

        Ok(())
    }
}

impl TerminalRenderer {
    fn build_title_header(&self, df: &DayFile, migration: Option<&DayFile>) -> String {
        let date_str = df.date.format("%a %d %b %Y").to_string();

        let mut title = if let Some(from_df) = migration {
            let from_date_str = from_df.date.format("%a %d %b %Y").to_string();
            format!(
                "Migration from {} → {}",
                self.theme.info(&from_date_str),
                self.theme.info(&date_str)
            )
        } else {
            format!("Tasks for: {}", date_str)
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
        let completed = dayfile.items.iter().filter(|i| i.done_at.is_some()).count();
        let total = dayfile.items.len();
        let open = total - completed;

        writeln!(
            out,
            "\n{} task(s) ({} open, {} done)",
            &self.theme.info(&total.to_string()),
            &self.theme.warn(&open.to_string()),
            &self.theme.ok(&completed.to_string())
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
        let underline = Self::repeat_char('-', title.len());

        writeln!(out)?;
        writeln!(out, "{}", theme.title(&title))?;
        writeln!(out, "{}", underline)?;
        Ok(())
    }

    fn repeat_char(c: char, n: usize) -> String {
        let mut s = String::with_capacity(n);
        for _ in 0..n {
            s.push(c);
        }
        s
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
