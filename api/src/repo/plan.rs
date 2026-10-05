use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use super::pool::DbConn;
use super::schema::{plan_levels, plan_requirement_habits, plan_requirements};
use crate::domain::{Measure, Period, PlanLevel, Requirement};
use crate::error::{AppError, AppResult};

#[derive(Queryable, Selectable)]
#[diesel(table_name = plan_levels, check_for_backend(diesel::pg::Pg))]
pub struct LevelRow {
    pub id: Uuid,
    pub name: String,
    pub period: String,
    pub position: f64,
    pub archived_on: Option<NaiveDate>,
}

impl From<LevelRow> for PlanLevel {
    fn from(row: LevelRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            period: Period::parse(&row.period).unwrap_or(Period::Week),
            position: row.position,
            archived_on: row.archived_on,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = plan_levels)]
pub struct NewLevel {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub period: String,
    pub position: f64,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = plan_levels)]
pub struct LevelChanges {
    pub name: Option<String>,
    pub period: Option<String>,
    pub position: Option<f64>,
    pub archived_on: Option<Option<NaiveDate>>,
}

impl LevelChanges {
    fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.period.is_none()
            && self.position.is_none()
            && self.archived_on.is_none()
    }
}

fn owned(user_id: Uuid) -> plan_levels::BoxedQuery<'static, diesel::pg::Pg> {
    plan_levels::table
        .filter(plan_levels::user_id.eq(user_id))
        .into_boxed()
}

pub async fn list_levels(
    conn: &mut DbConn,
    user_id: Uuid,
    period: Option<Period>,
    include_archived: bool,
) -> AppResult<Vec<PlanLevel>> {
    let mut query = owned(user_id);
    if let Some(period) = period {
        query = query.filter(plan_levels::period.eq(period.as_str()));
    }
    if !include_archived {
        query = query.filter(plan_levels::archived_on.is_null());
    }
    let rows = query
        .order((plan_levels::position.asc(), plan_levels::name.asc()))
        .select(LevelRow::as_select())
        .load(conn)
        .await?;
    Ok(rows.into_iter().map(PlanLevel::from).collect())
}

pub async fn get_level(conn: &mut DbConn, user_id: Uuid, id: Uuid) -> AppResult<PlanLevel> {
    let row = owned(user_id)
        .filter(plan_levels::id.eq(id))
        .select(LevelRow::as_select())
        .first(conn)
        .await?;
    Ok(row.into())
}

pub async fn insert_level(conn: &mut DbConn, level: NewLevel) -> AppResult<PlanLevel> {
    let row = diesel::insert_into(plan_levels::table)
        .values(level)
        .returning(LevelRow::as_select())
        .get_result(conn)
        .await?;
    Ok(row.into())
}

pub async fn update_level(
    conn: &mut DbConn,
    user_id: Uuid,
    id: Uuid,
    changes: LevelChanges,
) -> AppResult<PlanLevel> {
    if changes.is_empty() {
        return get_level(conn, user_id, id).await;
    }
    let row = diesel::update(
        plan_levels::table
            .filter(plan_levels::id.eq(id))
            .filter(plan_levels::user_id.eq(user_id)),
    )
    .set(changes)
    .returning(LevelRow::as_select())
    .get_result(conn)
    .await?;
    Ok(row.into())
}

pub async fn delete_level(conn: &mut DbConn, user_id: Uuid, id: Uuid) -> AppResult<()> {
    let affected = diesel::delete(
        plan_levels::table
            .filter(plan_levels::id.eq(id))
            .filter(plan_levels::user_id.eq(user_id)),
    )
    .execute(conn)
    .await?;
    if affected == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub async fn next_level_position(
    conn: &mut DbConn,
    user_id: Uuid,
    period: Period,
) -> AppResult<f64> {
    let highest: Option<f64> = plan_levels::table
        .filter(plan_levels::user_id.eq(user_id))
        .filter(plan_levels::period.eq(period.as_str()))
        .select(diesel::dsl::max(plan_levels::position))
        .first(conn)
        .await?;
    Ok(highest.unwrap_or(0.0) + 100.0)
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = plan_requirements, check_for_backend(diesel::pg::Pg))]
pub struct RequirementRow {
    pub id: Uuid,
    pub level_id: Uuid,
    pub name: Option<String>,
    pub quota: f64,
    pub measure: String,
    pub position: f64,
    pub valid_from: NaiveDate,
    pub valid_to: Option<NaiveDate>,
}

