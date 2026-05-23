use anyhow::Result;
use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> Result<()> {
    fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("holla_relay=info".parse()?))
        .init();

    let app = Router::new().route("/healthz", get(healthz));

    let listener = TcpListener::bind("127.0.0.1:46552").await?;
    info!("holla-relay listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn healthz() -> &'static str {
    "ok"
}
