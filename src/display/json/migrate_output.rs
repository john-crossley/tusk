use chrono::NaiveDate;
use serde::Serialize;

use crate::{
    display::json::dayfile_output::{DayOutput, ItemOutput},
    models::{dayfile::DayFile, item::Item},
};

#[derive(Serialize, Debug)]
pub struct MigrateStatsOutput {
    pub considered: usize,
    pub migrated: usize,
    pub skipped_done: usize,
}

#[derive(Serialize, Debug)]
pub struct MigrateOutput {
    dry_run: bool,
    from: DayOutput,
    to: DayOutput,
    stats: MigrateStatsOutput,
    items: Vec<ItemOutput>,
}

impl MigrateOutput {
    pub fn new(dry_run: bool, from_df: &DayFile, to_date: NaiveDate, items: &[Item]) -> Self {

        let considered = from_df.items.len();
        let migrated = items.len();

        Self {
            dry_run,
            from: DayOutput {
                date: from_df.date,
                path: None,
            },
            to: DayOutput {
                date: to_date,
                path: None,
            },
            stats: MigrateStatsOutput {
                considered,
                migrated,
                skipped_done: from_df.items.iter().filter(|i| i.done_at.is_some()).count(),
            },
            items: items.iter().map(Into::into).collect(),
        }
    }
}
