use chrono::NaiveDate;

use crate::models::{dayfile::DayFile, focus_file::FocusFile};

pub struct Agenda {
    pub date: NaiveDate,
    pub dayfile: Option<DayFile>,
    pub focusfile: Option<FocusFile>,
}

impl Agenda {
    pub fn new(date: NaiveDate, dayfile: Option<DayFile>, focusfile: Option<FocusFile>) -> Self {
        Self { date, dayfile, focusfile }
    }
}
