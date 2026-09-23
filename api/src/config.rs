use std::net::SocketAddr;

use anyhow::Context;

pub struct Config {
    pub bind_addr: SocketAddr,
    pub database_url: String,
    pub allowed_origins: Vec<String>,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            bind_addr: std::env::var("LUDUS_BIND_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:7530".to_owned())
                .parse()
                .context("LUDUS_BIND_ADDR must be host:port")?,
            database_url: std::env::var("DATABASE_URL").context("DATABASE_URL is required")?,
            allowed_origins: std::env::var("LUDUS_ALLOWED_ORIGINS")
                .unwrap_or_default()
                .split(',')
                .map(str::trim)
                .filter(|origin| !origin.is_empty())
                .map(str::to_owned)
                .collect(),
        })
    }
}
