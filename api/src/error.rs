use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{message}")]
    Invalid {
        field: Option<String>,
        message: String,
    },
    #[error("not found")]
    NotFound,
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl AppError {
    pub fn invalid(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Invalid {
            field: Some(field.into()),
            message: message.into(),
        }
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(err: diesel::result::Error) -> Self {
        match err {
            diesel::result::Error::NotFound => Self::NotFound,
            other => Self::Internal(other.into()),
        }
    }
}

impl From<diesel_async::pooled_connection::deadpool::PoolError> for AppError {
    fn from(err: diesel_async::pooled_connection::deadpool::PoolError) -> Self {
        Self::Internal(err.into())
    }
}

#[derive(Serialize)]
struct Body {
    error: Detail,
}

#[derive(Serialize)]
struct Detail {
    code: &'static str,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, field) = match &self {
            Self::Invalid { field, .. } => {
                (StatusCode::BAD_REQUEST, "invalid_request", field.clone())
            }
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found", None),
            Self::Internal(err) => {
                tracing::error!(error = ?err, "request failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal", None)
            }
        };

        let message = match &self {
            Self::Internal(_) => "internal error".to_owned(),
            other => other.to_string(),
        };

        let body = Body {
            error: Detail {
                code,
                message,
                field,
            },
        };
        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
