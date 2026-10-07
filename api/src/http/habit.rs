use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::dto::{List, double_option};
use super::extract::{Json, Path, Query};
use crate::domain::Habit;
use crate::error::AppResult;
use crate::service::UserService;
use crate::service::habit::{CreateHabit, UpdateHabit};

#[derive(Serialize)]
pub struct HabitResponse {
    id: Uuid,
    name: String,
    description: String,
    icon: Option<String>,
    color: Option<String>,
    unit: Option<String>,
    tracking: &'static str,
    weekdays: Vec<i16>,
    category_id: Option<Uuid>,
    position: f64,
    archived: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<Habit> for HabitResponse {
    fn from(habit: Habit) -> Self {
        Self {
            id: habit.id,
            name: habit.name,
            description: habit.description,
            icon: habit.icon,
            color: habit.color,
            unit: habit.unit,
            tracking: habit.tracking.as_str(),
            weekdays: habit.weekdays,
            category_id: habit.category_id,
            position: habit.position,
            archived: habit.archived_at.is_some(),
            created_at: habit.created_at,
            updated_at: habit.updated_at,
        }
    }
}

#[derive(Deserialize, Default)]
pub struct ListQuery {
    #[serde(default)]
    archived: bool,
}

#[derive(Deserialize)]
pub struct CreateBody {
    name: String,
    description: Option<String>,
    icon: Option<String>,
    color: Option<String>,
    unit: Option<String>,
    tracking: Option<String>,
    weekdays: Option<Vec<i16>>,
    category_id: Option<Uuid>,
}

#[derive(Deserialize, Default)]
pub struct UpdateBody {
    name: Option<String>,
    description: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    icon: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    color: Option<Option<String>>,
    #[serde(default, deserialize_with = "double_option")]
    unit: Option<Option<String>>,
    tracking: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    weekdays: Option<Option<Vec<i16>>>,
    #[serde(default, deserialize_with = "double_option")]
    category_id: Option<Option<Uuid>>,
    position: Option<f64>,
    archived: Option<bool>,
}

pub async fn list(
    service: UserService,
    Query(query): Query<ListQuery>,
) -> AppResult<Json<List<HabitResponse>>> {
    Ok(Json(
        service.habits(query.archived).await?.into_iter().collect(),
    ))
}

pub async fn show(service: UserService, Path(id): Path<Uuid>) -> AppResult<Json<HabitResponse>> {
    Ok(Json(service.habit(id).await?.into()))
}

pub async fn create(
    service: UserService,
    Json(body): Json<CreateBody>,
) -> AppResult<(StatusCode, Json<HabitResponse>)> {
    let habit = service
        .create_habit(CreateHabit {
            name: body.name,
            description: body.description,
            icon: body.icon,
            color: body.color,
            unit: body.unit,
            tracking: body.tracking,
            weekdays: body.weekdays,
            category_id: body.category_id,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(habit.into())))
}

pub async fn update(
    service: UserService,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateBody>,
) -> AppResult<Json<HabitResponse>> {
    let habit = service
        .update_habit(
            id,
            UpdateHabit {
                name: body.name,
                description: body.description,
                icon: body.icon,
                color: body.color,
                unit: body.unit,
                tracking: body.tracking,
                weekdays: body.weekdays,
                category_id: body.category_id,
                position: body.position,
                archived: body.archived,
            },
        )
        .await?;
    Ok(Json(habit.into()))
}

pub async fn remove(service: UserService, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    service.delete_habit(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
