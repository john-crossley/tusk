use std::{
    fs::{create_dir_all, File},
    io::{self, BufReader, BufWriter, Error, ErrorKind, Write},
    path::{Path, PathBuf},
};

use chrono::{Datelike, NaiveDate};
use directories::ProjectDirs;

use crate::models::dayfile::DayFile;

pub fn tusk_data_root() -> PathBuf {
    match ProjectDirs::from("io", "jonnothebonno", "tusk") {
        Some(project_dir) => project_dir.data_dir().to_owned(),
        None => panic!("Unable to form config path"),
    }
}

pub fn resolve_day_file_path(date: &NaiveDate, base_dir: Option<&Path>, verbose: bool) -> PathBuf {
    let year = NaiveDate::year(date);
    let month = NaiveDate::month(date);

    let mut working_dir = base_dir
        .map(|p| p.to_path_buf())
        .unwrap_or(tusk_data_root());

    working_dir.push(format!("{:04}", year));
    working_dir.push(format!("{:02}", month));
    working_dir.push(format!("{}.json", date));

    if verbose {
        eprintln!("[🐘] File: {}", working_dir.display())
    }

    working_dir
}

pub fn load_or_create_dayfile(path: &Path, date: NaiveDate) -> io::Result<DayFile> {
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

    if let Some(parent_path) = path.parent() {
        create_dir_all(parent_path)?;
    }

    let dayfile = DayFile {
        date,
        items: vec![],
    };

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &dayfile)?;
    writer.write_all(b"\n")?;
    writer.flush()?;

    Ok(dayfile)
}
