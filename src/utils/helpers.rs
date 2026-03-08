use crate::{models::dayfile::DayFile, utils::tusk_error::TuskError};

pub fn validate_index(i: usize, len: usize) -> Result<usize, TuskError> {
    if i == 0 || i > len {
        return Err(TuskError::IndexOutOfRange { index: i, max: len });
    }

    Ok(i - 1)
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
    err: &std::io::Error,
    verbose: bool,
) {
    use std::io::ErrorKind;

    if err.kind() == ErrorKind::NotFound {
        return;
    }

    if verbose {
        eprintln!(
            "warn: {} — failed to load dayfile\n     error: {}",
            date,
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

pub struct DayFileStats {
    pub completed: usize,
    pub total: usize,
    pub open: usize,
}

pub fn stats(df: &DayFile) -> DayFileStats {
    let completed = df.items.iter().filter(|i| i.done_at.is_some()).count();
    let total = df.items.len();
    let open = total - completed;

    DayFileStats {
        completed,
        total,
        open,
    }
}
