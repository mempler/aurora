use axum::{routing::get, Router};
use const_format::formatcp;

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    let app = Router::new() //
        .route("/", get(root));

    info!("listening on :3000 :: {:#?}", root().await);
    info!("Available routes:");
    info!("  http://localhost:3000/");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    formatcp!("aurora-api@{}", env!("CARGO_PKG_VERSION"))
}
