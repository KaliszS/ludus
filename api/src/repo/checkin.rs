use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::pool::DbConn;
use super::schema::{habit_checkins, habits};
use crate::domain::Checkin;
use crate::error::{AppError, AppResult};

#[derive(Queryable, Selectable)]
#[diesel(table_name = habit_checkins, check_for_backend(diesel::pg::Pg))]
pub struct CheckinRow {
    pub day: NaiveDate,
    pub times: i32,
}

impl From<CheckinRow> for Checkin {
    fn from(row: CheckinRow) -> Self {
        Self {
            day: row.day,
            times: row.times,
        }
    }
}

pub async fn list(
    conn: &mut DbConn,
    habit_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> AppResult<Vec<Checkin>> {
    let rows = habit_checkins::table
        .filter(habit_checkins::habit_id.eq(habit_id))
        .filter(habit_checkins::day.between(from, to))
        .order(habit_checkins::day.asc())
        .select(CheckinRow::as_select())
        .load(conn)
        .await?;
    Ok(rows.into_iter().map(Checkin::from).collect())
}

pub async fn set(
    conn: &mut DbConn,
    habit_id: Uuid,
    day: NaiveDate,
    times: i32,
) -> AppResult<Checkin> {
    let now = Utc::now();
    let row = diesel::insert_into(habit_checkins::table)
        .values((
            habit_checkins::habit_id.eq(habit_id),
            habit_checkins::day.eq(day),
            habit_checkins::times.eq(times),
        ))
        .on_conflict((habit_checkins::habit_id, habit_checkins::day))
        .do_update()
        .set((
            habit_checkins::times.eq(times),
            habit_checkins::updated_at.eq(now),
        ))
        .returning(CheckinRow::as_select())
        .get_result(conn)
        .await?;
    Ok(row.into())
}

pub async fn unset(conn: &mut DbConn, habit_id: Uuid, day: NaiveDate) -> AppResult<()> {
    let affected = diesel::delete(
        habit_checkins::table
            .filter(habit_checkins::habit_id.eq(habit_id))
            .filter(habit_checkins::day.eq(day)),
    )
    .execute(conn)
    .await?;
    if affected == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// Every check-in the user owns in one round trip, so a board does not fan out per habit.
pub async fn range_for_user(
    conn: &mut DbConn,
    user_id: Uuid,
    from: NaiveDate,
    to: NaiveDate,
) -> AppResult<Vec<(Uuid, NaiveDate, i32)>> {
    Ok(habit_checkins::table
        .inner_join(habits::table)
        .filter(habits::user_id.eq(user_id))
        .filter(habit_checkins::day.between(from, to))
        .order((habit_checkins::habit_id, habit_checkins::day))
        .select((
            habit_checkins::habit_id,
            habit_checkins::day,
            habit_checkins::times,
        ))
        .load(conn)
        .await?)
}
