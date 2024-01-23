use axum::http::{HeaderMap, StatusCode};

use crate::v1::{
    error::{APIError, APIResult},
    token::AuthenticationToken,
};

#[axum::debug_handler]
pub async fn get_login(headers: HeaderMap) -> APIResult<String> {
    //let token = AuthenticationToken::from_headers(&headers)?;
    //trace!("Token: {:?}", token);

    Err(APIError::GenericError(
        StatusCode::UNAUTHORIZED,
        "Not logged in".to_string(),
    ))
}

pub async fn post_login() -> APIResult<String> {
    let token = AuthenticationToken::new(183718260674527232).unwrap();

    // TODO: implement real login
    Ok(token.into())
}
