use std::net::SocketAddr;

use anyhow::{Context, bail};

pub struct Config {
    pub bind_addr: SocketAddr,
    pub database_url: String,
    pub allowed_origins: Vec<String>,
    pub auth: AuthConfig,
}

pub struct AuthConfig {
    /// Where the daemon is reachable from outside; provider redirects are built from it.
    pub public_url: String,
    pub google: Option<OAuthClient>,
    /// Where a client may be sent once sign-in finishes. Exact matches only.
    pub redirect_uris: Vec<String>,
    pub registration: Registration,
}

pub struct OAuthClient {
    pub id: String,
    pub secret: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Registration {
    /// Anyone who signs in gets an active account.
    Open,
    /// Unknown accounts are created pending, for an admin to activate.
    Approval,
    /// Only accounts that already exist can sign in.
    Closed,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            bind_addr: var("LUDUS_BIND_ADDR")
                .unwrap_or_else(|| "127.0.0.1:7530".to_owned())
                .parse()
                .context("LUDUS_BIND_ADDR must be host:port")?,
            database_url: var("DATABASE_URL").context("DATABASE_URL is required")?,
            allowed_origins: list("LUDUS_ALLOWED_ORIGINS"),
            auth: AuthConfig::from_env()?,
        })
    }
}

impl AuthConfig {
    fn from_env() -> anyhow::Result<Self> {
        let google = client("LUDUS_GOOGLE_CLIENT_ID", "LUDUS_GOOGLE_CLIENT_SECRET")?;
        let public_url = match var("LUDUS_PUBLIC_URL") {
            Some(url) => url.trim_end_matches('/').to_owned(),
            None if google.is_some() => {
                bail!("LUDUS_PUBLIC_URL is required once a sign-in provider is configured")
            }
            None => String::new(),
        };
        let registration = match var("LUDUS_REGISTRATION").as_deref() {
            None | Some("closed") => Registration::Closed,
            Some("approval") => Registration::Approval,
            Some("open") => Registration::Open,
            Some(other) => {
                bail!("LUDUS_REGISTRATION must be open, approval or closed, not {other:?}")
            }
        };

        Ok(Self {
            public_url,
            google,
            redirect_uris: list("LUDUS_ALLOWED_REDIRECT_URIS"),
            registration,
        })
    }
}

/// A half-configured client is a mistake worth stopping for, not a provider to skip.
fn client(id_var: &str, secret_var: &str) -> anyhow::Result<Option<OAuthClient>> {
    match (var(id_var), var(secret_var)) {
        (Some(id), Some(secret)) => Ok(Some(OAuthClient { id, secret })),
        (None, None) => Ok(None),
        _ => bail!("{id_var} and {secret_var} must be set together"),
    }
}

/// Unset and empty mean the same thing, which is what an `.env` line like `KEY=` intends.
fn var(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn list(name: &str) -> Vec<String> {
    var(name)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_owned)
        .collect()
}
