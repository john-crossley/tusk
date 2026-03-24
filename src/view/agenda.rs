use chrono::NaiveDate;

use crate::{
    models::{dayfile::DayFile, focus_file::FocusFile},
    utils::helpers::SummaryStats,
};

pub struct Agenda {
    pub date: NaiveDate,
    pub dayfile: Option<DayFile>,
    pub focusfile: Option<FocusFile>,
}

impl Agenda {
    pub fn new(date: NaiveDate, dayfile: Option<DayFile>, focusfile: Option<FocusFile>) -> Self {
        Self {
            date,
            dayfile,
            focusfile,
        }
    }

    pub fn stats(&self) -> SummaryStats {
        let df_stats = self
            .dayfile
            .as_ref()
            .map(SummaryStats::from)
            .unwrap_or_default();

        let ff_stats = self
            .focusfile
            .as_ref()
            .map(SummaryStats::from)
            .unwrap_or_default();

        df_stats + ff_stats
    }
}
