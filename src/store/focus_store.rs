use std::io::Error;

use crate::models::focus_file::FocusFile;

pub trait FocusStore {
    fn load(&self) -> Result<FocusFile, Error>;
    fn save(&self, ff: &FocusFile) -> Result<(), Error>;
}