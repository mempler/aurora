use axum::http::StatusCode;

use crate::v1::error::APIError;

#[axum::debug_handler]
pub async fn get_login() -> Result<String, APIError> {
    Err(APIError::GenericError(
        StatusCode::UNAUTHORIZED,
        "Not implemented".to_string(),
    ))
}

pub async fn post_login() {
    unimplemented!()
}
