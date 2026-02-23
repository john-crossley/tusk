use chrono::NaiveDate;

use crate::{
    display::renderer::Renderer,
    models::{dayfile::DayFile, item::Item},
};

pub struct MarkdownRenderer;

impl Renderer for MarkdownRenderer {
    fn render_day(&self, _df: &DayFile) -> Result<(), std::io::Error> {
        todo!()
    }

    fn render_summary(
        &self,
        _date: NaiveDate,
        _index: usize,
        _item: &Item,
    ) -> Result<(), std::io::Error> {
        todo!()
    }

    fn render_migrate(
        &self,
        _to_date: NaiveDate,
        _from_df_original: &DayFile,
        _moved_items: &[Item],
        _dry_run: bool,
    ) -> Result<(), std::io::Error> {
        todo!()
    }

    fn render_review(
        &self,
        _start: &NaiveDate,
        _end: &NaiveDate,
        _days: u64,
        _dayfiles: &[DayFile],
    ) -> Result<(), std::io::Error> {
        todo!()
    }
}
