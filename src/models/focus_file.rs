use serde::{Deserialize, Serialize};

use crate::models::item::Item;


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FocusFile {
    pub items: Vec<Item>,
}