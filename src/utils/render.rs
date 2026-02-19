use chrono::NaiveDate;
use colored::{ColoredString, Colorize};
use std::io::{self, Error, IsTerminal, Write};

use crate::models::{
    dayfile::DayFile,
    item::{Item, ItemPriority},
};

#[derive(Clone, PartialEq)]
pub enum RenderOutput {
    Json,
    Markdown
}

#[derive(Clone)]
pub struct RenderOpts {
    pub output: RenderOutput,
    pub verbose: bool,
    pub no_color: bool,
    pub vault_name: Option<String>,
    pub dry_run: bool,
    pub markdown: bool,
}

impl Default for RenderOpts {
    fn default() -> Self {
        RenderOpts {
            output: RenderOutput::Json,
            verbose: false,
            no_color: true,
            vault_name: None,
            dry_run: false,
            markdown: false,
        }
    }
}

struct Theme {
    color: bool,
}

impl Theme {
    fn new(no_color: bool) -> Self {
        let color = io::stdout().is_terminal() && !no_color;
        Self { color }
    }

    fn title(&self, s: &str) -> ColoredString {
        if self.color { s.bold() } else { s.normal() }
    }

    fn dim(&self, s: &str) -> ColoredString {
        if self.color { s.dimmed() } else { s.normal() }
    }

    fn ok(&self, s: &str) -> ColoredString {
        if self.color {
            s.green().bold()
        } else {
            s.normal()
        }
    }

    fn warn(&self, s: &str) -> ColoredString {
        if self.color {
            s.yellow().bold()
        } else {
            s.normal()
        }
    }

    fn info(&self, s: &str) -> ColoredString {
        if self.color {
            s.blue().dimmed().bold()
        } else {
            s.normal()
        }
    }

    fn info_em(&self, s: &str) -> ColoredString {
        if !self.color {
            return s.normal();
        }

        s.blue().dimmed().bold().italic()
    }

    fn checkbox(&self, done: bool) -> &'static str {
        if self.color && io::stdout().is_terminal() {
            if done { "☑" } else { "☐" }
        } else {
            if done { "[x]" } else { "[ ]" }
        }
    }

    fn priority(&self, p: &ItemPriority) -> ColoredString {
        let g = match p {
            ItemPriority::High => "‼",
            ItemPriority::Medium => "▲",
            ItemPriority::Low => "▽",
        };

        if !self.color {
            return g.normal();
        }

        match p {
            ItemPriority::High => g.red().bold(),
            ItemPriority::Medium => g.yellow().bold(),
            ItemPriority::Low => g.dimmed(),
        }
    }
}

