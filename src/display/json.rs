use std::io::{self, Write};

use chrono::NaiveDate;
use serde::Serialize;

use crate::{
    display::{
        json::{
            action_output::ActionOutput,
            dayfile_output::{DayFileOutput, DayOutput, Response},
            migrate_output::MigrateOutput,
            review_output::ReviewOutput,
            show_output::{Reference, ReferenceKind, ShowOutput},
        },
        renderer::Renderer,
    },
    models::{dayfile::DayFile, item::Item},
    utils::{helpers::item_count_meta, render::ActionKind},
};

mod action_output;
mod dayfile_output;
mod migrate_output;
mod review_output;
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
        start: NaiveDate,
        end: NaiveDate,
        days: u64,
        dayfiles: &[DayFile],
    ) -> Result<(), std::io::Error> {
        let count = item_count_meta(dayfiles);
        let payload = ReviewOutput::new(days, start, end, true, dayfiles, count);
        let response = Response::new("review", &payload);
        Self::to_json(&response)
    }

    fn render_action(
        &self,
        index: usize,
        date: NaiveDate,
        action: ActionKind,
        item: Option<&Item>,
    ) -> Result<(), std::io::Error> {
        let payload = ActionOutput::new(
            DayOutput { date, path: None },
            Reference {
                kind: ReferenceKind::Index,
                value: index,
            },
            action.as_result(),
            item.map(Into::into),
        );

        let response = Response::new(action.as_command(), payload);
        Self::to_json(&response)
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
