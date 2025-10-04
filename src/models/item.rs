use chrono::{DateTime, NaiveDate, Utc};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

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
    pub migrated_from: Option<NaiveDate>,
}

impl Item {
    pub fn new(
        text: String,
        priority: ItemPriority,
        tags: Vec<String>,
        notes: Option<String>,
    ) -> Self {
        Item {
            id: nanoid!(6),
            text: text,
            created_at: Utc::now(),
            done_at: None,
            priority,
            tags,
            due: None,
            notes,
            migrated_from: None,
        }
    }
}
