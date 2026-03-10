use crate::models::{dayfile::DayFile, focus_file::FocusFile};

pub struct Agenda {
    pub dayfile: DayFile,
    pub focusfile: Option<FocusFile>,
}
