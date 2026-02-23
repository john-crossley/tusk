use chrono::NaiveDate;
use serde::Serialize;

use crate::{
    display::json::dayfile_output::{DayOutput, DayStatsOutput, IndexItemOutput},
    models::dayfile::DayFile,
    utils::helpers::ItemCountResult,
};

#[derive(Serialize, Debug)]
pub struct ReviewOutput {
    pub range: RangeOutput,
    pub stats: RangeStats,
    pub days: Vec<ReviewDayOutput>,
}

impl ReviewOutput {
    pub fn new(
        days: u64,
        from: NaiveDate,
        to: NaiveDate,
        exclude_today: bool,
        dayfiles: &[DayFile],
        count_meta: ItemCountResult,
    ) -> Self {
        Self {
            range: RangeOutput {
                days,
                from,
                to,
                exclude_today: exclude_today,
            },
            stats: RangeStats {
                total: count_meta.total,
                open: count_meta.open,
                done: count_meta.complete,
                active_days: dayfiles.len(),
            },
            days: Self::make_days(dayfiles),
        }
    }

    fn make_days(dayfiles: &[DayFile]) -> Vec<ReviewDayOutput> {
        dayfiles
            .iter()
            .map(|df| {
                let total = df.items.len();
                let done = df.items.iter().filter(|i| i.done_at.is_some()).count();
                let open = total - done;

                ReviewDayOutput {
                    day: DayOutput {
                        date: df.date,
                        path: None,
                    },
                    stats: DayStatsOutput { total, open, done },
                    items: df
                        .items
                        .iter()
                        .enumerate()
                        .map(|(i, item)| IndexItemOutput::new(i + 1, item.into()))
                        .collect(),
                }
            })
            .collect()
    }
}

#[derive(Serialize, Debug)]
pub struct RangeOutput {
    pub days: u64,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub exclude_today: bool,
}

#[derive(Serialize, Debug)]
pub struct RangeStats {
    pub total: usize,
    pub open: usize,
    pub done: usize,
    pub active_days: usize,
}

#[derive(Serialize, Debug)]
pub struct ReviewDayOutput {
    pub day: DayOutput,
    pub stats: DayStatsOutput,
    pub items: Vec<IndexItemOutput>,
}
