use std::{
    io::{self, Error},
    path::PathBuf,
};

use chrono::NaiveDate;

use crate::{
    Cli,
    models::{dayfile::DayFile, item::ItemPriority},
    utils::{dates::todays_date, files::resolve_day_file_path},
};

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
    let date = cli.date.unwrap_or_else(todays_date);

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

pub fn warn_dayfile_error(
    date: chrono::NaiveDate,
    path: &std::path::Path,
    err: &std::io::Error,
    verbose: bool,
) {
    use std::io::ErrorKind;

    if err.kind() == ErrorKind::NotFound {
        return;
    }

    if verbose {
        eprintln!(
            "warn: {} — failed to load dayfile\n      path: {}\n      error: {}",
            date,
            path.display(),
            err
        );
    }
}

pub struct ItemCountResult {
    pub open: usize,
    pub complete: usize,
    pub total: usize,
}

pub fn item_count_meta(dayfiles: &[DayFile]) -> ItemCountResult {
    let open_count: usize = dayfiles
        .iter()
        .map(|d| d.items.iter().filter(|i| i.done_at.is_none()).count())
        .sum();

    let complete_count: usize = dayfiles
        .iter()
        .map(|d| d.items.iter().filter(|i| i.done_at.is_some()).count())
        .sum();

    let total = open_count + complete_count;

    ItemCountResult {
        open: open_count,
        complete: complete_count,
        total,
    }
}
