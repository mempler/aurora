use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[repr(u64)]
#[derive(thiserror::Error, Clone, Debug)]
pub enum APIError {
    #[error("{0} - {1}")]
    GenericError(StatusCode, String) = 0,
    // 10000 - Unknown entities
    #[error("Unknown user: {who:?}'")]
    UnknownUser { who: Option<String> } = 10000,
    // 20000 - Bot-related errors
    // 30000 - Limits reached
    // 40000 - Authorization errors
    // 50000 - Access errors
}

impl APIError {
    fn discriminant(&self) -> u64 {
        // SAFETY: because #[repr(u64)] is used, the discriminant is the first 8 bytes of the enum
        unsafe { *(self as *const Self as *const u64) }
    }
}

#[derive(serde::Serialize)]
struct JSONError {
    code: u64,
    message: String,
}

impl IntoResponse for APIError {
    fn into_response(self) -> Response {
        let (status_code, obj) = match &self {
            Self::GenericError(status_code, _) => (
                *status_code,
                JSONError {
                    code: self.discriminant(),
                    message: self.to_string(),
                },
            ),

            Self::UnknownUser { .. } => (
                StatusCode::NOT_FOUND,
                JSONError {
                    code: self.discriminant(),
                    message: self.to_string(),
                },
            ),
        };

        let body = serde_json::to_string(&obj);

        match body {
            Ok(body) => (status_code, body).into_response(),
            Err(err) => {
                error!("Failed to serialize error: {}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error.").into_response()
            }
        }
    }
}
