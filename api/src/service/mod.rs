pub mod auth;
pub mod checkin;
pub mod habit;
pub mod plan;

use std::sync::Arc;

use uuid::Uuid;

use crate::config::AuthConfig;
use crate::domain::{Period, Tracking};
use crate::error::{AppError, AppResult};
use crate::repo::pool::{DbConn, DbPool};

/// Everything that does not need to know who is asking: signing in, mostly.
#[derive(Clone)]
pub struct Service {
    pool: DbPool,
    auth: Arc<AuthConfig>,
    http: reqwest::Client,
}

/// The service acting for one signed-in user. Only the auth extractor builds one,
/// so a handler holding it has already proven who is asking - and a handler that
/// forgets to ask does not compile, because the data methods live only here.
pub struct UserService {
    service: Service,
    user_id: Uuid,
}

impl Service {
    pub fn new(pool: DbPool, auth: AuthConfig) -> Self {
        Self {
            pool,
            auth: Arc::new(auth),
            http: reqwest::Client::new(),
        }
    }

    fn for_user(&self, user_id: Uuid) -> UserService {
        UserService {
            service: self.clone(),
            user_id,
        }
    }

    async fn conn(&self) -> AppResult<DbConn> {
        Ok(self.pool.get().await?)
    }
}

impl UserService {
    async fn conn(&self) -> AppResult<DbConn> {
        self.service.conn().await
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
