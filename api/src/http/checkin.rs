use axum::body::Bytes;
use axum::extract::State;
use axum::http::StatusCode;
use chrono::{Days, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::dto::List;
use super::extract::{Json, Path, Query};
use crate::domain::Checkin;
use crate::error::{AppError, AppResult};
use crate::service::Service;

const DEFAULT_WINDOW_DAYS: u64 = 365;

#[derive(Serialize)]
pub struct CheckinResponse {
    day: NaiveDate,
    times: i32,
}

impl From<Checkin> for CheckinResponse {
    fn from(checkin: Checkin) -> Self {
        Self {
            day: checkin.day,
            times: checkin.times,
        }
    }
}

#[derive(Serialize)]
pub struct UserCheckinResponse {
    habit_id: Uuid,
    day: NaiveDate,
    times: i32,
}

#[derive(Deserialize)]
pub struct RangeQuery {
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
}

#[derive(Deserialize, Default)]
pub struct SetBody {
    times: Option<i32>,
}

pub async fn list(
    State(service): State<Service>,
    Path(habit_id): Path<Uuid>,
    Query(query): Query<RangeQuery>,
) -> AppResult<Json<List<CheckinResponse>>> {
    let to = query.to.unwrap_or_else(|| Utc::now().date_naive());
    let from = query.from.unwrap_or(to - Days::new(DEFAULT_WINDOW_DAYS));
    Ok(Json(
        service
            .checkins(habit_id, from, to)
            .await?
            .into_iter()
            .collect(),
    ))
}

/// The plain click sends no body at all, so an empty one is not an error.
pub async fn list_all(
    State(service): State<Service>,
    Query(query): Query<RangeQuery>,
) -> AppResult<Json<List<UserCheckinResponse>>> {
    let to = query.to.unwrap_or_else(|| Utc::now().date_naive());
    let from = query.from.unwrap_or(to - Days::new(DEFAULT_WINDOW_DAYS));
    Ok(Json(
        service
            .all_checkins(from, to)
            .await?
            .into_iter()
            .map(|(habit_id, day, times)| UserCheckinResponse {
                habit_id,
                day,
                times,
            })
            .collect(),
    ))
}

pub async fn set(
    State(service): State<Service>,
    Path((habit_id, day)): Path<(Uuid, NaiveDate)>,
    body: Bytes,
) -> AppResult<Json<CheckinResponse>> {
    let times = if body.is_empty() {
        None
    } else {
        serde_json::from_slice::<SetBody>(&body)
            .map_err(|_| AppError::invalid("body", "must be a JSON object"))?
            .times
    };
    Ok(Json(
        service.set_checkin(habit_id, day, times).await?.into(),
    ))
}

pub async fn remove(
    State(service): State<Service>,
    Path((habit_id, day)): Path<(Uuid, NaiveDate)>,
) -> AppResult<StatusCode> {
    service.unset_checkin(habit_id, day).await?;
    Ok(StatusCode::NO_CONTENT)
}
