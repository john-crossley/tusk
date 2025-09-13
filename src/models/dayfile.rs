use chrono::{NaiveDate};

use crate::models::item::Item;

pub struct DayFile {
    pub date: NaiveDate,
    pub items: Vec<Item>,
}