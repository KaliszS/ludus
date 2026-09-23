pub mod checkin;
pub mod habit;
pub mod plan;

use uuid::Uuid;

use crate::domain::{Period, Tracking};
use crate::error::{AppError, AppResult};
use crate::repo::pool::{DbConn, DbPool};

/// Stands in for the authenticated user until auth lands; seeded by dev_seed.sql.
pub const DEV_USER_ID: Uuid = Uuid::from_u128(1);

#[derive(Clone)]
pub struct Service {
    pool: DbPool,
}

impl Service {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn current_user(&self) -> Uuid {
        DEV_USER_ID
    }

    async fn conn(&self) -> AppResult<DbConn> {
        Ok(self.pool.get().await?)
    }
}

pub fn require_name(field: &str, value: &str) -> AppResult<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(AppError::invalid(field, "must not be empty"));
    }
    Ok(trimmed.to_owned())
}

pub fn parse_tracking(value: &str) -> AppResult<Tracking> {
    Tracking::parse(value)
        .ok_or_else(|| AppError::invalid("tracking", r#"must be "binary" or "quantity""#))
}

pub fn parse_period(value: &str) -> AppResult<Period> {
    Period::parse(value).ok_or_else(|| {
        AppError::invalid(
            "period",
            r#"must be one of "day", "week", "month", "quarter", "year""#,
        )
    })
}

pub fn check_weekdays(days: &[i16]) -> AppResult<()> {
    if days.iter().any(|d| !(1..=7).contains(d)) {
        return Err(AppError::invalid("weekdays", "must be ISO numbers 1 to 7"));
    }
    Ok(())
}
