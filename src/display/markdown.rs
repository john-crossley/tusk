use crate::display::renderer::Renderer;

pub struct MarkdownRenderer;

impl Renderer for MarkdownRenderer {
    fn render_day(&self, df: &crate::models::dayfile::DayFile) -> Result<(), std::io::Error> {
        println!("Markdown Renderer");

        Ok(())
    }
}