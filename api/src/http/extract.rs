use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::AppError;

/// Axum's own extractors reject with plain text; these keep every 4xx inside the
/// {"error": ...} envelope clients rely on.
pub struct Path<T>(pub T);
pub struct Query<T>(pub T);
pub struct Json<T>(pub T);

impl From<PathRejection> for AppError {
    fn from(err: PathRejection) -> Self {
        Self::invalid("path", err.body_text())
    }
}

impl From<QueryRejection> for AppError {
    fn from(err: QueryRejection) -> Self {
        Self::invalid("query", err.body_text())
    }
}

impl From<JsonRejection> for AppError {
    fn from(err: JsonRejection) -> Self {
        Self::invalid("body", err.body_text())
    }
}

impl<T, S> FromRequestParts<S> for Path<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let inner = axum::extract::Path::<T>::from_request_parts(parts, state).await?;
        Ok(Self(inner.0))
    }
}

impl<T, S> FromRequestParts<S> for Query<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let inner = axum::extract::Query::<T>::from_request_parts(parts, state).await?;
        Ok(Self(inner.0))
    }
}

impl<T, S> FromRequest<S> for Json<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let inner = axum::Json::<T>::from_request(req, state).await?;
        Ok(Self(inner.0))
    }
}

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}
