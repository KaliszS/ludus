use axum::extract::State;
use axum::http::StatusCode;
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::dto::List;
use super::extract::{Json, Path, Query};
use super::habit::HabitResponse;
use crate::domain::{
    LevelOutcome, LevelProgress, PeriodOutcome, PlanLevel, PlanProgress, Requirement,
    RequirementProgress,
};
use crate::error::AppResult;
use crate::service::Service;
use crate::service::parse_period;
use crate::service::plan::{CreateLevel, CreateRequirement, UpdateLevel, UpdateRequirement};

#[derive(Serialize)]
pub struct LevelResponse {
    id: Uuid,
    name: String,
    period: &'static str,
    position: f64,
}

impl From<PlanLevel> for LevelResponse {
    fn from(level: PlanLevel) -> Self {
        Self {
            id: level.id,
            name: level.name,
            period: level.period.as_str(),
            position: level.position,
        }
    }
}

#[derive(Serialize)]
pub struct RequirementResponse {
    id: Uuid,
    level_id: Uuid,
    name: Option<String>,
    habit_ids: Vec<Uuid>,
    quota: f64,
    measure: &'static str,
    position: f64,
}

impl From<Requirement> for RequirementResponse {
    fn from(requirement: Requirement) -> Self {
        Self {
            id: requirement.id,
            level_id: requirement.level_id,
            name: requirement.name,
            habit_ids: requirement.habit_ids,
            quota: requirement.quota,
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
    quota: f64,
    measure: &'static str,
    done: f64,
    met: bool,
}

impl From<RequirementProgress> for RequirementProgressResponse {
    fn from(item: RequirementProgress) -> Self {
        Self {
            met: item.met(),
            id: item.id,
            name: item.name,
            habits: item.habits.into_iter().map(Into::into).collect(),
            quota: item.quota,
            measure: item.measure.as_str(),
            done: item.done,
        }
    }
}

#[derive(Serialize)]
pub struct LevelProgressResponse {
    #[serde(flatten)]
    level: LevelResponse,
    met: bool,
    streak: u32,
    items: Vec<RequirementProgressResponse>,
}

impl From<LevelProgress> for LevelProgressResponse {
    fn from(progress: LevelProgress) -> Self {
        Self {
            met: progress.met(),
            streak: progress.streak,
            level: progress.level.into(),
            items: progress.items.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
pub struct ProgressResponse {
    period: &'static str,
    period_start: NaiveDate,
    period_end: NaiveDate,
    levels: Vec<LevelProgressResponse>,
}

impl From<PlanProgress> for ProgressResponse {
    fn from(progress: PlanProgress) -> Self {
        Self {
            period: progress.period.as_str(),
            period_start: progress.period_start,
            period_end: progress.period_end,
            levels: progress.levels.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
pub struct OutcomeResponse {
    period_start: NaiveDate,
    period_end: NaiveDate,
    levels: Vec<LevelOutcomeResponse>,
}

#[derive(Serialize)]
pub struct LevelOutcomeResponse {
    id: Uuid,
    name: String,
    met: bool,
    reached: usize,
    total: usize,
}

impl From<LevelOutcome> for LevelOutcomeResponse {
    fn from(outcome: LevelOutcome) -> Self {
        Self {
            met: outcome.met(),
            id: outcome.level.id,
            name: outcome.level.name,
            reached: outcome.reached,
            total: outcome.total,
        }
    }
}

impl From<PeriodOutcome> for OutcomeResponse {
    fn from(outcome: PeriodOutcome) -> Self {
        Self {
            period_start: outcome.period_start,
            period_end: outcome.period_end,
            levels: outcome.levels.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Deserialize)]
pub struct LevelQuery {
    period: Option<String>,
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
pub struct CreateLevelBody {
    name: String,
    period: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct UpdateLevelBody {
    name: Option<String>,
    period: Option<String>,
    position: Option<f64>,
}

#[derive(Deserialize)]
pub struct CreateRequirementBody {
    name: Option<String>,
    habit_ids: Vec<Uuid>,
    quota: f64,
    measure: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct UpdateRequirementBody {
    #[serde(default, deserialize_with = "super::dto::double_option")]
    name: Option<Option<String>>,
    habit_ids: Option<Vec<Uuid>>,
    quota: Option<f64>,
    measure: Option<String>,
    position: Option<f64>,
}

pub async fn list_levels(
    State(service): State<Service>,
    Query(query): Query<LevelQuery>,
) -> AppResult<Json<List<LevelResponse>>> {
    let period = query.period.as_deref().map(parse_period).transpose()?;
    Ok(Json(
        service.plan_levels(period).await?.into_iter().collect(),
    ))
}

pub async fn show_level(
    State(service): State<Service>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<LevelResponse>> {
    Ok(Json(service.plan_level(id).await?.into()))
}

pub async fn create_level(
    State(service): State<Service>,
    Json(body): Json<CreateLevelBody>,
) -> AppResult<(StatusCode, Json<LevelResponse>)> {
    let level = service
        .create_plan_level(CreateLevel {
            name: body.name,
            period: body.period,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(level.into())))
}

pub async fn update_level(
    State(service): State<Service>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateLevelBody>,
) -> AppResult<Json<LevelResponse>> {
    let level = service
        .update_plan_level(
            id,
            UpdateLevel {
                name: body.name,
                period: body.period,
                position: body.position,
            },
        )
        .await?;
    Ok(Json(level.into()))
}

pub async fn remove_level(
    State(service): State<Service>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    service.delete_plan_level(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_requirements(
    State(service): State<Service>,
    Path(level_id): Path<Uuid>,
) -> AppResult<Json<List<RequirementResponse>>> {
    Ok(Json(
        service.requirements(level_id).await?.into_iter().collect(),
    ))
}

pub async fn create_requirement(
    State(service): State<Service>,
    Path(level_id): Path<Uuid>,
    Json(body): Json<CreateRequirementBody>,
) -> AppResult<(StatusCode, Json<RequirementResponse>)> {
    let requirement = service
        .create_requirement(
            level_id,
            CreateRequirement {
                name: body.name,
                habit_ids: body.habit_ids,
                quota: body.quota,
                measure: body.measure,
            },
        )
        .await?;
    Ok((StatusCode::CREATED, Json(requirement.into())))
}

pub async fn update_requirement(
    State(service): State<Service>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateRequirementBody>,
) -> AppResult<StatusCode> {
    service
        .update_requirement(
            id,
            UpdateRequirement {
                name: body.name,
                habit_ids: body.habit_ids,
                quota: body.quota,
                measure: body.measure,
                position: body.position,
            },
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_requirement(
    State(service): State<Service>,
    Path(id): Path<Uuid>,
) -> AppResult<StatusCode> {
    service.delete_requirement(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn progress(
    State(service): State<Service>,
    Query(query): Query<ProgressQuery>,
) -> AppResult<Json<ProgressResponse>> {
    let period = parse_period(query.period.as_deref().unwrap_or("week"))?;
    let on = query.on.unwrap_or_else(|| Utc::now().date_naive());
    Ok(Json(service.plan_progress(period, on).await?.into()))
}

pub async fn history(
    State(service): State<Service>,
    Query(query): Query<HistoryQuery>,
) -> AppResult<Json<List<OutcomeResponse>>> {
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
