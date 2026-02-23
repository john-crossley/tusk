use std::io::{self, Write};

use chrono::NaiveDate;
use serde::Serialize;

use crate::{
    display::{
        json::{
            dayfile_output::{DayFileOutput, Response},
            migrate_output::MigrateOutput,
            show_output::ShowOutput,
        },
        renderer::Renderer,
    },
    models::{dayfile::DayFile, item::Item},
};

mod dayfile_output;
mod migrate_output;
mod show_output;

pub struct JsonRenderer;

impl Renderer for JsonRenderer {
    fn render_day(&self, df: &DayFile) -> Result<(), std::io::Error> {
        let payload = DayFileOutput::from(df);
        let response = Response::<&DayFileOutput>::new("ls", &payload);
        Self::to_json(&response)
    }

    fn render_summary(
        &self,
        date: NaiveDate,
        index: usize,
        item: &Item,
    ) -> Result<(), std::io::Error> {
        let payload = ShowOutput::new(index, date, item);
        let response = Response::new("show", &payload);
        Self::to_json(&response)
    }

    fn render_migrate(
        &self,
        to_date: NaiveDate,
        from_df_original: &DayFile,
        moved_items: &[Item],
        dry_run: bool,
    ) -> Result<(), std::io::Error> {
        let payload = MigrateOutput::new(dry_run, from_df_original, to_date, moved_items);
        let response = Response::new("migrate", &payload);
        Self::to_json(&response)
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
