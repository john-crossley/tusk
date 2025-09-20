use std::{env, io, io::Write, process::Command};

use tempfile::NamedTempFile;

pub fn edit_in_editor(initial: &str) -> io::Result<String> {
    let mut tmp = NamedTempFile::new()?;
    writeln!(tmp, "{}", initial)?;

    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| String::from("nano"));

    Command::new(editor).arg(tmp.path()).status()?;

    let contents = std::fs::read_to_string(tmp.path())?;
    Ok(contents)
}
