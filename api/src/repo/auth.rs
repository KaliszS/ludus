use chrono::{DateTime, Utc};
use diesel::dsl::now;
use diesel::prelude::*;
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use super::pool::DbConn;
use super::schema::{oauth_states, sessions, user_identities, users};
use crate::domain::{User, UserStatus};
use crate::error::{AppError, AppResult};

pub const PASSWORD: &str = "password";

#[derive(Queryable, Selectable)]
#[diesel(table_name = users, check_for_backend(diesel::pg::Pg))]
struct UserRow {
    id: Uuid,
    email: String,
    display_name: String,
    avatar_url: Option<String>,
    status: String,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: row.id,
            email: row.email,
            display_name: row.display_name,
            avatar_url: row.avatar_url,
            status: UserStatus::parse(&row.status),
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub status: String,
}

#[derive(Insertable)]
#[diesel(table_name = user_identities)]
pub struct NewIdentity {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_id: String,
    pub email: String,
    pub password_hash: Option<String>,
}

pub async fn user(conn: &mut DbConn, id: Uuid) -> AppResult<User> {
    let row = users::table
        .find(id)
        .select(UserRow::as_select())
        .first(conn)
        .await?;
    Ok(row.into())
}

pub async fn user_by_identity(
    conn: &mut DbConn,
    provider: &str,
    subject: &str,
) -> AppResult<Option<User>> {
    let row = user_identities::table
        .inner_join(users::table)
        .filter(user_identities::provider.eq(provider))
        .filter(user_identities::provider_id.eq(subject))
        .select(UserRow::as_select())
        .first(conn)
        .await
        .optional()?;
    Ok(row.map(User::from))
}

/// The account and the login that owns it land together or not at all.
pub async fn create_user(
    conn: &mut DbConn,
    user: NewUser,
    identity: NewIdentity,
) -> AppResult<User> {
    let row = conn
        .transaction(async |conn| {
            let row = diesel::insert_into(users::table)
                .values(&user)
                .returning(UserRow::as_returning())
                .get_result(conn)
                .await?;
            diesel::insert_into(user_identities::table)
                .values(&identity)
                .execute(conn)
                .await?;
            Ok::<_, diesel::result::Error>(row)
        })
        .await
        .map_err(email_taken)?;
    Ok(row.into())
}

/// The unique index is the real guard: two sign-ups racing for one address both
/// pass any earlier check, and only one of them can win the insert.
fn email_taken(err: diesel::result::Error) -> AppError {
    match err {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::UniqueViolation,
            _,
        ) => AppError::Conflict {
            field: "email",
            message: "is already registered",
        },
        other => other.into(),
    }
}

/// The account behind a password login, with the hash to check against.
pub async fn password_login(conn: &mut DbConn, email: &str) -> AppResult<Option<(User, String)>> {
    let row: Option<(UserRow, Option<String>)> = user_identities::table
        .inner_join(users::table)
        .filter(user_identities::provider.eq(PASSWORD))
        .filter(user_identities::provider_id.eq(email))
        .select((UserRow::as_select(), user_identities::password_hash))
        .first(conn)
        .await
        .optional()?;
    Ok(row.and_then(|(user, hash)| hash.map(|hash| (user.into(), hash))))
}

pub async fn password_taken(conn: &mut DbConn, email: &str) -> AppResult<bool> {
    Ok(diesel::select(diesel::dsl::exists(
        user_identities::table
            .filter(user_identities::provider.eq(PASSWORD))
            .filter(user_identities::provider_id.eq(email)),
    ))
    .get_result(conn)
    .await?)
}

pub async fn password_of(conn: &mut DbConn, user_id: Uuid) -> AppResult<Option<String>> {
    Ok(user_identities::table
        .filter(user_identities::user_id.eq(user_id))
        .filter(user_identities::provider.eq(PASSWORD))
        .select(user_identities::password_hash)
        .first::<Option<String>>(conn)
        .await
        .optional()?
        .flatten())
}

/// Replaces the account's password, or gives it one if it signed in some other way
/// until now. That second case claims `email` as a password login.
pub async fn set_password(
    conn: &mut DbConn,
    user_id: Uuid,
    email: &str,
    hash: &str,
) -> AppResult<()> {
    let updated = diesel::update(
        user_identities::table
            .filter(user_identities::user_id.eq(user_id))
            .filter(user_identities::provider.eq(PASSWORD)),
    )
    .set(user_identities::password_hash.eq(hash))
    .execute(conn)
    .await?;
    if updated == 0 {
        diesel::insert_into(user_identities::table)
            .values(NewIdentity {
                id: Uuid::new_v4(),
                user_id,
                provider: PASSWORD.to_owned(),
                provider_id: email.to_owned(),
                email: email.to_owned(),
                password_hash: Some(hash.to_owned()),
            })
            .execute(conn)
            .await
            .map_err(email_taken)?;
    }
    Ok(())
}

/// How this account can sign in: provider names, `password` among them.
pub async fn logins(conn: &mut DbConn, user_id: Uuid) -> AppResult<Vec<String>> {
    Ok(user_identities::table
        .filter(user_identities::user_id.eq(user_id))
        .select(user_identities::provider)
        .order(user_identities::provider)
        .load(conn)
        .await?)
}

pub async fn users(conn: &mut DbConn) -> AppResult<Vec<User>> {
    let rows = users::table
        .order(users::created_at)
        .select(UserRow::as_select())
        .load(conn)
        .await?;
    Ok(rows.into_iter().map(User::from).collect())
}

