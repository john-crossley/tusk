use std::{fs::create_dir_all, path::{Path, PathBuf}};

use chrono::{Datelike, NaiveDate};
use directories::ProjectDirs;


pub fn tusk_data_root() -> PathBuf {
    match ProjectDirs::from("io", "jonnothebonno", "tusk") {
        Some(project_dir) => project_dir.data_dir().to_owned(),
        None => panic!("Unable to form config path")
    }
}

pub fn resolve_day_file(date: &NaiveDate, base_dir: Option<&Path>) -> PathBuf {
    let year = NaiveDate::year(date);
    let month = NaiveDate::month(date);

    let mut working_dir = base_dir
        .map(|p| p.to_path_buf())
        .unwrap_or(tusk_data_root());

    working_dir.push(format!("{:04}", year));
    working_dir.push(format!("{:02}", month));

    create_dir_all(&working_dir).expect("Failed to create Tusk directory");

    working_dir.push(format!("{}.json", date));

    println!("Tusk dir: {}", working_dir.display());

    working_dir
}