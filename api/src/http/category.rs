use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::dto::{List, double_option};
use super::extract::{Json, Path};
use crate::domain::HabitCategory;
use crate::error::AppResult;
use crate::service::UserService;
use crate::service::category::{CreateCategory, UpdateCategory};

#[derive(Serialize)]
pub struct CategoryResponse {
    id: Uuid,
    parent_id: Option<Uuid>,
    name: String,
    position: f64,
}

impl From<HabitCategory> for CategoryResponse {
    fn from(category: HabitCategory) -> Self {
        Self {
            id: category.id,
            parent_id: category.parent_id,
            name: category.name,
            position: category.position,
        }
    }
}

#[derive(Deserialize)]
pub struct CreateBody {
    name: String,
    parent_id: Option<Uuid>,
}

#[derive(Deserialize, Default)]
pub struct UpdateBody {
    name: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    parent_id: Option<Option<Uuid>>,
    position: Option<f64>,
}

pub async fn list(service: UserService) -> AppResult<Json<List<CategoryResponse>>> {
    Ok(Json(service.categories().await?.into_iter().collect()))
}

pub async fn create(
    service: UserService,
    Json(body): Json<CreateBody>,
) -> AppResult<(StatusCode, Json<CategoryResponse>)> {
    let category = service
        .create_category(CreateCategory {
            name: body.name,
            parent_id: body.parent_id,
        })
        .await?;
    Ok((StatusCode::CREATED, Json(category.into())))
}

pub async fn update(
    service: UserService,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateBody>,
) -> AppResult<Json<CategoryResponse>> {
    let category = service
        .update_category(
            id,
            UpdateCategory {
                name: body.name,
                parent_id: body.parent_id,
                position: body.position,
            },
        )
        .await?;
    Ok(Json(category.into()))
}

pub async fn remove(service: UserService, Path(id): Path<Uuid>) -> AppResult<StatusCode> {
    service.delete_category(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
