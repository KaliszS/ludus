use diesel::prelude::*;
use diesel_async::RunQueryDsl;
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
}

impl From<LevelRow> for PlanLevel {
    fn from(row: LevelRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            period: Period::parse(&row.period).unwrap_or(Period::Week),
            position: row.position,
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
}

impl LevelChanges {
    fn is_empty(&self) -> bool {
        self.name.is_none() && self.period.is_none() && self.position.is_none()
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
) -> AppResult<Vec<PlanLevel>> {
    let mut query = owned(user_id);
    if let Some(period) = period {
        query = query.filter(plan_levels::period.eq(period.as_str()));
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
        })
        .collect())
}

/// The level a requirement belongs to, scoped to its owner.
pub async fn requirement_level(conn: &mut DbConn, user_id: Uuid, id: Uuid) -> AppResult<Uuid> {
    Ok(plan_requirements::table
        .inner_join(plan_levels::table)
        .filter(plan_requirements::id.eq(id))
        .filter(plan_levels::user_id.eq(user_id))
        .select(plan_requirements::level_id)
        .first(conn)
        .await?)
}

async fn replace_members(conn: &mut DbConn, id: Uuid, habit_ids: &[Uuid]) -> AppResult<()> {
    diesel::delete(
        plan_requirement_habits::table.filter(plan_requirement_habits::requirement_id.eq(id)),
    )
    .execute(conn)
    .await?;

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
/// One with no members can never be satisfied, so the level would be stuck.
pub async fn delete_orphaned_requirements(conn: &mut DbConn) -> AppResult<usize> {
    Ok(diesel::delete(plan_requirements::table.filter(
        diesel::dsl::not(diesel::dsl::exists(plan_requirement_habits::table.filter(
            plan_requirement_habits::requirement_id.eq(plan_requirements::id),
        ))),
    ))
    .execute(conn)
    .await?)
}
