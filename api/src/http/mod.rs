mod checkin;
mod dto;
mod extract;
mod habit;
mod plan;

use axum::Router;
use axum::http::{HeaderValue, Method, header};
use axum::routing::{get, patch, put};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::service::Service;

pub const PREFIX: &str = "/v1";

pub fn router(service: Service, allowed_origins: &[String]) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .nest(PREFIX, api())
        .layer(cors(allowed_origins))
        .layer(TraceLayer::new_for_http())
        .with_state(service)
}

/// Clients are served from a different origin than the daemon, so without this
/// the browser refuses every call.
fn cors(allowed_origins: &[String]) -> CorsLayer {
    let origins: Vec<HeaderValue> = allowed_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::PUT,
            Method::DELETE,
        ])
        .allow_headers([header::CONTENT_TYPE])
}

fn api() -> Router<Service> {
    Router::new()
        .route("/habits", get(habit::list).post(habit::create))
        .route(
            "/habits/{id}",
            get(habit::show).patch(habit::update).delete(habit::remove),
        )
        .route("/checkins", get(checkin::list_all))
        .route("/habits/{id}/checkins", get(checkin::list))
        .route(
            "/habits/{id}/checkins/{day}",
            put(checkin::set).delete(checkin::remove),
        )
        .route(
            "/plan-levels",
            get(plan::list_levels).post(plan::create_level),
        )
        .route(
            "/plan-levels/{id}",
            get(plan::show_level)
                .patch(plan::update_level)
                .delete(plan::remove_level),
        )
        .route(
            "/plan-levels/{id}/requirements",
            get(plan::list_requirements).post(plan::create_requirement),
        )
        .route(
            "/plan-requirements/{id}",
            patch(plan::update_requirement).delete(plan::remove_requirement),
        )
        .route("/plan", get(plan::progress))
        .route("/plan/history", get(plan::history))
}
