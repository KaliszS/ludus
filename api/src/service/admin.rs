//! Operations only the command line can reach. Running them needs a shell on the
//! server and its database credentials, so they grant nothing that was not already
//! held - which is why none of them is exposed over HTTP.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use uuid::Uuid;

use super::Service;
use crate::domain::{User, UserStatus};
use crate::error::{AppError, AppResult};
use crate::password::normalize_email;
use crate::repo::auth::{self as repo, NewIdentity, NewUser, PASSWORD};

impl Service {
    pub async fn admin_users(&self) -> AppResult<Vec<(User, Vec<String>)>> {
        let mut conn = self.conn().await?;
        let mut listed = Vec::new();
        for user in repo::users(&mut conn).await? {
            let logins = repo::logins(&mut conn, user.id).await?;
            listed.push((user, logins));
        }
        Ok(listed)
    }

    /// Creates an active account that signs in with the returned password.
    pub async fn admin_add(&self, email: &str, name: Option<String>) -> AppResult<(User, String)> {
        let email = normalize_email(email)
            .ok_or_else(|| AppError::invalid("email", "is not an email address"))?;
        let password = generated_password();
        let hash = self.passwords.hash(password.clone()).await?;
        let id = Uuid::new_v4();
        let user = repo::create_user(
            &mut self.conn().await?,
            NewUser {
                id,
                display_name: name
                    .unwrap_or_else(|| email.split('@').next().unwrap_or_default().to_owned()),
                email: email.clone(),
                avatar_url: None,
                status: UserStatus::Active.as_str().to_owned(),
            },
            NewIdentity {
                id: Uuid::new_v4(),
                user_id: id,
                provider: PASSWORD.to_owned(),
                provider_id: email.clone(),
                email,
                password_hash: Some(hash),
            },
        )
        .await?;
        Ok((user, password))
    }

    /// The only way back in without email: an admin issues a new password, and
    /// every session the account had ends.
    pub async fn admin_reset_password(&self, key: &str) -> AppResult<(User, String)> {
        let user = self.admin_find(key).await?;
        let email = normalize_email(&user.email)
            .ok_or_else(|| AppError::invalid("email", "the account has no usable email"))?;
        let password = generated_password();
        let hash = self.passwords.hash(password.clone()).await?;
        let mut conn = self.conn().await?;
        repo::set_password(&mut conn, user.id, &email, &hash).await?;
        repo::delete_sessions(&mut conn, user.id).await?;
        Ok((user, password))
    }

    pub async fn admin_activate(&self, key: &str) -> AppResult<User> {
        let user = self.admin_find(key).await?;
        repo::set_status(&mut self.conn().await?, user.id, UserStatus::Active).await?;
        Ok(User {
            status: UserStatus::Active,
            ..user
        })
    }

    /// By id, or by email when exactly one account carries it: a Google account and
    /// a password account may share an address, and guessing between them is not ours.
    async fn admin_find(&self, key: &str) -> AppResult<User> {
        let mut conn = self.conn().await?;
        if let Ok(id) = Uuid::parse_str(key) {
            return repo::user(&mut conn, id).await;
        }
        let email = normalize_email(key).ok_or(AppError::NotFound)?;
        let mut found = repo::users_by_email(&mut conn, &email).await?;
        match found.len() {
            0 => Err(AppError::NotFound),
            1 => Ok(found.remove(0)),
            _ => Err(AppError::invalid(
                "user",
                "several accounts use this email; pass the id instead",
            )),
        }
    }
}

/// 128 random bits as 22 characters: strong, and short enough to type once.
fn generated_password() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("the OS random generator is available");
    URL_SAFE_NO_PAD.encode(bytes)
}
