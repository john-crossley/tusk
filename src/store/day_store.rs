use chrono::NaiveDate;

use crate::models::dayfile::DayFile;

// pub enum StoreType {
//     Day {
//         date: NaiveDate,
//         vault: Option<String>,
//     },
//     Focus {
//         vault: Option<String>,
//     },
// }

pub trait DayStore {
    fn load(&self, date: NaiveDate) -> Result<DayFile, std::io::Error>;
    fn save(&self, df: &DayFile) -> Result<(), std::io::Error>;
}
