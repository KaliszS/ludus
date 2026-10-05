use std::collections::HashMap;

use chrono::NaiveDate;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use uuid::Uuid;

use super::pool::DbConn;
use super::schema::{
    plan_requirement_habits, plan_requirement_quotas, plan_requirements, plan_tiers, plans,
};
use crate::domain::{Measure, Medal, Period, Plan, Requirement, Tier, TierQuota};
use crate::error::{AppError, AppResult};

#[derive(Queryable, Selectable)]
#[diesel(table_name = plans, check_for_backend(diesel::pg::Pg))]
pub struct PlanRow {
    pub id: Uuid,
    pub name: String,
    pub period: String,
    pub position: f64,
    pub archived_on: Option<NaiveDate>,
}

#[derive(Insertable)]
#[diesel(table_name = plans)]
pub struct NewPlan {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub period: String,
    pub position: f64,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = plans)]
pub struct PlanChanges {
    pub name: Option<String>,
    pub period: Option<String>,
    pub position: Option<f64>,
    pub archived_on: Option<Option<NaiveDate>>,
}

impl PlanChanges {
    fn is_empty(&self) -> bool {
        self.name.is_none()
            && self.period.is_none()
            && self.position.is_none()
            && self.archived_on.is_none()
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = plan_tiers, check_for_backend(diesel::pg::Pg))]
struct TierRow {
    id: Uuid,
    plan_id: Uuid,
    name: Option<String>,
    medal: Option<String>,
    position: f64,
    retired_on: Option<NaiveDate>,
}

impl From<TierRow> for Tier {
    fn from(row: TierRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            medal: row.medal.as_deref().and_then(Medal::parse),
            position: row.position,
            retired_on: row.retired_on,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = plan_tiers)]
pub struct NewTier {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub name: Option<String>,
    pub medal: Option<String>,
    pub position: f64,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = plan_tiers)]
pub struct TierChanges {
    pub name: Option<Option<String>>,
    pub medal: Option<Option<String>>,
    pub retired_on: Option<Option<NaiveDate>>,
}

fn owned(user_id: Uuid) -> plans::BoxedQuery<'static, diesel::pg::Pg> {
    plans::table.filter(plans::user_id.eq(user_id)).into_boxed()
}

/// One query for the tiers of every plan listed, not one per plan.
async fn with_tiers(conn: &mut DbConn, rows: Vec<PlanRow>) -> AppResult<Vec<Plan>> {
    let ids: Vec<Uuid> = rows.iter().map(|row| row.id).collect();
    let tiers: Vec<TierRow> = plan_tiers::table
        .filter(plan_tiers::plan_id.eq_any(&ids))
        .order((plan_tiers::position.asc(), plan_tiers::created_at.asc()))
        .select(TierRow::as_select())
        .load(conn)
        .await?;
    let mut by_plan: HashMap<Uuid, Vec<Tier>> = HashMap::new();
    for row in tiers {
        by_plan.entry(row.plan_id).or_default().push(row.into());
    }

    Ok(rows
        .into_iter()
        .map(|row| Plan {
            tiers: by_plan.remove(&row.id).unwrap_or_default(),
            id: row.id,
            name: row.name,
            period: Period::parse(&row.period).unwrap_or(Period::Week),
            position: row.position,
            archived_on: row.archived_on,
        })
        .collect())
}

pub async fn list(
    conn: &mut DbConn,
    user_id: Uuid,
    period: Option<Period>,
    include_archived: bool,
) -> AppResult<Vec<Plan>> {
    let mut query = owned(user_id);
    if let Some(period) = period {
        query = query.filter(plans::period.eq(period.as_str()));
    }
    if !include_archived {
        query = query.filter(plans::archived_on.is_null());
    }
    let rows = query
        .order((plans::position.asc(), plans::name.asc()))
        .select(PlanRow::as_select())
        .load(conn)
        .await?;
    with_tiers(conn, rows).await
}

pub async fn get(conn: &mut DbConn, user_id: Uuid, id: Uuid) -> AppResult<Plan> {
    let row = owned(user_id)
        .filter(plans::id.eq(id))
        .select(PlanRow::as_select())
        .first(conn)
        .await?;
    one(with_tiers(conn, vec![row]).await?)
}

fn one(mut plans: Vec<Plan>) -> AppResult<Plan> {
    plans.pop().ok_or(AppError::NotFound)
}

/// A plan never exists without its base tier, the one its streak is judged on.
pub async fn insert(conn: &mut DbConn, plan: NewPlan, base: NewTier) -> AppResult<Plan> {
    let row = conn
        .transaction(async |conn| {
            let row = diesel::insert_into(plans::table)
                .values(plan)
                .returning(PlanRow::as_select())
                .get_result(conn)
                .await?;
            diesel::insert_into(plan_tiers::table)
                .values(base)
                .execute(conn)
                .await?;
            Ok::<_, diesel::result::Error>(row)
        })
        .await?;
    one(with_tiers(conn, vec![row]).await?)
}

