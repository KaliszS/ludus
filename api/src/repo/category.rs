use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use super::pool::DbConn;
use super::schema::habit_categories;
use crate::domain::HabitCategory;
use crate::error::{AppError, AppResult};

#[derive(Queryable, Selectable)]
#[diesel(table_name = habit_categories, check_for_backend(diesel::pg::Pg))]
pub struct CategoryRow {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub position: f64,
}

impl From<CategoryRow> for HabitCategory {
    fn from(row: CategoryRow) -> Self {
        Self {
            id: row.id,
            parent_id: row.parent_id,
            name: row.name,
            position: row.position,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = habit_categories)]
pub struct NewCategory {
    pub id: Uuid,
    pub user_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub position: f64,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = habit_categories)]
pub struct CategoryChanges {
    pub parent_id: Option<Option<Uuid>>,
    pub name: Option<String>,
    pub position: Option<f64>,
}

impl CategoryChanges {
    fn is_empty(&self) -> bool {
        self.parent_id.is_none() && self.name.is_none() && self.position.is_none()
    }
}

fn owned(user_id: Uuid) -> habit_categories::BoxedQuery<'static, diesel::pg::Pg> {
    habit_categories::table
        .filter(habit_categories::user_id.eq(user_id))
        .into_boxed()
}

/// Sibling names are unique, so a clash reads as a conflict instead of a crash.
fn name_taken(err: diesel::result::Error) -> AppError {
    match err {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            _,
        ) => AppError::Conflict {
            field: "name",
            message: "is already used by another category here",
        },
        other => other.into(),
    }
}

pub async fn list(conn: &mut DbConn, user_id: Uuid) -> AppResult<Vec<HabitCategory>> {
    let rows = owned(user_id)
        .order((
            habit_categories::position.asc(),
            habit_categories::created_at.asc(),
        ))
        .select(CategoryRow::as_select())
        .load(conn)
        .await?;
    Ok(rows.into_iter().map(HabitCategory::from).collect())
}

pub async fn get(conn: &mut DbConn, user_id: Uuid, id: Uuid) -> AppResult<HabitCategory> {
    let row = owned(user_id)
        .filter(habit_categories::id.eq(id))
        .select(CategoryRow::as_select())
        .first(conn)
        .await?;
    Ok(row.into())
}

pub async fn has_children(conn: &mut DbConn, id: Uuid) -> AppResult<bool> {
    let found = diesel::select(diesel::dsl::exists(
        habit_categories::table.filter(habit_categories::parent_id.eq(id)),
    ))
    .get_result(conn)
    .await?;
    Ok(found)
}

pub async fn insert(conn: &mut DbConn, category: NewCategory) -> AppResult<HabitCategory> {
    let row = diesel::insert_into(habit_categories::table)
        .values(category)
        .returning(CategoryRow::as_select())
        .get_result(conn)
        .await
        .map_err(name_taken)?;
    Ok(row.into())
}

pub async fn update(
    conn: &mut DbConn,
    user_id: Uuid,
    id: Uuid,
    changes: CategoryChanges,
) -> AppResult<HabitCategory> {
    if changes.is_empty() {
        return get(conn, user_id, id).await;
    }
    let row = diesel::update(
        habit_categories::table
            .filter(habit_categories::id.eq(id))
            .filter(habit_categories::user_id.eq(user_id)),
    )
    .set(changes)
    .returning(CategoryRow::as_select())
    .get_result(conn)
    .await
    .map_err(name_taken)?;
    Ok(row.into())
}

pub async fn delete(conn: &mut DbConn, user_id: Uuid, id: Uuid) -> AppResult<()> {
    let affected = diesel::delete(
        habit_categories::table
            .filter(habit_categories::id.eq(id))
            .filter(habit_categories::user_id.eq(user_id)),
    )
    .execute(conn)
    .await?;
    if affected == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

/// After the last sibling, so a new category lands at the end of its level.
pub async fn next_position(
    conn: &mut DbConn,
    user_id: Uuid,
    parent_id: Option<Uuid>,
) -> AppResult<f64> {
    let mut query = owned(user_id);
    query = match parent_id {
        Some(parent) => query.filter(habit_categories::parent_id.eq(parent)),
        None => query.filter(habit_categories::parent_id.is_null()),
    };
    let highest: Option<f64> = query
        .select(diesel::dsl::max(habit_categories::position))
        .first(conn)
        .await?;
    Ok(highest.unwrap_or(0.0) + 100.0)
}
