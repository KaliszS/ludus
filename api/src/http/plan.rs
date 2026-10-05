use axum::http::StatusCode;
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::dto::List;
use super::extract::{Json, Path, Query};
use super::habit::HabitResponse;
use std::collections::HashMap;

use crate::domain::{
    Medal, PeriodOutcome, PeriodProgress, Plan, PlanOutcome, PlanProgress, Requirement,
    RequirementProgress, Tier, TierQuota,
};
use crate::error::AppResult;
use crate::service::UserService;
use crate::service::parse_period;
use crate::service::plan::{
    CreatePlan, CreateRequirement, CreateTier, QuotaInput, UpdatePlan, UpdateRequirement,
    UpdateTier,
};

#[derive(Serialize)]
pub struct TierResponse {
    id: Uuid,
    name: Option<String>,
    medal: Option<&'static str>,
    position: f64,
    /// Retired tiers stay listed: past periods were judged against them.
    retired_on: Option<NaiveDate>,
}

impl From<Tier> for TierResponse {
    fn from(tier: Tier) -> Self {
        Self {
            id: tier.id,
            name: tier.name,
            medal: tier.medal.map(Medal::as_str),
            position: tier.position,
            retired_on: tier.retired_on,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct QuotaResponse {
    tier_id: Uuid,
    quota: f64,
}

impl From<TierQuota> for QuotaResponse {
    fn from(quota: TierQuota) -> Self {
        Self {
            tier_id: quota.tier_id,
            quota: quota.quota,
        }
    }
}

impl From<QuotaResponse> for QuotaInput {
    fn from(quota: QuotaResponse) -> Self {
        Self {
            tier_id: quota.tier_id,
            quota: quota.quota,
        }
    }
}

fn quotas(quotas: Vec<TierQuota>) -> Vec<QuotaResponse> {
    quotas.into_iter().map(Into::into).collect()
}

#[derive(Serialize)]
pub struct PlanResponse {
    id: Uuid,
    name: String,
    period: &'static str,
    position: f64,
    /// The day it was archived; `None` while it is in use.
    archived_on: Option<NaiveDate>,
    /// Lowest first.
    tiers: Vec<TierResponse>,
}

impl From<Plan> for PlanResponse {
    fn from(plan: Plan) -> Self {
        Self {
            id: plan.id,
            name: plan.name,
            period: plan.period.as_str(),
            position: plan.position,
            archived_on: plan.archived_on,
            tiers: plan.tiers.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
pub struct RequirementResponse {
    id: Uuid,
    plan_id: Uuid,
    name: Option<String>,
    habit_ids: Vec<Uuid>,
    quotas: Vec<QuotaResponse>,
    measure: &'static str,
    position: f64,
}

impl From<Requirement> for RequirementResponse {
    fn from(requirement: Requirement) -> Self {
        Self {
            id: requirement.id,
            plan_id: requirement.plan_id,
            name: requirement.name,
            habit_ids: requirement.habit_ids,
            quotas: quotas(requirement.quotas),
            measure: requirement.measure.as_str(),
            position: requirement.position,
        }
    }
}

#[derive(Serialize)]
pub struct RequirementProgressResponse {
    id: Uuid,
    name: Option<String>,
    habits: Vec<HabitResponse>,
    quotas: Vec<QuotaResponse>,
    measure: &'static str,
    done: f64,
    /// Whether it holds its part of the plan's lowest tier; a goal that only asks
    /// further up never holds the plan back.
    met: bool,
}

impl RequirementProgressResponse {
    fn new(item: RequirementProgress, base: Option<Uuid>) -> Self {
        let base_quota = item
            .quotas
            .iter()
            .find(|quota| Some(quota.tier_id) == base)
            .map(|quota| quota.quota);
        Self {
            met: base_quota.is_none_or(|quota| item.done >= quota),
            id: item.id,
            name: item.name,
            habits: item.habits.into_iter().map(Into::into).collect(),
            quotas: quotas(item.quotas),
            measure: item.measure.as_str(),
            done: item.done,
        }
    }
}

#[derive(Serialize)]
pub struct StandingResponse {
    tier_id: Uuid,
    reached: bool,
}

#[derive(Serialize)]
pub struct PlanProgressResponse {
    #[serde(flatten)]
    plan: PlanResponse,
    met: bool,
    streak: u32,
    /// The tiers this period asks at, lowest first, and whether each is reached.
    standing: Vec<StandingResponse>,
    items: Vec<RequirementProgressResponse>,
}

impl From<PlanProgress> for PlanProgressResponse {
    fn from(progress: PlanProgress) -> Self {
        let base = progress.standing.first().map(|step| step.tier.id);
        Self {
            met: crate::domain::plan::met(&progress.standing),
            streak: progress.streak,
            standing: progress
                .standing
                .into_iter()
                .map(|step| StandingResponse {
                    tier_id: step.tier.id,
                    reached: step.reached,
                })
                .collect(),
            plan: progress.plan.into(),
            items: progress
                .items
                .into_iter()
                .map(|item| RequirementProgressResponse::new(item, base))
                .collect(),
        }
    }
}

#[derive(Serialize)]
pub struct PeriodProgressResponse {
    period: &'static str,
    period_start: NaiveDate,
    period_end: NaiveDate,
    plans: Vec<PlanProgressResponse>,
}

impl From<PeriodProgress> for PeriodProgressResponse {
    fn from(progress: PeriodProgress) -> Self {
        Self {
            period: progress.period.as_str(),
            period_start: progress.period_start,
            period_end: progress.period_end,
            plans: progress.plans.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
pub struct PeriodOutcomeResponse {
    period_start: NaiveDate,
    period_end: NaiveDate,
    plans: Vec<PlanOutcomeResponse>,
}

#[derive(Serialize)]
pub struct PlanOutcomeResponse {
    id: Uuid,
    name: String,
    archived: bool,
    met: bool,
    reached: usize,
    total: usize,
    medals: Vec<&'static str>,
}

impl From<PlanOutcome> for PlanOutcomeResponse {
    fn from(outcome: PlanOutcome) -> Self {
        Self {
            met: outcome.met,
            medals: outcome.medals.into_iter().map(Medal::as_str).collect(),
            id: outcome.plan.id,
            archived: outcome.plan.archived_on.is_some(),
            name: outcome.plan.name,
            reached: outcome.reached,
            total: outcome.total,
        }
    }
}

impl From<PeriodOutcome> for PeriodOutcomeResponse {
    fn from(outcome: PeriodOutcome) -> Self {
        Self {
            period_start: outcome.period_start,
            period_end: outcome.period_end,
            plans: outcome.plans.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
pub struct VersionResponse {
    id: Uuid,
    name: Option<String>,
    quotas: Vec<QuotaResponse>,
    measure: &'static str,
    /// Lets a client mark what changed where a stretch begins.
    valid_from: NaiveDate,
    habits: Vec<HabitResponse>,
}

#[derive(Serialize)]
pub struct EraResponse {
    from: NaiveDate,
    to: Option<NaiveDate>,
    requirements: Vec<VersionResponse>,
}

#[derive(Serialize)]
pub struct VersionsResponse {
    plan: PlanResponse,
    eras: Vec<EraResponse>,
}

#[derive(Deserialize)]
pub struct PlanQuery {
    period: Option<String>,
    #[serde(default)]
    archived: bool,
}

/// The client's today on an edit, so the change lands in the period its screen shows.
#[derive(Deserialize, Default)]
pub struct OnQuery {
    on: Option<NaiveDate>,
}

#[derive(Deserialize)]
pub struct ProgressQuery {
    period: Option<String>,
    on: Option<NaiveDate>,
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    period: Option<String>,
    count: Option<u32>,
    on: Option<NaiveDate>,
}

#[derive(Deserialize)]
pub struct CreatePlanBody {
    name: String,
    period: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct UpdatePlanBody {
    name: Option<String>,
    period: Option<String>,
    position: Option<f64>,
    archived: Option<bool>,
}

#[derive(Deserialize)]
pub struct CreateTierBody {
    name: Option<String>,
    medal: Option<String>,
    after: Option<Uuid>,
}

#[derive(Deserialize, Default)]
pub struct UpdateTierBody {
    #[serde(default, deserialize_with = "super::dto::double_option")]
    name: Option<Option<String>>,
    #[serde(default, deserialize_with = "super::dto::double_option")]
    medal: Option<Option<String>>,
}

#[derive(Deserialize)]
pub struct CreateRequirementBody {
    name: Option<String>,
    habit_ids: Vec<Uuid>,
    quotas: Vec<QuotaResponse>,
    measure: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct UpdateRequirementBody {
    #[serde(default, deserialize_with = "super::dto::double_option")]
    name: Option<Option<String>>,
    habit_ids: Option<Vec<Uuid>>,
    quotas: Option<Vec<QuotaResponse>>,
    measure: Option<String>,
    position: Option<f64>,
}

pub async fn list(
    service: UserService,
    Query(query): Query<PlanQuery>,
) -> AppResult<Json<List<PlanResponse>>> {
    let period = query.period.as_deref().map(parse_period).transpose()?;
    Ok(Json(
        service
            .plans(period, query.archived)
            .await?
            .into_iter()
            .collect(),
    ))
}

pub async fn show(service: UserService, Path(id): Path<Uuid>) -> AppResult<Json<PlanResponse>> {
    Ok(Json(service.plan(id).await?.into()))
}

pub async fn create(
    service: UserService,
    Json(body): Json<CreatePlanBody>,
) -> AppResult<(StatusCode, Json<PlanResponse>)> {
    let plan = service
        .create_plan(CreatePlan {
            name: body.name,
            period: body.period,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(plan.into())))
}

pub async fn update(
    service: UserService,
    Path(id): Path<Uuid>,
    Query(query): Query<OnQuery>,
    Json(body): Json<UpdatePlanBody>,
) -> AppResult<Json<PlanResponse>> {
    let plan = service
        .update_plan(
            id,
            UpdatePlan {
                name: body.name,
                period: body.period,
                position: body.position,
                archived: body.archived,
            },
            query.on,
        )
        .await?;
    Ok(Json(plan.into()))
}

pub async fn remove(service: UserService, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    service.delete_plan(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_tier(
    service: UserService,
    Path(plan_id): Path<Uuid>,
    Json(body): Json<CreateTierBody>,
) -> AppResult<(StatusCode, Json<PlanResponse>)> {
    let plan = service
        .create_tier(
            plan_id,
            CreateTier {
                name: body.name,
                medal: body.medal,
                after: body.after,
            },
        )
        .await?;
    Ok((StatusCode::CREATED, Json(plan.into())))
}

pub async fn update_tier(
    service: UserService,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateTierBody>,
) -> AppResult<Json<PlanResponse>> {
    let plan = service
        .update_tier(
            id,
            UpdateTier {
                name: body.name,
                medal: body.medal,
            },
        )
        .await?;
    Ok(Json(plan.into()))
}

pub async fn retire_tier(
    service: UserService,
    Path(id): Path<Uuid>,
    Query(query): Query<OnQuery>,
) -> AppResult<Json<PlanResponse>> {
    Ok(Json(service.retire_tier(id, query.on).await?.into()))
}

pub async fn versions(
    service: UserService,
    Path(id): Path<Uuid>,
) -> AppResult<Json<VersionsResponse>> {
    let (plan, eras, habits) = service.plan_versions(id).await?;
    let habits: HashMap<Uuid, _> = habits.into_iter().map(|habit| (habit.id, habit)).collect();
    Ok(Json(VersionsResponse {
        plan: plan.into(),
        eras: eras
            .into_iter()
            .map(|era| EraResponse {
                from: era.from,
                to: era.to,
                requirements: era
                    .requirements
                    .into_iter()
                    .map(|version| VersionResponse {
                        habits: version
                            .habit_ids
                            .iter()
                            .filter_map(|habit_id| habits.get(habit_id).cloned().map(Into::into))
                            .collect(),
                        id: version.id,
                        name: version.name,
                        quotas: quotas(version.quotas),
                        measure: version.measure.as_str(),
                        valid_from: version.valid_from,
                    })
                    .collect(),
            })
            .collect(),
    }))
}

pub async fn list_requirements(
    service: UserService,
    Path(plan_id): Path<Uuid>,
) -> AppResult<Json<List<RequirementResponse>>> {
    Ok(Json(
        service.requirements(plan_id).await?.into_iter().collect(),
    ))
}

pub async fn create_requirement(
    service: UserService,
    Path(plan_id): Path<Uuid>,
    Query(query): Query<OnQuery>,
    Json(body): Json<CreateRequirementBody>,
) -> AppResult<(StatusCode, Json<RequirementResponse>)> {
    let requirement = service
        .create_requirement(
            plan_id,
            CreateRequirement {
                name: body.name,
                habit_ids: body.habit_ids,
                quotas: body.quotas.into_iter().map(Into::into).collect(),
                measure: body.measure,
            },
            query.on,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(requirement.into())))
}

pub async fn update_requirement(
    service: UserService,
    Path(id): Path<Uuid>,
    Query(query): Query<OnQuery>,
    Json(body): Json<UpdateRequirementBody>,
) -> AppResult<StatusCode> {
    service
        .update_requirement(
            id,
            UpdateRequirement {
                name: body.name,
                habit_ids: body.habit_ids,
                quotas: body
                    .quotas
                    .map(|quotas| quotas.into_iter().map(Into::into).collect()),
                measure: body.measure,
                position: body.position,
            },
            query.on,
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_requirement(
    service: UserService,
    Path(id): Path<Uuid>,
    Query(query): Query<OnQuery>,
) -> AppResult<StatusCode> {
    service.delete_requirement(id, query.on).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn progress(
    service: UserService,
    Query(query): Query<ProgressQuery>,
) -> AppResult<Json<PeriodProgressResponse>> {
    let period = parse_period(query.period.as_deref().unwrap_or("week"))?;
    let on = query.on.unwrap_or_else(|| Utc::now().date_naive());
    Ok(Json(service.plan_progress(period, on).await?.into()))
}

pub async fn history(
    service: UserService,
    Query(query): Query<HistoryQuery>,
) -> AppResult<Json<List<PeriodOutcomeResponse>>> {
    let period = parse_period(query.period.as_deref().unwrap_or("week"))?;
    let on = query.on.unwrap_or_else(|| Utc::now().date_naive());
    Ok(Json(
        service
            .plan_history(period, query.count.unwrap_or(12), on)
            .await?
            .into_iter()
            .collect(),
    ))
}