pub async fn update(
    conn: &mut DbConn,
    user_id: Uuid,
    id: Uuid,
    changes: PlanChanges,
) -> AppResult<Plan> {
    if changes.is_empty() {
        return get(conn, user_id, id).await;
    }
    let row = diesel::update(
        plans::table
            .filter(plans::id.eq(id))
            .filter(plans::user_id.eq(user_id)),
    )
    .set(changes)
    .returning(PlanRow::as_select())
    .get_result(conn)
    .await?;
    one(with_tiers(conn, vec![row]).await?)
}

pub async fn delete(conn: &mut DbConn, user_id: Uuid, id: Uuid) -> AppResult<()> {
    let affected = diesel::delete(
        plans::table
            .filter(plans::id.eq(id))
            .filter(plans::user_id.eq(user_id)),
    )
    .execute(conn)
    .await?;
    if affected == 0 {
        return Err(AppError::NotFound);
    }
    Ok(())
}

pub async fn next_position(conn: &mut DbConn, user_id: Uuid, period: Period) -> AppResult<f64> {
    let highest: Option<f64> = plans::table
        .filter(plans::user_id.eq(user_id))
        .filter(plans::period.eq(period.as_str()))
        .select(diesel::dsl::max(plans::position))
        .first(conn)
        .await?;
    Ok(highest.unwrap_or(0.0) + 100.0)
}

/// The plan a live tier belongs to, scoped to its owner. Retired tiers are part of
/// the record and cannot be changed.
pub async fn tier_plan(conn: &mut DbConn, user_id: Uuid, tier_id: Uuid) -> AppResult<Uuid> {
    Ok(plan_tiers::table
        .inner_join(plans::table)
        .filter(plan_tiers::id.eq(tier_id))
        .filter(plan_tiers::retired_on.is_null())
        .filter(plans::user_id.eq(user_id))
        .select(plan_tiers::plan_id)
        .first(conn)
        .await?)
}

pub async fn insert_tier(conn: &mut DbConn, tier: NewTier) -> AppResult<()> {
    diesel::insert_into(plan_tiers::table)
        .values(tier)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn update_tier(conn: &mut DbConn, id: Uuid, changes: TierChanges) -> AppResult<()> {
    diesel::update(plan_tiers::table.find(id))
        .set(changes)
        .execute(conn)
        .await?;
    Ok(())
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = plan_requirements, check_for_backend(diesel::pg::Pg))]
pub struct RequirementRow {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub name: Option<String>,
    pub measure: String,
    pub position: f64,
    pub valid_from: NaiveDate,
    pub valid_to: Option<NaiveDate>,
}

#[derive(Insertable)]
#[diesel(table_name = plan_requirements)]
pub struct NewRequirement {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub name: Option<String>,
    pub measure: String,
    pub position: f64,
    pub valid_from: NaiveDate,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = plan_requirements)]
pub struct RequirementChanges {
    pub name: Option<Option<String>>,
    pub measure: Option<String>,
    pub position: Option<f64>,
}

impl RequirementChanges {
    fn is_empty(&self) -> bool {
        self.name.is_none() && self.measure.is_none() && self.position.is_none()
    }
}

pub async fn requirements_for(conn: &mut DbConn, plan_ids: &[Uuid]) -> AppResult<Vec<Requirement>> {
    let rows: Vec<RequirementRow> = plan_requirements::table
        .filter(plan_requirements::plan_id.eq_any(plan_ids))
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
    // In tier order, so a requirement's first quota is its lowest.
    let quotas: Vec<(Uuid, Uuid, f64)> = plan_requirement_quotas::table
        .inner_join(plan_tiers::table)
        .filter(plan_requirement_quotas::requirement_id.eq_any(&ids))
        .order((plan_tiers::position.asc(), plan_tiers::created_at.asc()))
        .select((
            plan_requirement_quotas::requirement_id,
            plan_requirement_quotas::tier_id,
            plan_requirement_quotas::quota,
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
            quotas: quotas
                .iter()
                .filter(|(requirement_id, _, _)| *requirement_id == row.id)
                .map(|(_, tier_id, quota)| TierQuota {
                    tier_id: *tier_id,
                    quota: *quota,
                })
                .collect(),
            id: row.id,
            plan_id: row.plan_id,
            name: row.name,
            measure: Measure::parse(&row.measure).unwrap_or(Measure::Amount),
            position: row.position,
            valid_from: row.valid_from,
            valid_to: row.valid_to,
        })
        .collect())
}

