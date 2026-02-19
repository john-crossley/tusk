use std::io::{self, Write};

use crate::display::renderer::Renderer;

pub struct JsonRenderer;

impl Renderer for JsonRenderer {
    fn render_day(&self, df: &crate::models::dayfile::DayFile) -> Result<(), std::io::Error> {
        
        let mut out = io::stdout().lock();
        serde_json::to_writer_pretty(&mut out, &df)?;
        writeln!(out)?;

        Ok(())
    }
}