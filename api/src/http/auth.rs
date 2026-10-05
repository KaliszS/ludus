use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::extract::{Json, Path, Query};
use crate::domain::{OAuthProvider, User};
use crate::error::{AppError, AppResult};
use crate::service::auth::Session;
use crate::service::password::Registered;
use crate::service::{Service, UserService};

/// Taking a `UserService` in a handler is what makes the route require sign-in.
impl FromRequestParts<Service> for UserService {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, service: &Service) -> AppResult<Self> {
        let token = bearer(&parts.headers).ok_or(AppError::Unauthenticated)?;
        service.authenticate(token).await
    }
}

fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|token| !token.is_empty())
}

fn provider(name: &str) -> AppResult<OAuthProvider> {
    OAuthProvider::parse(name).ok_or(AppError::NotFound)
}

#[derive(Serialize)]
pub struct UserResponse {
    id: Uuid,
    email: String,
    display_name: String,
    avatar_url: Option<String>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
        }
    }
}

#[derive(Deserialize)]
pub struct StartQuery {
    redirect_uri: String,
    code_challenge: String,
}

pub async fn start(
    State(service): State<Service>,
    Path(name): Path<String>,
    Query(query): Query<StartQuery>,
) -> AppResult<Redirect> {
    let url = service
        .begin_sign_in(provider(&name)?, query.redirect_uri, query.code_challenge)
        .await?;
    Ok(Redirect::to(url.as_str()))
}

/// A provider that is cancelled sends `error` instead of `code`; the missing code
/// is all the service needs to know.
#[derive(Deserialize)]
pub struct CallbackQuery {
    state: String,
    code: Option<String>,
}

pub async fn callback(
    State(service): State<Service>,
    Path(name): Path<String>,
    Query(query): Query<CallbackQuery>,
) -> AppResult<Redirect> {
    let url = service
        .finish_sign_in(provider(&name)?, &query.state, query.code.as_deref())
        .await?;
    Ok(Redirect::to(url.as_str()))
}

#[derive(Deserialize)]
pub struct TokenBody {
    code: String,
    code_verifier: String,
}

#[derive(Serialize)]
pub struct SessionResponse {
    token: String,
    expires_at: DateTime<Utc>,
    user: UserResponse,
}

impl From<Session> for SessionResponse {
    fn from(session: Session) -> Self {
        Self {
            token: session.token,
            expires_at: session.expires_at,
            user: session.user.into(),
        }
    }
}

fn user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

pub async fn token(
    State(service): State<Service>,
    headers: HeaderMap,
    Json(body): Json<TokenBody>,
) -> AppResult<Json<SessionResponse>> {
    let session = service
        .redeem(&body.code, &body.code_verifier, user_agent(&headers))
        .await?;
    Ok(Json(session.into()))
}

#[derive(Serialize)]
pub struct MethodsResponse {
    providers: Vec<&'static str>,
    registration: &'static str,
}

pub async fn methods(State(service): State<Service>) -> Json<MethodsResponse> {
    let methods = service.methods();
    Json(MethodsResponse {
        providers: methods
            .providers
            .into_iter()
            .map(OAuthProvider::as_str)
            .collect(),
        registration: methods.registration.as_str(),
    })
}

#[derive(Deserialize)]
pub struct PasswordSignIn {
    email: String,
    password: String,
}

pub async fn password_login(
    State(service): State<Service>,
    headers: HeaderMap,
    Json(body): Json<PasswordSignIn>,
) -> AppResult<Json<SessionResponse>> {
    let session = service
        .password_sign_in(&body.email, body.password, user_agent(&headers))
        .await?;
    Ok(Json(session.into()))
}

#[derive(Deserialize)]
pub struct RegisterBody {
    email: String,
    password: String,
    display_name: Option<String>,
}

#[derive(Serialize)]
struct PendingResponse {
    pending: bool,
}

/// 201 with a session when the account is live at once, 202 when it waits for approval.
pub async fn register(
    State(service): State<Service>,
    headers: HeaderMap,
    Json(body): Json<RegisterBody>,
) -> AppResult<Response> {
    let registered = service
        .register(
            &body.email,
            body.password,
            body.display_name,
            user_agent(&headers),
        )
        .await?;
    Ok(match registered {
        Registered::SignedIn(session) => {
            (StatusCode::CREATED, Json(SessionResponse::from(session))).into_response()
        }
        Registered::Pending => (
            StatusCode::ACCEPTED,
            Json(PendingResponse { pending: true }),
        )
            .into_response(),
    })
}

#[derive(Deserialize)]
pub struct PasswordChange {
    current_password: Option<String>,
    new_password: String,
}

pub async fn change_password(
    service: UserService,
    headers: HeaderMap,
    Json(body): Json<PasswordChange>,
) -> AppResult<Json<SessionResponse>> {
    let session = service
        .change_password(
            body.current_password,
            body.new_password,
            user_agent(&headers),
        )
        .await?;
    Ok(Json(session.into()))
}

/// Succeeds even without a valid session: the client wants to be signed out,
/// and it is.
pub async fn logout(State(service): State<Service>, headers: HeaderMap) -> AppResult<StatusCode> {
    if let Some(token) = bearer(&headers) {
        service.sign_out(token).await?;
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct AccountResponse {
    #[serde(flatten)]
    user: UserResponse,
    /// How this account can sign in, e.g. `["google", "password"]`.
    logins: Vec<String>,
}

pub async fn me(service: UserService) -> AppResult<Json<AccountResponse>> {
    let account = service.account().await?;
    Ok(Json(AccountResponse {
        user: account.user.into(),
        logins: account.logins,
    }))
}