#[derive(Insertable)]
#[diesel(table_name = plan_requirements)]
pub struct NewRequirement {
    pub id: Uuid,
    pub level_id: Uuid,
    pub name: Option<String>,
    pub quota: f64,
    pub measure: String,
    pub position: f64,
    pub valid_from: NaiveDate,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = plan_requirements)]
pub struct RequirementChanges {
    pub name: Option<Option<String>>,
    pub quota: Option<f64>,
    pub measure: Option<String>,
    pub position: Option<f64>,
}

impl RequirementChanges {
    fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.quota.is_none()
            && self.measure.is_none()
            && self.position.is_none()
    }
}

pub async fn requirements_for(
    conn: &mut DbConn,
    level_ids: &[Uuid],
) -> AppResult<Vec<Requirement>> {
    let rows: Vec<RequirementRow> = plan_requirements::table
        .filter(plan_requirements::level_id.eq_any(level_ids))
        .order((
            plan_requirements::position.asc(),
            plan_requirements::id.asc(),
        ))
        .select(RequirementRow::as_select())
        .load(conn)
        .await?;

    let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
    let members: Vec<(Uuid, Uuid)> = plan_requirement_habits::table
        .filter(plan_requirement_habits::requirement_id.eq_any(&ids))
        .select((
            plan_requirement_habits::requirement_id,
            plan_requirement_habits::habit_id,
        ))
        .load(conn)
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| Requirement {
            habit_ids: members
                .iter()
                .filter(|(requirement_id, _)| *requirement_id == row.id)
                .map(|(_, habit_id)| *habit_id)
                .collect(),
            id: row.id,
            level_id: row.level_id,
            name: row.name,
            quota: row.quota,
            measure: Measure::parse(&row.measure).unwrap_or(Measure::Amount),
            position: row.position,
            valid_from: row.valid_from,
            valid_to: row.valid_to,
        })
        .collect())
}

/// The level a requirement belongs to, scoped to its owner. Only the version in
/// force can be changed; earlier ones are the record of what was asked.
pub async fn requirement_level(conn: &mut DbConn, user_id: Uuid, id: Uuid) -> AppResult<Uuid> {
    Ok(plan_requirements::table
        .inner_join(plan_levels::table)
        .filter(plan_requirements::id.eq(id))
        .filter(plan_requirements::valid_to.is_null())
        .filter(plan_levels::user_id.eq(user_id))
        .select(plan_requirements::level_id)
        .first(conn)
        .await?)
}

/// Ends a version at `valid_to`, the start of the period the change happens in.
pub async fn close_requirement(conn: &mut DbConn, id: Uuid, valid_to: NaiveDate) -> AppResult<()> {
    diesel::update(plan_requirements::table.find(id))
        .set(plan_requirements::valid_to.eq(valid_to))
        .execute(conn)
        .await?;
    Ok(())
}

/// Closes one version and opens its successor in the same transaction, so there is
/// never a moment with neither in force, or with both.
pub async fn replace_requirement(
    conn: &mut DbConn,
    old: Uuid,
    next: NewRequirement,
    habit_ids: &[Uuid],
) -> AppResult<Uuid> {
    let id = next.id;
    let valid_to = next.valid_from;
    conn.transaction(async |conn| {
        diesel::update(plan_requirements::table.find(old))
            .set(plan_requirements::valid_to.eq(valid_to))
            .execute(conn)
            .await?;
        diesel::insert_into(plan_requirements::table)
            .values(next)
            .execute(conn)
            .await?;
        insert_members(conn, id, habit_ids).await?;
        Ok::<_, diesel::result::Error>(())
    })
    .await?;
    Ok(id)
}

