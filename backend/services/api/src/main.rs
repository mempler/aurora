use axum::{routing::get, Router};
use const_format::formatcp;
use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;
use tokio::time::Duration;
use tower_http::trace::TraceLayer;

mod v1;

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate lazy_static;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv()?;

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .init();

    //
    // Database Connection
    //
    let db_connection_str = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&db_connection_str)
        .await?;

    let app = Router::new() //
        .route("/", get(root))
        .nest("/api/v1", v1::register_routes())
        .with_state(pool)
        .layer(TraceLayer::new_for_http());

    info!("listening on :3000 :: {:#?}", root().await);
    info!("Available routes:");
    info!("  http://localhost:3000/");
    info!("  http://localhost:3000/api/v1/auth/login");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    formatcp!("aurora-api@{}", env!("CARGO_PKG_VERSION"))
}
