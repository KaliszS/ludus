use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{DateTime, Duration, Utc};
use reqwest::Url;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::{Service, UserService};
use crate::config::{OAuthClient, Registration};
use crate::domain::{Provider, User, UserStatus};
use crate::error::{AppError, AppResult};
use crate::oauth;
use crate::repo::auth::{self as repo, NewIdentity, NewSession, NewState, NewUser};

/// How long a user has to finish signing in at the provider.
const STATE_TTL: Duration = Duration::minutes(10);
/// How long the client has to trade the one-time code for a session.
const CODE_TTL: Duration = Duration::minutes(2);
/// Sliding: every use pushes the expiry out again, so only an idle session dies.
const SESSION_TTL: Duration = Duration::days(30);
/// Recording every request would turn each read into a write.
const TOUCH_EVERY: Duration = Duration::hours(1);

/// Why a sign-in ended back at the client without a code. The client shows these.
#[derive(Debug, Clone, Copy)]
pub enum SignInError {
    Cancelled,
    RegistrationClosed,
    PendingApproval,
    Failed,
}

impl SignInError {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cancelled => "cancelled",
            Self::RegistrationClosed => "registration_closed",
            Self::PendingApproval => "pending_approval",
            Self::Failed => "sign_in_failed",
        }
    }
}

pub struct Session {
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub user: User,
}

impl Service {
    /// First leg: remember what the client asked for, send the browser to the provider.
    pub async fn begin_sign_in(
        &self,
        provider: Provider,
        redirect_uri: String,
        client_challenge: String,
    ) -> AppResult<Url> {
        let client = self.client(provider)?;
        if !self.auth.redirect_uris.contains(&redirect_uri) {
            return Err(AppError::invalid(
                "redirect_uri",
                "is not an allowed redirect",
            ));
        }
        if !is_challenge(&client_challenge) {
            return Err(AppError::invalid(
                "code_challenge",
                "must be a base64url SHA-256 digest",
            ));
        }

        let state = random_token();
        let verifier = random_token();
        let mut conn = self.conn().await?;
        repo::purge_expired(&mut conn).await?;
        repo::insert_state(
            &mut conn,
            NewState {
                state: state.clone(),
                provider: provider.as_str().to_owned(),
                pkce_verifier: verifier.clone(),
                client_challenge,
                redirect_uri,
                expires_at: Utc::now() + STATE_TTL,
            },
        )
        .await?;

        Ok(oauth::authorize_url(
            provider,
            client,
            &self.callback_uri(provider),
            &state,
            &challenge_of(&verifier),
        ))
    }

    /// Second leg: the provider is back. Whatever happens, the browser goes to the
    /// client it came from - with a code, or with the reason there is none.
    pub async fn finish_sign_in(
        &self,
        provider: Provider,
        state: &str,
        code: Option<&str>,
    ) -> AppResult<Url> {
        let client = self.client(provider)?;
        let mut conn = self.conn().await?;
        let pending = repo::pending_state(&mut conn, state, provider.as_str())
            .await?
            .ok_or_else(|| AppError::invalid("state", "is unknown or has expired"))?;
        let back = Url::parse(&pending.redirect_uri)
            .map_err(|_| AppError::invalid("redirect_uri", "is not a URL"))?;

        let outcome = match code {
            None => Err(SignInError::Cancelled),
            Some(code) => {
                match oauth::exchange(
                    &self.http,
                    provider,
                    client,
                    &self.callback_uri(provider),
                    code,
                    &pending.pkce_verifier,
                )
                .await
                {
                    Ok(profile) => self.admit(&mut conn, provider, profile).await?,
                    Err(err) => {
                        tracing::warn!(error = ?err, provider = provider.as_str(), "sign-in exchange failed");
                        Err(SignInError::Failed)
                    }
                }
            }
        };

        match outcome {
            Ok(user) => {
                let code = random_token();
                repo::resolve_state(
                    &mut conn,
                    state,
                    user.id,
                    &hash(&code),
                    Utc::now() + CODE_TTL,
                )
                .await?;
                Ok(with_query(back, "code", &code))
            }
            Err(reason) => {
                repo::delete_state(&mut conn, state).await?;
                Ok(with_query(back, "error", reason.as_str()))
            }
        }
    }

