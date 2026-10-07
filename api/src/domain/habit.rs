use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tracking {
    Binary,
    Quantity,
}

impl Tracking {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "binary" => Some(Self::Binary),
            "quantity" => Some(Self::Quantity),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Binary => "binary",
            Self::Quantity => "quantity",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Habit {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub unit: Option<String>,
    pub tracking: Tracking,
    pub weekdays: Vec<i16>,
    pub category_id: Option<Uuid>,
    pub position: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
}

/// A top-level category has no parent; a subcategory's parent is always top-level.
#[derive(Debug, Clone)]
pub struct HabitCategory {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub position: f64,
}
