use std::{
    fs::{File, create_dir_all},
    io::{self, BufReader, BufWriter, Error, ErrorKind, Write},
    path::{Path, PathBuf},
};

use chrono::{Datelike, NaiveDate};
use directories::ProjectDirs;

use crate::models::dayfile::DayFile;

pub fn tusk_data_root(vault: Option<&str>) -> io::Result<PathBuf> {
    let root = match ProjectDirs::from("io", "jonnothebonno", "tusk") {
        Some(project_dir) => project_dir.data_dir().to_owned(),
        None => {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "could not determine platform data directory",
            ));
        }
    };

    Ok(root.join("vaults").join(normalise_or_default(vault)))
}

pub fn resolve_day_file_path(
    date: &NaiveDate,
    base_dir: Option<&Path>,
    verbose: bool,
    vault: Option<&str>,
) -> io::Result<PathBuf> {
    let year = NaiveDate::year(date);
    let month = NaiveDate::month(date);

    let mut working_dir = base_dir
        .map(|p| p.to_path_buf())
        .unwrap_or(tusk_data_root(vault)?);

    working_dir.push(format!("{:04}", year));
    working_dir.push(format!("{:02}", month));
    working_dir.push(format!("{}.json", date));

    if verbose {
        eprintln!("[🦣] File: {}", working_dir.display())
    }

    Ok(working_dir)
}

pub fn save_dayfile(path: &Path, dayfile: &DayFile) -> Result<(), Error> {
    if let Some(parent_path) = path.parent() {
        create_dir_all(parent_path)?;
    }

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &dayfile)?;
    writer.write_all(b"\n")?;
    writer.flush()?;

    Ok(())
}

pub fn load_or_create_dayfile(path: &Path, date: NaiveDate) -> Result<DayFile, Error> {
    match File::open(path) {
        Ok(file) => {
            let buffer = BufReader::new(file);

            match serde_json::from_reader(buffer) {
                Ok(dayfile) => Ok(dayfile),
                Err(e) => Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("read {} failed: {}", path.display(), e),
                )),
            }
        }
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => create_new_dayfile(path, date),
            _ => Err(Error::new(
                ErrorKind::Other,
                format!("open {} failed: {}", path.display(), e),
            )),
        },
    }
}

fn create_new_dayfile(path: &Path, date: NaiveDate) -> io::Result<DayFile> {
    let dayfile = DayFile {
        date,
        items: vec![],
    };
    save_dayfile(path, &dayfile)?;
    Ok(dayfile)
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
