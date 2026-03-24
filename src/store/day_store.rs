use std::io::Error;
use chrono::NaiveDate;

use crate::models::dayfile::DayFile;

pub trait DayStore {
    fn load(&self, date: NaiveDate) -> Result<DayFile, Error>;
    fn save(&self, df: &DayFile) -> Result<(), Error>;
}
