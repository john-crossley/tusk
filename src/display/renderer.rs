use std::io::Error;

use crate::models::dayfile::DayFile;

pub trait Renderer {
    fn render_day(&self, df: &DayFile) -> Result<(), Error>;
}
