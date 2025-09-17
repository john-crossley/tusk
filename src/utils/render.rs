use colored::{ColoredString, Colorize};
use std::io::{self, Error, IsTerminal, Write};

use crate::models::{dayfile::DayFile, item::{Item, ItemPriority}};

pub struct RenderOpts {
    pub json: bool,
    pub verbose: bool,
    pub no_color: bool,
    pub vault_name: Option<String>,
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
            s.blue().bold()
        } else {
            s.normal()
        }
    }

    fn checkbox(&self, done: bool) -> &'static str {
        if done { "[x]" } else { "[ ]" }
    }

    fn priority(&self, p: &ItemPriority) -> ColoredString {
        let t = match p {
            ItemPriority::High => "!high",
            ItemPriority::Medium => "!med",
            ItemPriority::Low => "!low"
        };

        if !self.color {
            return t.normal();
        }

        match p {
            ItemPriority::High => t.red().bold(),
            ItemPriority::Medium => t.yellow().bold(),
            ItemPriority::Low => t.normal()
        }
    }
}

pub fn render(dayfile: &DayFile, opts: RenderOpts) -> Result<(), Error> {
    let mut stdout = io::stdout().lock();

    if opts.json {
        serde_json::to_writer_pretty(&mut stdout, &dayfile)?;
        writeln!(&mut stdout)?;
        return Ok(());
    }

    let theme = Theme::new(opts.no_color);
    let date_str = dayfile.date.format("%a %d %b %Y").to_string();
    let mut title = format!("Tasks for: {}", date_str);

    if let Some(v) = &opts.vault_name {
        title.push_str(&format!(" • vault: {}", v));
    }

    writeln!(&mut stdout)?;
    writeln!(&mut stdout, "{}", theme.title(&title))?;
    writeln!(&mut stdout, "{}", repeat_char('-', title.len()))?;

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

    let completed = dayfile.items.iter().filter(|i| i.done_at.is_some()).count();

    for (idx, i) in dayfile.items.iter().enumerate() {
        let n = idx + 1;
        let boxy = theme.checkbox(i.done_at.is_some());

        let short_id = if opts.verbose {
            format!("({})", abbrev_id(&i.id, 6))
        } else {
            String::new()
        };

        let space_after_id = if short_id.is_empty() { "" } else { " " };
        let line = format!("{n:>2}. {boxy} {short_id}{space_after_id}{} ", i.text);
        
        let priority = match i.priority {
            ItemPriority::Low => "".normal(),
            _ => theme.priority(&i.priority)
        };

        if i.done_at.is_some() {
            write!(&mut stdout, "{}", theme.dim(&line))?;
            writeln!(&mut stdout, "{}", priority)?;
        } else {
            write!(&mut stdout, "{line}")?;
            writeln!(&mut stdout, "{}", priority)?;
        }
    }

    // 🦶er
    let total = dayfile.items.len();
    let open = total - completed;

    writeln!(
        &mut stdout,
        "\n{} task(s) ({} open, {} done)",
        theme.info(&total.to_string()),
        theme.warn(&open.to_string()),
        theme.ok(&completed.to_string())
    )?;

    Ok(())
}

pub fn render_summary(item: &Item, json: bool, no_colour: bool) -> io::Result<()> {
    let mut out = io::stdout().lock();

    if json {
        serde_json::to_writer_pretty(&mut out, &item)?;
        writeln!(&mut out)?;
        return Ok(());
    }

    let theme = Theme::new(no_colour);
    let msg = format!(
        "Added  {}. {}",
        format!("{}", item.index).to_string(),
        &item.text
    );
    writeln!(&mut out, "{}", theme.ok(&msg))?;
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
