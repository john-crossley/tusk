use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::models::item::Item;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DayFile {
    pub date: NaiveDate,
    pub items: Vec<Item>,
}
