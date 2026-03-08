use std::{
    fs::{File, create_dir_all},
    io::{self, BufReader, BufWriter, Error, ErrorKind, Write},
    path::PathBuf,
};

use chrono::{Datelike, NaiveDate};
use directories::ProjectDirs;

use crate::{models::dayfile::DayFile, store::day_store::DayStore};

pub struct FsDayStore {
    pub base_dir: PathBuf,
    pub vault: Option<String>,
}

impl FsDayStore {
    pub fn new(base_dir: Option<PathBuf>, vault: Option<&str>) -> io::Result<Self> {
        Ok(Self {
            base_dir: base_dir.unwrap_or(Self::tusk_data_root()?),
            vault: vault.map(|v| v.to_string()),
        })
    }

    pub fn dayfile_path(&self, date: &NaiveDate) -> PathBuf {
        let year = date.year();
        let month = date.month();

        self.base_dir
            .join("vaults")
            .join(Self::normalise_or_default(self.vault.as_deref()))
            .join(format!("{:04}", year))
            .join(format!("{:02}", month))
            .join(format!("{}.json", date))
    }

    fn tusk_data_root() -> io::Result<PathBuf> {
        let root = match ProjectDirs::from("io", "jonnothebonno", "tusk") {
            Some(proj_dir) => proj_dir.data_dir().to_owned(),
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "could not determine platform data directory.",
                ));
            }
        };

        Ok(root)
    }

    fn normalise_or_default(vault: Option<&str>) -> String {
        match vault {
            None => "default".to_string(),
            Some(s) => {
                let filtered = s
                    .trim()
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
                    .collect::<String>()
                    .to_ascii_lowercase();

                if filtered.is_empty() {
                    "default".to_string()
                } else {
                    filtered
                }
            }
        }
    }
}

impl DayStore for FsDayStore {
    fn load(&self, date: NaiveDate) -> Result<DayFile, std::io::Error> {
        let path = self.dayfile_path(&date);

        if !path.exists() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("failed to load dayfile at {}", path.display()),
            ));
        }

        let file = File::open(&path)?;
        let reader = BufReader::new(file);

        serde_json::from_reader(reader).map_err(|e| {
            Error::new(
                ErrorKind::InvalidData,
                format!("failed to parse JSON in {}: {}", path.display(), e),
            )
        })
    }

    fn save(&self, df: &DayFile) -> Result<(), std::io::Error> {
        let path = self.dayfile_path(&df.date);

        // TODO: Should we move this into a helper?
        if let Some(parent_path) = path.parent() && !parent_path.exists() {
            create_dir_all(parent_path)?;
        }

        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &df)?;
        writer.write_all(b"\n")?;
        writer.flush()?;

        Ok(())
    }
}
