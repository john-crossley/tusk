use std::{io::Error, path::PathBuf};

use chrono::NaiveDate;

use crate::{
    Cli,
    models::dayfile::DayFile,
    utils::{dates::todays_date, files::resolve_day_file_path, tusk_error::TuskError},
};

pub fn validate_index(i: usize, len: usize) -> Result<usize, TuskError> {
    if i == 0 || i > len {
        return Err(TuskError::IndexOutOfRange { index: i, max: len });
    }

    Ok(i - 1)
}

pub fn current_day_context(
    cli: &Cli,
    date: Option<NaiveDate>,
) -> Result<(NaiveDate, PathBuf), Error> {
    let date = date.unwrap_or_else(todays_date);

    let path = resolve_day_file_path(
        &date,
        cli.data_dir.as_deref(),
        cli.verbose,
        cli.vault.as_deref(),
    )?;

    Ok((date, path))
}

pub fn sanitise_str(text: &str) -> Result<String, TuskError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        Err(TuskError::InvalidInput {
            message: "Oops, did you forget to add some text?".to_string(),
        })
    } else {
        Ok(trimmed.to_owned())
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