    /// The client proves it started this sign-in and gets a session in return.
    pub async fn redeem(
        &self,
        code: &str,
        verifier: &str,
        user_agent: Option<String>,
    ) -> AppResult<Session> {
        let rejected = || AppError::invalid("code", "is invalid or has expired");
        let mut conn = self.conn().await?;
        let (user_id, challenge) = repo::take_code(&mut conn, &hash(code))
            .await?
            .ok_or_else(rejected)?;
        if challenge.as_deref() != Some(challenge_of(verifier).as_str()) {
            return Err(rejected());
        }

        let token = random_token();
        let expires_at = Utc::now() + SESSION_TTL;
        repo::insert_session(
            &mut conn,
            NewSession {
                id: Uuid::new_v4(),
                user_id,
                token_hash: hash(&token),
                client: "web".to_owned(),
                user_agent,
                expires_at,
            },
        )
        .await?;

        Ok(Session {
            token,
            expires_at,
            user: repo::user(&mut conn, user_id).await?,
        })
    }

    pub async fn authenticate(&self, token: &str) -> AppResult<UserService> {
        let token_hash = hash(token);
        let mut conn = self.conn().await?;
        let (user_id, last_used_at) = repo::session_user(&mut conn, &token_hash)
            .await?
            .ok_or(AppError::Unauthenticated)?;

        let now = Utc::now();
        if now - last_used_at > TOUCH_EVERY {
            repo::touch_session(&mut conn, &token_hash, now + SESSION_TTL).await?;
        }
        Ok(self.for_user(user_id))
    }

    pub async fn sign_out(&self, token: &str) -> AppResult<()> {
        let mut conn = self.conn().await?;
        repo::delete_session(&mut conn, &hash(token)).await
    }

    fn client(&self, provider: Provider) -> AppResult<&OAuthClient> {
        match provider {
            Provider::Google => self.auth.google.as_ref(),
        }
        .ok_or(AppError::NotFound)
    }

    fn callback_uri(&self, provider: Provider) -> String {
        format!(
            "{}/v1/auth/{}/callback",
            self.auth.public_url,
            provider.as_str()
        )
    }

    /// Recognises a returning user by the provider's subject, never by email: a
    /// second provider vouching for the same address is not proof of the same person.
    async fn admit(
        &self,
        conn: &mut crate::repo::pool::DbConn,
        provider: Provider,
        profile: oauth::Profile,
    ) -> AppResult<Result<User, SignInError>> {
        let user = match repo::user_by_identity(conn, provider.as_str(), &profile.subject).await? {
            Some(user) => user,
            None => {
                let status = match self.auth.registration {
                    Registration::Closed => return Ok(Err(SignInError::RegistrationClosed)),
                    Registration::Approval => UserStatus::Pending,
                    Registration::Open => UserStatus::Active,
                };
                let id = Uuid::new_v4();
                repo::create_user(
                    conn,
                    NewUser {
                        id,
                        email: profile.email.clone(),
                        display_name: profile.name,
                        avatar_url: profile.avatar_url,
                        status: status.as_str().to_owned(),
                    },
                    NewIdentity {
                        id: Uuid::new_v4(),
                        user_id: id,
                        provider: provider.as_str().to_owned(),
                        provider_id: profile.subject,
                        email: profile.email,
                    },
                )
                .await?
            }
        };

        Ok(match user.status {
            UserStatus::Active => Ok(user),
            UserStatus::Pending => Err(SignInError::PendingApproval),
        })
    }
}

impl UserService {
    pub async fn me(&self) -> AppResult<User> {
        let mut conn = self.conn().await?;
        repo::user(&mut conn, self.user_id).await
    }
}

/// 32 bytes from the OS generator: 256 bits, past any guessing.
fn random_token() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("the OS random generator is available");
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Only digests are stored, so a leaked table hands out no working token.
fn hash(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

/// PKCE S256, RFC 7636: base64url of the verifier's SHA-256, unpadded.
fn challenge_of(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

fn is_challenge(value: &str) -> bool {
    value.len() == 43
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn with_query(mut url: Url, key: &str, value: &str) -> Url {
    url.query_pairs_mut().append_pair(key, value);
    url
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The worked example from RFC 7636, appendix B.
    #[test]
    fn challenge_matches_the_rfc_example() {
        assert_eq!(
            challenge_of("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn random_tokens_are_long_and_distinct() {
        let (a, b) = (random_token(), random_token());
        assert_eq!(a.len(), 43);
        assert_ne!(a, b);
        assert!(is_challenge(&challenge_of(&a)));
    }

    #[test]
    fn challenge_shape_is_checked() {
        assert!(!is_challenge("too-short"));
        assert!(!is_challenge(&"a".repeat(42)));
        assert!(!is_challenge(&format!("{}=", "a".repeat(42))));
    }
}
