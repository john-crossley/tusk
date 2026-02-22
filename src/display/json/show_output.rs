use chrono::NaiveDate;
use serde::Serialize;

use crate::{
    display::json::dayfile_output::{DayOutput, ItemOutput},
    models::item::Item,
};

#[derive(Debug, Serialize)]
pub struct Reference {
    kind: ReferenceKind,
    value: usize,
}

#[derive(Debug, Serialize)]
pub struct ShowOutput {
    day: DayOutput,
    reference: Reference,
    item: ItemOutput,
}

impl ShowOutput {
    pub fn new(index: usize, date: NaiveDate, item: &Item) -> Self {
        Self {
            day: DayOutput { date, path: None },
            reference: Reference {
                kind: ReferenceKind::Index,
                value: index,
            },
            item: ItemOutput::from(item),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReferenceKind {
    Id,
    Index,
}
