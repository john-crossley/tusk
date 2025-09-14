use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum ItemPriority {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub text: String,
    pub created_at: DateTime<Utc>,
    pub done_at: Option<DateTime<Utc>>,
    pub priority: ItemPriority,
    pub tags: Vec<String>,
    pub due: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub index: u32,
}

impl Item {
    pub fn new(text: String, next_idx: u32) -> Self {
        Item {
            id: Uuid::new_v4().to_string(),
            text: text,
            created_at: Utc::now(),
            done_at: None,
            priority: ItemPriority::Low,
            tags: Vec::new(),
            due: None,
            notes: None,
            index: next_idx,
        }
    }
}
