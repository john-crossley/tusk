use chrono::{NaiveDate};

use crate::models::item::Item;

#[derive(Debug)]
pub struct DayFile {
    pub date: NaiveDate,
    pub items: Vec<Item>,
}