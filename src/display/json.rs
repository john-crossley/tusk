use std::io::{self, Write};

use chrono::NaiveDate;
use serde::Serialize;

use crate::{
    display::{
        json::{dayfile_output::{DayFileOutput, Response}, show_output::ShowOutput},
        renderer::Renderer,
    },
    models::{dayfile::DayFile, item::Item},
};

mod dayfile_output;
mod show_output;
mod migrate_output;

pub struct JsonRenderer;

impl Renderer for JsonRenderer {
    fn render_day(&self, df: &DayFile) -> Result<(), std::io::Error> {
        let payload = DayFileOutput::from(df);
        let response = Response::<&DayFileOutput>::new("ls", &payload);
        Self::to_json(&response)
    }

    fn render_summary(&self, date: NaiveDate, index: usize, item: &Item) -> Result<(), std::io::Error> {
        let payload = ShowOutput::new(index, date, item);
        let response = Response::<&ShowOutput>::new("show", &payload);
        Self::to_json(&response)
    }

    fn render_migrate(
        &self,
        to_df: &DayFile,
        from_df: &DayFile,
        items: &[Item],
        dry_run: bool,
    ) -> Result<(), std::io::Error> {
        todo!()
        // let output = json!({
        //     "from_dayfile": from_df,
        //     "to_dayfile": to_df,
        //     "items_migrated": {
        //         "dry_run": dry_run,
        //         "items": items
        //     }
        // });

        // Self::to_json(&output)
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
