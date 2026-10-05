use uuid::Uuid;

use super::auth::{Session, SignInError, open_session};
use super::{Service, UserService};
use crate::config::Registration;
use crate::domain::{OAuthProvider, User, UserStatus};
use crate::error::{AppError, AppResult};
use crate::password::{MAX_LENGTH, MIN_LENGTH, acceptable, normalize_email};
use crate::repo::auth::{self as repo, NewIdentity, NewUser, PASSWORD};

/// What the sign-in page should offer on this instance.
pub struct Methods {
    pub providers: Vec<OAuthProvider>,
    pub registration: Registration,
}

pub enum Registered {
    SignedIn(Session),
    Pending,
}

pub struct Account {
    pub user: User,
    pub logins: Vec<String>,
}

impl Service {
    pub fn methods(&self) -> Methods {
        Methods {
            providers: self
                .auth
                .google
                .as_ref()
                .map(|_| OAuthProvider::Google)
                .into_iter()
                .collect(),
            registration: self.auth.registration,
        }
    }

    pub async fn register(
        &self,
        email: &str,
        password: String,
        name: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<Registered> {
        let status = match self.auth.registration {
            Registration::Closed => return Err(SignInError::RegistrationClosed.into()),
            Registration::Approval => UserStatus::Pending,
            Registration::Open => UserStatus::Active,
        };
        let email = normalize_email(email)
            .ok_or_else(|| AppError::invalid("email", "is not an email address"))?;
        require_length("password", &password)?;

        if repo::password_taken(&mut self.conn().await?, &email).await? {
            return Err(AppError::Conflict {
                field: "email",
                message: "is already registered",
            });
        }
        let hash = self.passwords.hash(password).await?;

        let display_name = name
            .map(|name| name.trim().to_owned())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| email.split('@').next().unwrap_or_default().to_owned());
        let id = Uuid::new_v4();
        let mut conn = self.conn().await?;
        let user = repo::create_user(
            &mut conn,
            NewUser {
                id,
                email: email.clone(),
                display_name,
                avatar_url: None,
                status: status.as_str().to_owned(),
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

        match status {
            UserStatus::Active => Ok(Registered::SignedIn(
                open_session(&mut conn, user, user_agent).await?,
            )),
            UserStatus::Pending => Ok(Registered::Pending),
        }
    }

    /// The database connection goes back to the pool before hashing starts: 12 ms of
    /// CPU per attempt must not hold a slot that habit requests are waiting for.
    pub async fn password_sign_in(
        &self,
        email: &str,
        password: String,
        user_agent: Option<String>,
    ) -> AppResult<Session> {
        let Some(email) = normalize_email(email) else {
            return Err(AppError::InvalidCredentials);
        };
        if self.attempts.blocked(&email) {
            return Err(AppError::TooManyAttempts);
        }

        let found = repo::password_login(&mut self.conn().await?, &email).await?;
        let (user, stored) = found.unzip();
        let matches = self.passwords.verify(password, stored).await?;
        let Some(user) = user.filter(|_| matches) else {
            self.attempts.failed(&email);
            return Err(AppError::InvalidCredentials);
        };

        self.attempts.succeeded(&email);
        if user.status != UserStatus::Active {
            return Err(SignInError::PendingApproval.into());
        }
        open_session(&mut self.conn().await?, user, user_agent).await
    }
}

impl UserService {
    pub async fn account(&self) -> AppResult<Account> {
        let mut conn = self.conn().await?;
        Ok(Account {
            user: repo::user(&mut conn, self.user_id).await?,
            logins: repo::logins(&mut conn, self.user_id).await?,
        })
    }

    /// Sets a first password on an account that only had a provider, or replaces the
    /// current one - which then has to be proven. Every session ends and the caller
    /// gets a fresh one, so a device that stole the old session is out.
    pub async fn change_password(
        &self,
        current: Option<String>,
        new: String,
        user_agent: Option<String>,
    ) -> AppResult<Session> {
        require_length("new_password", &new)?;
        let (user, stored) = {
            let mut conn = self.conn().await?;
            (
                repo::user(&mut conn, self.user_id).await?,
                repo::password_of(&mut conn, self.user_id).await?,
            )
        };
        let email = normalize_email(&user.email)
            .ok_or_else(|| AppError::invalid("email", "the account has no usable email"))?;

        if let Some(stored) = stored {
            let attempts = &self.service.attempts;
            if attempts.blocked(&email) {
                return Err(AppError::TooManyAttempts);
            }
            let current =
                current.ok_or_else(|| AppError::invalid("current_password", "is required"))?;
            if !self.service.passwords.verify(current, Some(stored)).await? {
                attempts.failed(&email);
                return Err(AppError::invalid("current_password", "is incorrect"));
            }
            attempts.succeeded(&email);
        }

        let hash = self.service.passwords.hash(new).await?;
        let mut conn = self.conn().await?;
        repo::set_password(&mut conn, user.id, &email, &hash).await?;
        repo::delete_sessions(&mut conn, user.id).await?;
        open_session(&mut conn, user, user_agent).await
    }
}

fn require_length(field: &'static str, password: &str) -> AppResult<()> {
    if acceptable(password) {
        return Ok(());
    }
    Err(AppError::Invalid {
        field: Some(field.to_owned()),
        message: format!("must be {MIN_LENGTH} to {MAX_LENGTH} characters"),
    })
}
