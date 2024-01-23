use axum::routing::{get, post};
use axum::Router;

pub mod error;
pub mod routes;
pub mod token;

pub fn register_routes<S>() -> Router<S>
where
    S: std::marker::Sync + std::marker::Send + std::clone::Clone + 'static,
{
    Router::new()
        .route("/auth/login", get(routes::auth::get_login))
        .route("/auth/login", post(routes::auth::post_login))
}
