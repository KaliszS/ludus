mod cli;
mod config;
mod domain;
mod error;
mod http;
mod oauth;
mod password;
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
    let registration = config.auth.registration;
    let service = service::Service::new(pool, config.auth)?;

    // Before binding: an admin command must work while the daemon holds the port.
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        return cli::run(&args, service).await;
    }

    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, registration = registration.as_str(), "ludusd listening");
    axum::serve(listener, http::router(service, &config.allowed_origins)).await?;

    Ok(())
}
