use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::pool::DbConn;
use super::schema::habits;
use crate::domain::{Habit, Tracking};
use crate::error::{AppError, AppResult};

#[derive(Queryable, Selectable)]
#[diesel(table_name = habits, check_for_backend(diesel::pg::Pg))]
pub struct HabitRow {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub weekdays: Option<Vec<Option<i16>>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub archived_at: Option<DateTime<Utc>>,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub unit: Option<String>,
    pub tracking: String,
    pub position: f64,
}

impl From<HabitRow> for Habit {
    fn from(row: HabitRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            description: row.description,
            icon: row.icon,
            color: row.color,
            unit: row.unit,
            tracking: Tracking::parse(&row.tracking).unwrap_or(Tracking::Binary),
            weekdays: row
                .weekdays
                .unwrap_or_default()
                .into_iter()
                .flatten()
                .collect(),
            position: row.position,
            created_at: row.created_at,
            updated_at: row.updated_at,
            archived_at: row.archived_at,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = habits)]
pub struct NewHabit {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub unit: Option<String>,
    pub tracking: String,
    pub weekdays: Option<Vec<Option<i16>>>,
    pub position: f64,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = habits)]
pub struct HabitChanges {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<Option<String>>,
    pub color: Option<Option<String>>,
    pub unit: Option<Option<String>>,
    pub tracking: Option<String>,
    pub weekdays: Option<Option<Vec<Option<i16>>>>,
    pub position: Option<f64>,
    pub archived_at: Option<Option<DateTime<Utc>>>,
}

impl HabitChanges {
    fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.description.is_none()
            && self.icon.is_none()
            && self.color.is_none()
            && self.unit.is_none()
            && self.tracking.is_none()
            && self.weekdays.is_none()
            && self.position.is_none()
            && self.archived_at.is_none()
    }
}

fn owned(user_id: Uuid) -> habits::BoxedQuery<'static, diesel::pg::Pg> {
    habits::table
        .filter(habits::user_id.eq(user_id))
        .into_boxed()
}

pub async fn list(
    conn: &mut DbConn,
    user_id: Uuid,
    include_archived: bool,
) -> AppResult<Vec<Habit>> {
    let mut query = owned(user_id);
    if !include_archived {
        query = query.filter(habits::archived_at.is_null());
    }
    let rows = query
        .order((habits::position.asc(), habits::created_at.asc()))
        .select(HabitRow::as_select())
        .load(conn)
        .await?;
    Ok(rows.into_iter().map(Habit::from).collect())
}

pub async fn get(conn: &mut DbConn, user_id: Uuid, id: Uuid) -> AppResult<Habit> {
    let row = owned(user_id)
        .filter(habits::id.eq(id))
        .select(HabitRow::as_select())
        .first(conn)
        .await?;
    Ok(row.into())
}

pub async fn insert(conn: &mut DbConn, habit: NewHabit) -> AppResult<Habit> {
    let row = diesel::insert_into(habits::table)
        .values(habit)
        .returning(HabitRow::as_select())
        .get_result(conn)
        .await?;
    Ok(row.into())
}

pub async fn update(
    conn: &mut DbConn,
    user_id: Uuid,
    id: Uuid,
    changes: HabitChanges,
) -> AppResult<Habit> {
    if changes.is_empty() {
        return get(conn, user_id, id).await;
    }
    let row = diesel::update(
        habits::table
            .filter(habits::id.eq(id))
            .filter(habits::user_id.eq(user_id)),
    )
    .set(changes)
    .returning(HabitRow::as_select())
    .get_result(conn)
    .await?;
    Ok(row.into())
}

pub async fn delete(conn: &mut DbConn, user_id: Uuid, id: Uuid) -> AppResult<()> {
    let affected = diesel::delete(
        habits::table
            .filter(habits::id.eq(id))
            .filter(habits::user_id.eq(user_id)),
    )
    .execute(conn)
    .await?;
    if affected == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub async fn next_position(conn: &mut DbConn, user_id: Uuid) -> AppResult<f64> {
    let highest: Option<f64> = habits::table
        .filter(habits::user_id.eq(user_id))
        .select(diesel::dsl::max(habits::position))
        .first(conn)
        .await?;
    Ok(highest.unwrap_or(0.0) + 100.0)
}

pub async fn by_ids(conn: &mut DbConn, user_id: Uuid, ids: &[Uuid]) -> AppResult<Vec<Habit>> {
    let rows = habits::table
        .filter(habits::user_id.eq(user_id))
        .filter(habits::id.eq_any(ids))
        .select(HabitRow::as_select())
        .load(conn)
        .await?;
    Ok(rows.into_iter().map(Habit::from).collect())
}

pub async fn active_ids(conn: &mut DbConn, user_id: Uuid) -> AppResult<Vec<Uuid>> {
    Ok(habits::table
        .filter(habits::user_id.eq(user_id))
        .filter(habits::archived_at.is_null())
        .select(habits::id)
        .load(conn)
        .await?)
}
