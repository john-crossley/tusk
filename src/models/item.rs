use core::fmt;

use chrono::{DateTime, NaiveDate, Utc};
use clap::ValueEnum;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ItemPriority {
    High,
    Medium,
    Low,
}

impl fmt::Display for ItemPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ItemPriority::High => "high",
            ItemPriority::Medium => "medium",
            ItemPriority::Low => "low",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemStatus {
    Open,
    Done,
}

impl fmt::Display for ItemStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ItemStatus::Open => "open",
            ItemStatus::Done => "done",
        };
        f.write_str(s)
    }
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

    pub fn status(&self) -> ItemStatus {
        if self.done_at.is_some() {
            ItemStatus::Done
        } else {
            ItemStatus::Open
        }
    }
}
