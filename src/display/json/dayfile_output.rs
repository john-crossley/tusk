use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;

use crate::models::{
    dayfile::DayFile,
    item::{Item, ItemPriority, ItemStatus},
};

#[derive(Serialize, Debug)]
pub struct DayOutput {
    pub date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct IndexItemOutput {
    pub index: usize,
    pub item: ItemOutput,
}

impl IndexItemOutput {
    pub fn new(index: usize, item: ItemOutput) -> Self {
        Self { index, item }
    }
}

#[derive(Serialize, Debug)]
pub struct ItemOutput {
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
pub struct DayStatsOutput {
    pub total: usize,
    pub open: usize,
    pub done: usize,
}

#[derive(Serialize, Debug)]
pub struct DayFileOutput {
    day: DayOutput,
    stats: DayStatsOutput,
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
            day: DayOutput {
                date: value.date,
                path: None,
            },
            stats: DayStatsOutput { total, open, done },
            items: value.items.iter().map(ItemOutput::from).collect(),
        }
    }
}