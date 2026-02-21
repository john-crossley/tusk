use std::io::{self, Write};

use chrono::NaiveDate;
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

    fn render_migrate(
        &self,
        to_df: &DayFile,
        _from_df: &DayFile,
        _items: &[Item],
        _dry_run: bool,
    ) -> Result<(), std::io::Error> {
        Self::to_json(to_df)?;
        Ok(())
    }

    fn render_review(
        &self,
        start: &NaiveDate,
        end: &NaiveDate,
        days: u64,
        dayfiles: &[DayFile],
    ) -> Result<(), std::io::Error> {
        todo!()
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
