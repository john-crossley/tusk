use std::io::Error;

use crate::models::{dayfile::DayFile, item::Item};

pub trait Renderer {
    fn render_day(&self, df: &DayFile) -> Result<(), Error>;
    fn render_summary(&self, index: Option<usize>, item: &Item) -> Result<(), std::io::Error>;
}
