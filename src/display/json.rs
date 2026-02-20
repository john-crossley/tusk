use std::io::{self, Write};

use serde::Serialize;

use crate::{
    display::renderer::Renderer,
    models::{dayfile::DayFile, item::Item},
};

pub struct JsonRenderer;

impl Renderer for JsonRenderer {
    fn render_day(&self, df: &DayFile) -> Result<(), std::io::Error> {
        Self::to_json(df)?;
        Ok(())
    }

    fn render_summary(&self, _index: Option<usize>, item: &Item) -> Result<(), std::io::Error> {
        Self::to_json(item)?;
        Ok(())
    }
}

impl JsonRenderer {
    fn to_json<T>(value: &T) -> Result<(), std::io::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut out = io::stdout().lock();
        serde_json::to_writer_pretty(&mut out, value)?;
        writeln!(out)?;

        Ok(())
    }
}
