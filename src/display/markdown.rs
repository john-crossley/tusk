use crate::{display::renderer::Renderer, models::{dayfile::DayFile, item::Item}};

pub struct MarkdownRenderer;

impl Renderer for MarkdownRenderer {
    fn render_day(&self, df: &crate::models::dayfile::DayFile) -> Result<(), std::io::Error> {
        println!("Markdown Renderer");

        Ok(())
    }

    fn render_summary(&self, index: Option<usize>, item: &Item) -> Result<(), std::io::Error> {
        todo!()
    }
}