use axum::http::{HeaderMap, StatusCode};

use crate::v1::{
    error::{APIError, APIResult},
    token::AuthenticationToken,
};

/// GET /api/v1/auth/login - used to refresh a token. It must be called every login.
///                          returns a new token; The old one is valid until it expires.
///
#[axum::debug_handler]
pub async fn get_login(headers: HeaderMap) -> APIResult<String> {
    let token = AuthenticationToken::from_headers(&headers)?;
    if token.expired() {
        return Err(APIError::ExpiredToken);
    }

    // TODO: store it in some kind of database to check for revocation

    // otherwise, we can now refresh the token
    let new_token = token.refresh();
    Ok(new_token.into())
}

/// POST /api/v1/auth/login - used to authenticate a user through Username/Password
///                           may have multiple stages (e.g. 2FA)
pub async fn post_login() -> APIResult<String> {
    let token = AuthenticationToken::new(183718260674527232).unwrap();

    // TODO: implement real login
    Ok(token.into())
}
