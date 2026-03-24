use serde::{Deserialize, Serialize};

use crate::models::{item::Item, task_stats::{HasItems, TaskStats}};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FocusFile {
    pub items: Vec<Item>,
}

impl FocusFile {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}

impl HasItems for FocusFile {
    fn items(&self) -> &[Item] {
        &self.items
    }
}

impl TaskStats for FocusFile {}