/// Whether anything was asked of this level before `start`, which is what makes
/// reinterpreting its periods rewrite history.
pub async fn level_has_history(
    conn: &mut DbConn,
    level_id: Uuid,
    start: NaiveDate,
) -> AppResult<bool> {
    Ok(diesel::select(diesel::dsl::exists(
        plan_requirements::table
            .filter(plan_requirements::level_id.eq(level_id))
            .filter(
                plan_requirements::valid_from
                    .lt(start)
                    .or(plan_requirements::valid_to.is_not_null()),
            ),
    ))
    .get_result(conn)
    .await?)
}

/// A level that switches period before it has any history starts over on the new
/// period's boundary.
pub async fn realign_requirements(
    conn: &mut DbConn,
    level_id: Uuid,
    start: NaiveDate,
) -> AppResult<()> {
    diesel::update(plan_requirements::table.filter(plan_requirements::level_id.eq(level_id)))
        .set(plan_requirements::valid_from.eq(start))
        .execute(conn)
        .await?;
    Ok(())
}

async fn replace_members(conn: &mut DbConn, id: Uuid, habit_ids: &[Uuid]) -> AppResult<()> {
    diesel::delete(
        plan_requirement_habits::table.filter(plan_requirement_habits::requirement_id.eq(id)),
    )
    .execute(conn)
    .await?;
    Ok(insert_members(conn, id, habit_ids).await?)
}

async fn insert_members(
    conn: &mut diesel_async::AsyncPgConnection,
    id: Uuid,
    habit_ids: &[Uuid],
) -> Result<(), diesel::result::Error> {
    let values: Vec<_> = habit_ids
        .iter()
        .map(|habit_id| {
            (
                plan_requirement_habits::requirement_id.eq(id),
                plan_requirement_habits::habit_id.eq(*habit_id),
            )
        })
        .collect();

    diesel::insert_into(plan_requirement_habits::table)
        .values(values)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn insert_requirement(
    conn: &mut DbConn,
    requirement: NewRequirement,
    habit_ids: &[Uuid],
) -> AppResult<Uuid> {
    let id = requirement.id;
    diesel::insert_into(plan_requirements::table)
        .values(requirement)
        .execute(conn)
        .await?;
    replace_members(conn, id, habit_ids).await?;
    Ok(id)
}

pub async fn update_requirement(
    conn: &mut DbConn,
    id: Uuid,
    changes: RequirementChanges,
    habit_ids: Option<&[Uuid]>,
) -> AppResult<()> {
    if !changes.is_empty() {
        diesel::update(plan_requirements::table.filter(plan_requirements::id.eq(id)))
            .set(changes)
            .execute(conn)
            .await?;
    }
    if let Some(habit_ids) = habit_ids {
        replace_members(conn, id, habit_ids).await?;
    }
    Ok(())
}

pub async fn delete_requirement(conn: &mut DbConn, id: Uuid) -> AppResult<()> {
    let affected = diesel::delete(plan_requirements::table.filter(plan_requirements::id.eq(id)))
        .execute(conn)
        .await?;
    if affected == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub async fn next_requirement_position(conn: &mut DbConn, level_id: Uuid) -> AppResult<f64> {
    let highest: Option<f64> = plan_requirements::table
        .filter(plan_requirements::level_id.eq(level_id))
        .select(diesel::dsl::max(plan_requirements::position))
        .first(conn)
        .await?;
    Ok(highest.unwrap_or(0.0) + 100.0)
}

/// A habit deletion cascades to the member rows but leaves the requirement behind.
/// One with no members can never be satisfied, so the level would be stuck. Scoped
/// to the owner: another account's broken requirements are not this caller's to sweep.
pub async fn delete_orphaned_requirements(conn: &mut DbConn, user_id: Uuid) -> AppResult<usize> {
    let owned_levels = plan_levels::table
        .filter(plan_levels::user_id.eq(user_id))
        .select(plan_levels::id);
    Ok(diesel::delete(
        plan_requirements::table
            .filter(plan_requirements::level_id.eq_any(owned_levels))
            .filter(diesel::dsl::not(diesel::dsl::exists(
                plan_requirement_habits::table
                    .filter(plan_requirement_habits::requirement_id.eq(plan_requirements::id)),
            ))),
    )
    .execute(conn)
    .await?)
}
