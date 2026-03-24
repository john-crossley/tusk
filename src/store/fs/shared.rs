use std::{
    fs::{File, create_dir_all},
    io::{self, BufReader, BufWriter, Error, ErrorKind, Write},
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use serde::{Serialize, de::DeserializeOwned};

pub(super) fn normalise_or_default(vault: Option<&str>) -> String {
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

pub(super) fn tusk_data_root() -> io::Result<PathBuf> {
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

pub(super) fn read_json<T>(path: &Path) -> Result<T, std::io::Error>
where
    T: DeserializeOwned,
{
    if !path.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("failed to load JSON at {}", path.display()),
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

pub(super) fn save_to_json<T>(path: &Path, data: &T) -> Result<(), std::io::Error>
where
    T: Serialize,
{
    if let Some(parent_path) = path.parent()
        && !parent_path.exists()
    {
        create_dir_all(parent_path)?;
    }

    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, data)?;
    writer.write_all(b"\n")?;
    writer.flush()?;

    Ok(())
}