//! The provider side of sign-in: building the authorize URL and turning the code
//! that comes back into a verified profile. Nothing here touches the database.

use anyhow::{Context, bail};
use reqwest::Url;
use serde::Deserialize;

use crate::config::OAuthClient;
use crate::domain::OAuthProvider;

const GOOGLE_AUTHORIZE: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO: &str = "https://openidconnect.googleapis.com/v1/userinfo";

/// Who the provider says signed in. `subject` is the provider's stable id; the
/// email can change, so it is never used to recognise a returning user.
pub struct Profile {
    pub subject: String,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
}

pub fn authorize_url(
    provider: OAuthProvider,
    client: &OAuthClient,
    redirect_uri: &str,
    state: &str,
    challenge: &str,
) -> Url {
    match provider {
        OAuthProvider::Google => Url::parse_with_params(
            GOOGLE_AUTHORIZE,
            [
                ("client_id", client.id.as_str()),
                ("redirect_uri", redirect_uri),
                ("response_type", "code"),
                ("scope", "openid email profile"),
                ("state", state),
                ("code_challenge", challenge),
                ("code_challenge_method", "S256"),
                ("prompt", "select_account"),
            ],
        )
        .expect("constant base URL"),
    }
}

pub async fn exchange(
    http: &reqwest::Client,
    provider: OAuthProvider,
    client: &OAuthClient,
    redirect_uri: &str,
    code: &str,
    verifier: &str,
) -> anyhow::Result<Profile> {
    match provider {
        OAuthProvider::Google => google(http, client, redirect_uri, code, verifier).await,
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct GoogleUser {
    sub: String,
    email: String,
    #[serde(default)]
    email_verified: bool,
    name: Option<String>,
    picture: Option<String>,
}

async fn google(
    http: &reqwest::Client,
    client: &OAuthClient,
    redirect_uri: &str,
    code: &str,
    verifier: &str,
) -> anyhow::Result<Profile> {
    let response = http
        .post(GOOGLE_TOKEN)
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", client.id.as_str()),
            ("client_secret", client.secret.as_str()),
            ("redirect_uri", redirect_uri),
            ("code_verifier", verifier),
        ])
        .send()
        .await
        .context("token request")?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        bail!("token exchange failed with {status}: {body}");
    }
    let token: TokenResponse = response.json().await.context("token response")?;

    let user: GoogleUser = http
        .get(GOOGLE_USERINFO)
        .bearer_auth(&token.access_token)
        .send()
        .await
        .context("userinfo request")?
        .error_for_status()
        .context("userinfo status")?
        .json()
        .await
        .context("userinfo response")?;

    // An unverified address could belong to anyone; it must not open an account.
    if !user.email_verified {
        bail!("google account email is not verified");
    }

    Ok(Profile {
        name: user.name.unwrap_or_else(|| user.email.clone()),
        subject: user.sub,
        email: user.email,
        avatar_url: user.picture,
    })
}
