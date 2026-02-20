use crate::{
    display::renderer::Renderer,
    models::{dayfile::DayFile, item::Item},
};

pub struct MarkdownRenderer;

impl Renderer for MarkdownRenderer {
    fn render_day(&self, _df: &crate::models::dayfile::DayFile) -> Result<(), std::io::Error> {
        todo!()
    }

    fn render_summary(&self, _index: Option<usize>, _item: &Item) -> Result<(), std::io::Error> {
        todo!()
    }

    fn render_migrate(
        &self,
        _to_df: &DayFile,
        _from_df: &DayFile,
        _items: &[Item],
        _dry_run: bool,
    ) -> Result<(), std::io::Error> {
        todo!()
    }
}
