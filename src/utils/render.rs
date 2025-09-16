use std::io::{self, Error, Write};
use colored::Colorize;

use crate::{models::dayfile::DayFile};

pub fn render(dayfile: &DayFile, json: bool, verbose: bool) -> Result<(), Error> {
    let mut stdout = io::stdout().lock();

    if json {
        serde_json::to_writer_pretty(&mut stdout, &dayfile)?;
        writeln!(&mut stdout)?;
        stdout.flush()?;
        return Ok(());
    }

    let title = format!("Tasks for: {}", dayfile.date.format("%a %d %b %Y"));
    writeln!(&mut stdout, "{}", title.bold())?;
    writeln!(&mut stdout, "{}", repeat_char('-', title.len()))?;

    if dayfile.items.is_empty() {
        writeln!(
            &mut stdout,
            "🦣 No tasks added for {}\n\tAdd one with: {}",
            dayfile.date,
            "tusk add \"Plant more trees 🌳\"".green().bold()
        )?;

        stdout.flush()?;
        return Ok(());
    }

    let completed = dayfile.items.iter().filter(|i| i.done_at.is_some()).count();
    let total_item_count = dayfile.items.len();

    for (idx, i) in dayfile.items.iter().enumerate() {
        let n = idx + 1;
        let boxy = if i.done_at.is_some() { "[x]" } else { "[ ]" };
        let short_id = if verbose {
            format!(" {}", abbrev_id(&i.id, 6))
        } else {
            String::new()
        };

        writeln!(&mut stdout, "{n:>2}. {boxy}{short_id} {}", i.text)?;
    }

    // 🦶er
    writeln!(
        &mut stdout,
        "\n{} task(s) ({} open, {} done)",
        total_item_count,
        total_item_count - completed,
        completed
    )?;

    stdout.flush()?;

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
