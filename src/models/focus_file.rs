use serde::{Deserialize, Serialize};

use crate::models::item::Item;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FocusFile {
    pub items: Vec<Item>,
}

impl FocusFile {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
}
