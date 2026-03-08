use crate::models::item::Item;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DayFile {
    pub date: NaiveDate,
    pub items: Vec<Item>,
}

impl DayFile {

    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            items: Vec::new()
        }
    }

    pub fn migratable_items(&self) -> Vec<Item> {
        self.items
            .iter()
            .filter(|i| i.done_at.is_none())
            .cloned()
            .collect()
    }

    pub fn filtered_by_tags(&self, tags: &[String]) -> DayFile {
        let items: Vec<_> = self
            .items
            .iter()
            .filter(|item| {
                tags.iter()
                    .all(|tag| item.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
            })
            .cloned()
            .collect();

        DayFile {
            date: self.date,
            items,
        }
    }
}
