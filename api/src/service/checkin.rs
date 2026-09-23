use chrono::NaiveDate;
use uuid::Uuid;

use super::Service;
use crate::domain::{Checkin, Tracking};
use crate::error::{AppError, AppResult};
use crate::repo::{checkin, habit};

impl Service {
    pub async fn checkins(
        &self,
        habit_id: Uuid,
        from: NaiveDate,
        to: NaiveDate,
    ) -> AppResult<Vec<Checkin>> {
        if from > to {
            return Err(AppError::invalid("from", "must not be after to"));
        }
        let mut conn = self.conn().await?;
        habit::get(&mut conn, self.current_user(), habit_id).await?;
        checkin::list(&mut conn, habit_id, from, to).await
    }

    pub async fn all_checkins(
        &self,
        from: NaiveDate,
        to: NaiveDate,
    ) -> AppResult<Vec<(Uuid, NaiveDate, i32)>> {
        if from > to {
            return Err(AppError::invalid("from", "must not be after to"));
        }
        let mut conn = self.conn().await?;
        checkin::range_for_user(&mut conn, self.current_user(), from, to).await
    }

    /// Idempotent per day. A binary habit ignores any amount and stores 1.
    pub async fn set_checkin(
        &self,
        habit_id: Uuid,
        day: NaiveDate,
        times: Option<i32>,
    ) -> AppResult<Checkin> {
        let mut conn = self.conn().await?;
        let habit = habit::get(&mut conn, self.current_user(), habit_id).await?;

        let times = match (habit.tracking, times) {
            (Tracking::Binary, _) => 1,
            (Tracking::Quantity, Some(value)) => value,
            (Tracking::Quantity, None) => 1,
        };
        if times < 1 {
            return Err(AppError::invalid("times", "must be at least 1"));
        }

        checkin::set(&mut conn, habit_id, day, times).await
    }

    pub async fn unset_checkin(&self, habit_id: Uuid, day: NaiveDate) -> AppResult<()> {
        let mut conn = self.conn().await?;
        habit::get(&mut conn, self.current_user(), habit_id).await?;
        checkin::unset(&mut conn, habit_id, day).await
    }
}
