use std::{io, path::PathBuf};

use directories::ProjectDirs;

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