pub fn render_migrate(
    to_df: &DayFile,
    from_df: &DayFile,
    items: &[Item],
    opts: RenderOpts,
) -> Result<(), Error> {
    let mut stdout = io::stdout().lock();

    if opts.output == RenderOutput::Json {
        return as_json(stdout, &to_df);
    }

    let theme = Theme::new(opts.no_color);
    let title = build_title_header(&to_df, opts.vault_name.as_deref(), Some(from_df), &theme);
    title_underline(&theme, &title, &mut stdout)?;

    if items.is_empty() {
        writeln!(
            &mut stdout,
            "🦣 {}",
            theme.dim(&format!("No tasks to migrate from {}", from_df.date))
        )?;

        return Ok(());
    }

    render_list(&mut stdout, items, &theme, opts.verbose)?;
    render_migration_count(&mut stdout, items, from_df.date, &theme, opts.dry_run)?;

    Ok(())
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

pub fn as_json(mut out: impl Write, dayfile: &DayFile) -> Result<(), Error> {
    serde_json::to_writer_pretty(&mut out, &dayfile)?;
    writeln!(out)?;

    Ok(())
}

pub fn render(dayfile: &DayFile, opts: &RenderOpts) -> Result<(), Error> {
    let mut stdout = io::stdout().lock();

    if opts.output == RenderOutput::Json {
        return as_json(stdout, &dayfile);
    }

    let theme = Theme::new(opts.no_color);
    let title = build_title_header(&dayfile, opts.vault_name.as_deref(), None, &theme);
    title_underline(&theme, &title, &mut stdout)?;

    if dayfile.items.is_empty() {
        writeln!(
            &mut stdout,
            "🦣 {}",
            theme.dim(&format!("No tasks for {}", dayfile.date))
        )?;

        let hint = r#"tusk add "Plant more trees 🌳""#;
        writeln!(&mut stdout, "  Add one with: {}", theme.ok(hint))?;

        return Ok(());
    }

    render_list(&mut stdout, &dayfile.items, &theme, opts.verbose)?;

    render_footer(&mut stdout, &dayfile, &theme)?;

    Ok(())
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

pub fn render_summary(idx: Option<usize>, item: &Item, opts: RenderOpts) -> io::Result<()> {
    let mut out = io::stdout().lock();

    if opts.output == RenderOutput::Json {
        serde_json::to_writer_pretty(&mut out, &item)?;
        writeln!(&mut out)?;
        return Ok(());
    }

    let theme = Theme::new(opts.no_color);
    let index = idx.map(|i| i.to_string()).unwrap_or(item.id.to_string());

    // Header
    writeln!(
        &mut out,
        "{}  {}",
        theme.info(&format!("#{}", index)),
        format_text(&item.text, &theme)
    )?;

    // Priority
    writeln!(
        &mut out,
        "    {} {}",
        theme.dim("Priority:"),
        theme.priority(&item.priority)
    )?;

    // Tags
    if !item.tags.is_empty() {
        let tags = item
            .tags
            .iter()
            .map(|t| format!("#{}", t))
            .collect::<Vec<_>>()
            .join("  ");

        writeln!(&mut out, "    {} {}", theme.dim("Tags:"), tags)?;
    }

    // Created
    writeln!(
        &mut out,
        "    {} {}",
        theme.dim("Created:"),
        item.created_at.format("%Y-%m-%d %H:%M")
    )?;

    // Done
    let done_s = match &item.done_at {
        Some(ts) => ts.format("%Y-%m-%d %H:%M").to_string(),
        None => "not yet".into(),
    };

    writeln!(&mut out, "    {} {}", theme.dim("Done:"), done_s)?;

    if let Some(migrated_from) = item.migrated_from {
        writeln!(
            &mut out,
            "    {} {}",
            theme.dim("Migrated from:"),
            migrated_from.format("%Y-%m-%d").to_string()
        )?;
    }

    // Notes
    if let Some(n) = &item.notes {
        writeln!(&mut out, "    {} ", theme.dim("Notes:"))?;
        for line in n.lines() {
            writeln!(&mut out, "      {}", line)?;
        }
    }

    Ok(())
}

fn abbrev_id(id: &str, len: usize) -> String {
    let mut it = id.chars();
    let mut s = String::with_capacity(len);
    for _ in 0..len {
        if let Some(ch) = it.next() {
            s.push(ch);
        } else {
            break;
        }
    }

    s
}

fn repeat_char(c: char, n: usize) -> String {
    let mut s = String::with_capacity(n);
    for _ in 0..n {
        s.push(c);
    }
    s
}

pub fn render_review_title(
    start: &NaiveDate,
    end: &NaiveDate,
    days: u64,
    opts: &RenderOpts,
) -> io::Result<()> {
    let mut out = io::stdout().lock();

    let theme = Theme::new(opts.no_color);

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
    let theme = Theme::new(opts.no_color);

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

fn build_title_header(
    df: &DayFile,
    vault_name: Option<&str>,
    migration: Option<&DayFile>,
    theme: &Theme,
) -> String {
    let date_str = df.date.format("%a %d %b %Y").to_string();

    let mut title = if let Some(from_df) = migration {
        let from_date_str = from_df.date.format("%a %d %b %Y").to_string();
        format!(
            "Migration from {} → {}",
            theme.info(&from_date_str),
            theme.info(&date_str)
        )
    } else {
        format!("Tasks for: {}", date_str)
    };

    if let Some(v) = vault_name {
        title.push_str(&format!(" • vault: {}", v));
    }

    title
}

fn title_underline(theme: &Theme, title: &str, mut out: impl Write) -> Result<(), Error> {
    writeln!(&mut out)?;
    writeln!(&mut out, "{}", theme.title(&title))?;
    writeln!(&mut out, "{}", repeat_char('-', title.len()))?;
    Ok(())
}

fn render_list(
    mut out: impl Write,
    items: &[Item],
    theme: &Theme,
    verbose: bool,
) -> Result<(), Error> {
    let width = items.len().to_string().len();

    for (idx, i) in items.iter().enumerate() {
        let n = idx + 1;
        let is_done = i.done_at.is_some();
        let boxy = theme.checkbox(is_done);

        let short_id = if verbose {
            let id = format!("({})", abbrev_id(&i.id, 6));
            theme.dim(&id).to_string()
        } else {
            String::new()
        };

        let spacer = if short_id.is_empty() { "" } else { " " };
        let line = format!(
            "{n:>width$}. {boxy} {short_id}{spacer}{}",
            format_text(&i.text, theme),
            width = width
        );

        let prio = format!(" {}", theme.priority(&i.priority));

        if is_done {
            write!(out, "{}{prio}", theme.dim(&line))?;
        } else {
            write!(out, "{line}{prio}")?;
        }

        if let Some(migrated_from) = i.migrated_from {
            let date_str = migrated_from.format("%a, %d %b").to_string();
            write!(out, "  ↪ {}", theme.dim(&date_str))?;
        }

        writeln!(&mut out)?;
    }

    Ok(())
}

fn render_footer(mut out: impl Write, dayfile: &DayFile, theme: &Theme) -> Result<(), Error> {
    let completed = dayfile.items.iter().filter(|i| i.done_at.is_some()).count();
    let total = dayfile.items.len();
    let open = total - completed;

    writeln!(
        &mut out,
        "\n{} task(s) ({} open, {} done)",
        theme.info(&total.to_string()),
        theme.warn(&open.to_string()),
        theme.ok(&completed.to_string())
    )?;

    Ok(())
}
