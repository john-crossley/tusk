use std::{io, path::PathBuf};

use chrono::{Datelike, NaiveDate};

use crate::{
    models::dayfile::DayFile,
    store::{
        day_store::DayStore,
        fs::shared::{normalise_or_default, read_json, save_to_json, tusk_data_root},
    },
};

pub struct FsDayStore {
    pub base_dir: PathBuf,
    pub vault: Option<String>,
}

impl FsDayStore {
    pub fn new(base_dir: Option<PathBuf>, vault: Option<&str>) -> io::Result<Self> {
        Ok(Self {
            base_dir: base_dir.unwrap_or(tusk_data_root()?),
            vault: vault.map(|v| v.to_string()),
        })
    }

    pub fn dayfile_path(&self, date: &NaiveDate) -> PathBuf {
        let year = date.year();
        let month = date.month();

        self.base_dir
            .join("vaults")
            .join(normalise_or_default(self.vault.as_deref()))
            .join(format!("{:04}", year))
            .join(format!("{:02}", month))
            .join(format!("{}.json", date))
    }
}

impl DayStore for FsDayStore {
    fn load(&self, date: NaiveDate) -> Result<DayFile, std::io::Error> {
        let path = self.dayfile_path(&date);
        read_json(&path)
    }

    fn save(&self, df: &DayFile) -> Result<(), std::io::Error> {
        let path = self.dayfile_path(&df.date);
        save_to_json(&path, df)
    }
}
