use std::io::Error;

use chrono::NaiveDate;

use crate::{
    models::{dayfile::DayFile, item::Item},
    utils::{render::ActionKind, tusk_error::TuskError}, view::agenda::Agenda,
};

pub trait Renderer {
    fn render_agenda(&self, agenda: &Agenda) -> std::io::Result<()>;

    fn render_day(&self, df: &DayFile) -> std::io::Result<()>;

    fn render_summary(&self, date: Option<NaiveDate>, index: usize, item: &Item) -> std::io::Result<()>;

    fn render_migrate(
        &self,
        to_date: NaiveDate,
        from_df_original: &DayFile,
        moved_items: &[Item],
        dry_run: bool,
    ) -> Result<(), Error>;

    fn render_review(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        days: u64,
        dayfiles: &[DayFile],
    ) -> std::io::Result<()>;

    fn render_action(
        &self,
        index: usize,
        date: NaiveDate,
        action: ActionKind,
        item: Option<&Item>,
    ) -> std::io::Result<()>;

    fn render_error(&self, command: &'static str, e: &TuskError) -> std::io::Result<()>;
}
