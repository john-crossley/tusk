use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::models::item::Item;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DayFile {
    pub date: NaiveDate,
    pub items: Vec<Item>,
}

impl DayFile {
    pub fn migratable_items(&self) -> Vec<Item> {
        self.items
            .iter()
            .filter(|i| i.done_at.is_none())
            .cloned()
            .collect()
    }
}
