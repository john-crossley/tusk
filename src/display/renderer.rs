use std::io::Error;

use chrono::NaiveDate;

use crate::models::{dayfile::DayFile, item::Item};

pub trait Renderer {
    fn render_day(&self, df: &DayFile) -> Result<(), Error>;

    fn render_summary(&self, index: Option<usize>, item: &Item) -> Result<(), Error>;

    fn render_migrate(
        &self,
        to_df: &DayFile,
        from_df: &DayFile,
        items: &[Item],
        dry_run: bool,
    ) -> Result<(), Error>;

    fn render_review(
        &self,
        start: &NaiveDate,
        end: &NaiveDate,
        days: u64,
        dayfiles: &[DayFile],
    ) -> Result<(), Error>;
}
