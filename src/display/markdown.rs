use chrono::NaiveDate;

use crate::{
    display::renderer::Renderer,
    models::{dayfile::DayFile, item::Item},
    utils::{render::ActionKind, tusk_error::TuskError},
};

pub struct MarkdownRenderer;

impl Renderer for MarkdownRenderer {
    fn render_day(&self, _df: &DayFile) -> std::io::Result<()> {
        todo!()
    }

    fn render_summary(&self, _date: NaiveDate, _index: usize, _item: &Item) -> std::io::Result<()> {
        todo!()
    }

    fn render_migrate(
        &self,
        _to_date: NaiveDate,
        _from_df_original: &DayFile,
        _moved_items: &[Item],
        _dry_run: bool,
    ) -> std::io::Result<()> {
        todo!()
    }

    fn render_review(
        &self,
        _start: NaiveDate,
        _end: NaiveDate,
        _days: u64,
        _dayfiles: &[DayFile],
    ) -> std::io::Result<()> {
        todo!()
    }

    fn render_action(
        &self,
        _index: usize,
        _date: NaiveDate,
        _action: ActionKind,
        _item: Option<&Item>,
    ) -> std::io::Result<()> {
        todo!()
    }

    fn render_error(&self, _command: &'static str, _e: &TuskError) -> std::io::Result<()> {
        todo!()
    }
}
