use serde::Serialize;

use crate::{
    display::json::dayfile_output::{DayFileOutput, DayStatsOutput, ItemOutput},
    models::focus_file::FocusFile,
    view::agenda::Agenda,
};

#[derive(Serialize, Debug)]
pub struct AgendaOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    focus: Option<FocusFileOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    day: Option<DayFileOutput>,
}

impl From<&Agenda> for AgendaOutput {
    fn from(value: &Agenda) -> Self {
        Self {
            focus: if let Some(ff) = &value.focusfile {
                Some(FocusFileOutput::from(ff))
            } else {
                None
            },
            day: if let Some(df) = &value.dayfile {
                Some(DayFileOutput::from(df))
            } else {
                None
            },
        }
    }
}

#[derive(Serialize, Debug)]
struct FocusFileOutput {
    stats: DayStatsOutput,
    items: Vec<ItemOutput>,
}

impl From<&FocusFile> for FocusFileOutput {
    fn from(value: &FocusFile) -> Self {
        let total = value.items.len();
        let done = value.items.iter().filter(|i| i.done_at.is_some()).count();
        let open = total - done;

        Self {
            stats: DayStatsOutput { total, open, done },
            items: value.items.iter().map(ItemOutput::from).collect(),
        }
    }
}
