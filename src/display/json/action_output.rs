use serde::Serialize;

use crate::{
    display::json::{
        dayfile_output::{DayOutput, ItemOutput},
        show_output::Reference,
    },
    utils::render::ActionKind,
};

#[derive(Debug, Serialize)]
pub struct ActionOutput {
    pub day: DayOutput,
    pub reference: Reference,
    pub result: ActionResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<ItemOutput>,
}

impl ActionOutput {
    pub fn new(
        day: DayOutput,
        reference: Reference,
        result: ActionResult,
        item: Option<ItemOutput>,
    ) -> Self {
        Self {
            day,
            reference,
            result,
            item,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionResult {
    Updated,
    Noop,
    Removed,
    NotFound,
}

impl ActionKind {
    pub fn as_result(&self) -> ActionResult {
        match self {
            ActionKind::Done | ActionKind::Undone => ActionResult::Updated, // or Noop based on logic outside
            ActionKind::Removed => ActionResult::Removed,
        }
    }
}