/// The plan a requirement belongs to, scoped to its owner. Only the version in
/// force can be changed; earlier ones are the record of what was asked.
pub async fn requirement_plan(conn: &mut DbConn, user_id: Uuid, id: Uuid) -> AppResult<Uuid> {
    Ok(plan_requirements::table
        .inner_join(plans::table)
        .filter(plan_requirements::id.eq(id))
        .filter(plan_requirements::valid_to.is_null())
        .filter(plans::user_id.eq(user_id))
        .select(plan_requirements::plan_id)
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

/// A requirement is its row, its habits and its quotas; it lands whole or not at all.
pub async fn insert_requirement(
    conn: &mut DbConn,
    requirement: NewRequirement,
    habit_ids: &[Uuid],
    quotas: &[TierQuota],
) -> AppResult<Uuid> {
    let id = requirement.id;
    conn.transaction(async |conn| {
        diesel::insert_into(plan_requirements::table)
            .values(requirement)
            .execute(conn)
            .await?;
        insert_members(conn, id, habit_ids).await?;
        insert_quotas(conn, id, quotas).await
    })
    .await?;
    Ok(id)
}

/// Closes one version and opens its successor in the same transaction, so there is
/// never a moment with neither in force, or with both.
pub async fn replace_requirement(
    conn: &mut DbConn,
    old: Uuid,
    next: NewRequirement,
    habit_ids: &[Uuid],
    quotas: &[TierQuota],
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
        insert_quotas(conn, id, quotas).await
    })
    .await?;
    Ok(id)
}

pub async fn update_requirement(
    conn: &mut DbConn,
    id: Uuid,
    changes: RequirementChanges,
    habit_ids: Option<&[Uuid]>,
    quotas: Option<&[TierQuota]>,
) -> AppResult<()> {
    conn.transaction(async |conn| {
        if !changes.is_empty() {
            diesel::update(plan_requirements::table.find(id))
                .set(changes)
                .execute(conn)
                .await?;
        }
        if let Some(habit_ids) = habit_ids {
            diesel::delete(
                plan_requirement_habits::table
                    .filter(plan_requirement_habits::requirement_id.eq(id)),
            )
            .execute(conn)
            .await?;
            insert_members(conn, id, habit_ids).await?;
        }
        if let Some(quotas) = quotas {
            diesel::delete(
                plan_requirement_quotas::table
                    .filter(plan_requirement_quotas::requirement_id.eq(id)),
            )
            .execute(conn)
            .await?;
            insert_quotas(conn, id, quotas).await?;
        }
        Ok::<_, diesel::result::Error>(())
    })
    .await?;
    Ok(())
}

async fn insert_members(
    conn: &mut AsyncPgConnection,
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

async fn insert_quotas(
    conn: &mut AsyncPgConnection,
    id: Uuid,
    quotas: &[TierQuota],
) -> Result<(), diesel::result::Error> {
    let values: Vec<_> = quotas
        .iter()
        .map(|quota| {
            (
                plan_requirement_quotas::requirement_id.eq(id),
                plan_requirement_quotas::tier_id.eq(quota.tier_id),
                plan_requirement_quotas::quota.eq(quota.quota),
            )
        })
        .collect();
    diesel::insert_into(plan_requirement_quotas::table)
        .values(values)
        .execute(conn)
        .await?;
    Ok(())
}

/// Whether anything was asked of this plan before `start`, which is what makes
/// reinterpreting its periods rewrite history.
pub async fn has_history(conn: &mut DbConn, plan_id: Uuid, start: NaiveDate) -> AppResult<bool> {
    Ok(diesel::select(diesel::dsl::exists(
        plan_requirements::table
            .filter(plan_requirements::plan_id.eq(plan_id))
            .filter(
                plan_requirements::valid_from
                    .lt(start)
                    .or(plan_requirements::valid_to.is_not_null()),
            ),
    ))
    .get_result(conn)
    .await?)
}

/// A plan that switches period before it has any history starts over on the new
/// period's boundary.
pub async fn realign_requirements(
    conn: &mut DbConn,
    plan_id: Uuid,
    start: NaiveDate,
) -> AppResult<()> {
    diesel::update(plan_requirements::table.filter(plan_requirements::plan_id.eq(plan_id)))
        .set(plan_requirements::valid_from.eq(start))
        .execute(conn)
        .await?;
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

pub async fn next_requirement_position(conn: &mut DbConn, plan_id: Uuid) -> AppResult<f64> {
    let highest: Option<f64> = plan_requirements::table
        .filter(plan_requirements::plan_id.eq(plan_id))
        .select(diesel::dsl::max(plan_requirements::position))
        .first(conn)
        .await?;
    Ok(highest.unwrap_or(0.0) + 100.0)
}

/// A habit deletion cascades to the member rows but leaves the requirement behind.
/// One with no members can never be satisfied, so the plan would be stuck. Scoped
/// to the owner: another account's broken requirements are not this caller's to sweep.
pub async fn delete_orphaned_requirements(conn: &mut DbConn, user_id: Uuid) -> AppResult<usize> {
    let owned_plans = plans::table
        .filter(plans::user_id.eq(user_id))
        .select(plans::id);
    Ok(diesel::delete(
        plan_requirements::table
            .filter(plan_requirements::plan_id.eq_any(owned_plans))
            .filter(diesel::dsl::not(diesel::dsl::exists(
                plan_requirement_habits::table
                    .filter(plan_requirement_habits::requirement_id.eq(plan_requirements::id)),
            ))),
    )
    .execute(conn)
    .await?)
}
