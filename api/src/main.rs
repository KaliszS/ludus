mod config;
mod domain;
mod error;
mod http;
mod oauth;
mod repo;
mod service;

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("LUDUS_LOG").unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = config::Config::from_env()?;
    let pool = repo::pool::build(&config.database_url)?;
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;

    if config.auth.google.is_none() {
        tracing::warn!("no sign-in provider configured; nobody will be able to sign in");
    }
    tracing::info!(addr = %config.bind_addr, registration = ?config.auth.registration, "ludusd listening");
    let app = http::router(
        service::Service::new(pool, config.auth),
        &config.allowed_origins,
    );
    axum::serve(listener, app).await?;

    Ok(())
}
