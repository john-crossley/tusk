use colored::{ColoredString, Colorize};
use std::io::{self, Error, IsTerminal, Write};

use crate::models::{
    dayfile::DayFile,
    item::{Item, ItemPriority},
};

pub struct RenderOpts {
    pub json: bool,
    pub verbose: bool,
    pub no_color: bool,
    pub vault_name: Option<String>,
    pub dry_run: bool,
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
            ItemPriority::Low => "",
        };

        if !self.color {
            return g.normal();
        }

        match p {
            ItemPriority::High => g.red().bold(),
            ItemPriority::Medium => g.yellow().bold(),
            ItemPriority::Low => g.normal(),
        }
    }
}

pub fn render_migrate(dayfile: &DayFile, opts: RenderOpts) -> Result<(), Error> {
    let mut stdout = io::stdout().lock();

    if opts.json {
        return as_json(stdout, &dayfile);
    }

    let theme = Theme::new(opts.no_color);
    let title = build_title_header(&dayfile, opts.vault_name.as_deref(), true);
    title_underline(&theme, &title, &mut stdout)?;

    let items: Vec<_> = dayfile
        .items
        .iter()
        .filter(|i| i.migrated_from.is_some())
        .collect();

    if items.is_empty() {
        writeln!(
            &mut stdout,
            "🦣 {}",
            theme.dim(&format!("No tasks to migrate from {}", dayfile.date))
        )?;

        return Ok(());
    }

    render_list(&mut stdout, &dayfile.items, &theme, opts.verbose)?;
    render_migration_count(&mut stdout, &dayfile, &theme, opts.dry_run)?;

    Ok(())
}

fn render_migration_count(
    mut out: impl Write,
    dayfile: &DayFile,
    theme: &Theme,
    dry_run: bool,
) -> Result<(), Error> {
    let count = dayfile.items.len();

    if count == 0 {
        return Ok(());
    }

    let item_word = if count == 1 { "item" } else { "items" };
    let details = if dry_run {
        "will be migrated from"
    } else {
        "migrated from"
    };
    let date_str = dayfile.date.format("%a %d %b %Y").to_string();

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
    return Ok(());
}

pub fn render(dayfile: &DayFile, opts: RenderOpts) -> Result<(), Error> {
    let mut stdout = io::stdout().lock();

    if opts.json {
        return as_json(stdout, &dayfile);
    }

    let theme = Theme::new(opts.no_color);
    let title = build_title_header(&dayfile, opts.vault_name.as_deref(), false);
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

    if opts.json {
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

fn build_title_header(dayfile: &DayFile, vault_name: Option<&str>, migration: bool) -> String {
    let date_str = dayfile.date.format("%a %d %b %Y").to_string();
    let mut title = if migration {
        format!("Migration for: {}", date_str)
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
        let boxy = theme.checkbox(i.done_at.is_some());

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

        let prio = match i.priority {
            ItemPriority::Low => String::new(),
            _ => format!(" {}", theme.priority(&i.priority)),
        };

        if i.done_at.is_some() {
            write!(out, "{}{prio}", theme.dim(&line))?;
        } else {
            write!(out, "{line}{prio}")?;
        }

        if let Some(migrated_from) = i.migrated_from {
            let date_str = migrated_from.format("%a %d %b %Y").to_string();
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
