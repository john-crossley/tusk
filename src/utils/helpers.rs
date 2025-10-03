use std::{io::{self, Error}, path::PathBuf};

use chrono::NaiveDate;

use crate::{models::item::ItemPriority, utils::{dates::today_date, files::resolve_day_file_path}, Cli};

pub fn validate_index(i: usize, len: usize) -> io::Result<usize> {
    if i == 0 || i > len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("item at index {} does not exist.", i),
        ));
    }

    Ok(i - 1)
}

pub fn current_day_context(cli: &Cli) -> Result<(NaiveDate, PathBuf), Error> {
    let date = cli.date.unwrap_or_else(today_date);

    let path = resolve_day_file_path(
        &date,
        cli.data_dir.as_deref(),
        cli.verbose,
        cli.vault.as_deref(),
    )?;

    Ok((date, path))
}

pub fn sanitise_str(text: &str) -> io::Result<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Oops, did you forget to add some text?",
        ))
    } else {
        Ok(trimmed.to_owned())
    }
}

pub fn get_item_priority(priority: Option<&str>) -> ItemPriority {
    match priority.map(|p| p.to_lowercase()) {
        Some(ref p) if p == "high" => ItemPriority::High,
        Some(ref p) if p == "med" || p == "medium" => ItemPriority::Medium,
        _ => ItemPriority::Low,
    }
}

pub fn extract_tags(s: &str) -> Vec<String> {
    s.split_whitespace()
        .filter_map(|w| w.strip_prefix('#').map(|t| t.to_string()))
        .collect()
}