pub async fn users_by_email(conn: &mut DbConn, email: &str) -> AppResult<Vec<User>> {
    let rows = users::table
        .filter(users::email.eq(email))
        .select(UserRow::as_select())
        .load(conn)
        .await?;
    Ok(rows.into_iter().map(User::from).collect())
}

pub async fn set_status(conn: &mut DbConn, user_id: Uuid, status: UserStatus) -> AppResult<()> {
    diesel::update(users::table.find(user_id))
        .set(users::status.eq(status.as_str()))
        .execute(conn)
        .await?;
    Ok(())
}

/// After a password change every existing session ends, including on stolen devices.
pub async fn delete_sessions(conn: &mut DbConn, user_id: Uuid) -> AppResult<()> {
    diesel::delete(sessions::table.filter(sessions::user_id.eq(user_id)))
        .execute(conn)
        .await?;
    Ok(())
}

#[derive(Insertable)]
#[diesel(table_name = oauth_states)]
pub struct NewState {
    pub state: String,
    pub provider: String,
    pub pkce_verifier: String,
    pub client_challenge: String,
    pub redirect_uri: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = oauth_states, check_for_backend(diesel::pg::Pg))]
pub struct PendingState {
    pub pkce_verifier: String,
    pub redirect_uri: String,
}

pub async fn insert_state(conn: &mut DbConn, state: NewState) -> AppResult<()> {
    diesel::insert_into(oauth_states::table)
        .values(state)
        .execute(conn)
        .await?;
    Ok(())
}

/// A state still waiting for the provider: live, for this provider, not yet resolved.
pub async fn pending_state(
    conn: &mut DbConn,
    state: &str,
    provider: &str,
) -> AppResult<Option<PendingState>> {
    Ok(oauth_states::table
        .find(state)
        .filter(oauth_states::provider.eq(provider))
        .filter(oauth_states::expires_at.gt(now))
        .filter(oauth_states::user_id.is_null())
        .select(PendingState::as_select())
        .first(conn)
        .await
        .optional()?)
}

pub async fn resolve_state(
    conn: &mut DbConn,
    state: &str,
    user_id: Uuid,
    code_hash: &[u8],
    expires_at: DateTime<Utc>,
) -> AppResult<()> {
    diesel::update(oauth_states::table.find(state))
        .set((
            oauth_states::user_id.eq(user_id),
            oauth_states::code_hash.eq(code_hash),
            oauth_states::expires_at.eq(expires_at),
        ))
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn delete_state(conn: &mut DbConn, state: &str) -> AppResult<()> {
    diesel::delete(oauth_states::table.find(state))
        .execute(conn)
        .await?;
    Ok(())
}

/// Reads and burns a sign-in code in one statement, so two concurrent requests
/// cannot both redeem it. A wrong verifier still costs the code.
pub async fn take_code(
    conn: &mut DbConn,
    code_hash: &[u8],
) -> AppResult<Option<(Uuid, Option<String>)>> {
    let row: Option<(Option<Uuid>, Option<String>)> = diesel::delete(
        oauth_states::table
            .filter(oauth_states::code_hash.eq(code_hash))
            .filter(oauth_states::expires_at.gt(now)),
    )
    .returning((oauth_states::user_id, oauth_states::client_challenge))
    .get_result(conn)
    .await
    .optional()?;
    Ok(row.and_then(|(user_id, challenge)| user_id.map(|id| (id, challenge))))
}

#[derive(Insertable)]
#[diesel(table_name = sessions)]
pub struct NewSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: Vec<u8>,
    pub client: String,
    pub user_agent: Option<String>,
    pub expires_at: DateTime<Utc>,
}

pub async fn insert_session(conn: &mut DbConn, session: NewSession) -> AppResult<()> {
    diesel::insert_into(sessions::table)
        .values(session)
        .execute(conn)
        .await?;
    Ok(())
}

/// The owner of a live session, provided that owner is still allowed in.
pub async fn session_user(
    conn: &mut DbConn,
    token_hash: &[u8],
) -> AppResult<Option<(Uuid, DateTime<Utc>)>> {
    Ok(sessions::table
        .inner_join(users::table)
        .filter(sessions::token_hash.eq(token_hash))
        .filter(sessions::expires_at.gt(now))
        .filter(users::status.eq(UserStatus::Active.as_str()))
        .select((sessions::user_id, sessions::last_used_at))
        .first(conn)
        .await
        .optional()?)
}

pub async fn touch_session(
    conn: &mut DbConn,
    token_hash: &[u8],
    expires_at: DateTime<Utc>,
) -> AppResult<()> {
    diesel::update(sessions::table.filter(sessions::token_hash.eq(token_hash)))
        .set((
            sessions::last_used_at.eq(now),
            sessions::expires_at.eq(expires_at),
        ))
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn delete_session(conn: &mut DbConn, token_hash: &[u8]) -> AppResult<()> {
    diesel::delete(sessions::table.filter(sessions::token_hash.eq(token_hash)))
        .execute(conn)
        .await?;
    Ok(())
}

/// Both tables only ever grow otherwise; sign-in is rare enough to pay for the sweep.
pub async fn purge_expired(conn: &mut DbConn) -> AppResult<()> {
    diesel::delete(oauth_states::table.filter(oauth_states::expires_at.le(now)))
        .execute(conn)
        .await?;
    diesel::delete(sessions::table.filter(sessions::expires_at.le(now)))
        .execute(conn)
        .await?;
    Ok(())
}
