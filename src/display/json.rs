use std::io::{self, Write};

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

use crate::{
    display::renderer::Renderer,
    models::{
        dayfile::DayFile,
        item::{Item, ItemPriority, ItemStatus},
    },
};

pub struct JsonRenderer;

impl Renderer for JsonRenderer {
    fn render_day(&self, df: &DayFile) -> Result<(), std::io::Error> {
        Self::to_json(&DayFileOutput::from(df))
    }

    fn render_summary(&self, _index: Option<usize>, item: &Item) -> Result<(), std::io::Error> {
        Self::to_json(item)
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

// Models
const SCHEMA_VERSION: u8 = 1;

#[derive(Serialize, Debug)]
struct DayOutput {
    date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
}

#[derive(Serialize, Debug)]
struct ItemOutput {
    id: String,
    text: String,
    created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    done_at: Option<DateTime<Utc>>,
    priority: ItemPriority,
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    due: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    migrated_from_date: Option<NaiveDate>,
    status: ItemStatus,
}

#[derive(Serialize, Debug)]
struct SummaryOutput {
    total: usize,
    open: usize,
    done: usize,
}

#[derive(Serialize, Debug)]
struct DayFileOutput {
    schema_version: u8,
    // command: &'static str,
    day: DayOutput,
    summary: SummaryOutput,
    items: Vec<ItemOutput>,
}

impl From<&Item> for ItemOutput {
    fn from(value: &Item) -> Self {
        Self {
            id: value.id.clone(),
            text: value.text.clone(),
            created_at: value.created_at,
            done_at: value.done_at,
            priority: value.priority,
            tags: value.tags.clone(),
            due: value.due,
            notes: value.notes.clone(),
            migrated_from_date: value.migrated_from,
            status: value.status(),
        }
    }
}

impl From<&DayFile> for DayFileOutput {
    fn from(value: &DayFile) -> Self {
        let total = value.items.len();
        let done = value.items.iter().filter(|i| i.done_at.is_some()).count();
        let open = total - done;

        Self {
            schema_version: SCHEMA_VERSION,
            day: DayOutput {
                date: value.date,
                path: None,
            },
            summary: SummaryOutput { total, open, done },
            items: value.items.iter().map(ItemOutput::from).collect(),
        }
    }
}
