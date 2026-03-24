use std::{
    io::{self, Error},
    path::PathBuf,
};

use crate::{
    models::focus_file::FocusFile,
    store::{
        focus_store::FocusStore,
        fs::shared::{normalise_or_default, read_json, save_to_json, tusk_data_root},
    },
};

pub struct FsFocusStore {
    pub base_dir: PathBuf,
    pub vault: Option<String>,
}

impl FsFocusStore {
    pub fn new(base_dir: Option<PathBuf>, vault: Option<&str>) -> io::Result<Self> {
        Ok(Self {
            base_dir: base_dir.unwrap_or(tusk_data_root()?),
            vault: vault.map(|v| v.to_string()),
        })
    }

    fn focusfile_path(&self) -> PathBuf {
        self.base_dir
            .join("vaults")
            .join(normalise_or_default(self.vault.as_deref()))
            .join("focus.json")
    }
}

impl FocusStore for FsFocusStore {
    fn load(&self) -> Result<FocusFile, Error> {
        let path = self.focusfile_path();
        read_json(&path)
    }

    fn save(&self, ff: &FocusFile) -> Result<(), Error> {
        let path = self.focusfile_path();
        save_to_json(&path, ff)
    }
}
