use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

/// Generalized error type for the API.
///
/// This is used to return errors from the API.
/// and allows us to easily add new errors in the future.
///
/// 10000 - Unknown entities
/// 20000 - Bot-related errors
/// 30000 - Limits reached
/// 40000 - Authorization errors
/// 50000 - Access errors
#[repr(u64)]
#[derive(thiserror::Error, Clone, Debug)]
pub enum APIError {
    #[error("{0} - {1}")]
    GenericError(StatusCode, String) = 0,
    /// A user was requested, but we don't know them.
    /// Perhaps they were deleted? or perhaps they never existed?
    /// we don't know.
    #[error("The user requested is not known to us: '{who:?}'.")]
    UnknownUser { who: Option<String> } = 10001,

    /// A header was missing from the request.
    #[error("Lack of {header} header")]
    MissingHeader { header: &'static str } = 40001,

    /// A header was present, but it was not in the correct format.
    #[error("Invalid {header} header format. Must be: '{format}'.")]
    InvalidHeader {
        header: &'static str,
        format: &'static str,
    } = 40002,

    /// The token provided was invalid. since this is PII, we don't want to leak any information on why it was invalid.
    #[error("Invalid token provided.")]
    InvalidToken = 40003,

    #[error("Failed to generate a token. This is a server-side error.")]
    FailedToGenerateToken = 40004,
}

impl APIError {
    fn discriminant(&self) -> u64 {
        // SAFETY: because #[repr(u64)] is used, the discriminant is the first 8 bytes of the enum
        unsafe { *(self as *const Self as *const u64) }
    }
}

pub type APIResult<T> = Result<T, APIError>;

#[derive(serde::Serialize)]
struct JSONError {
    code: u64,
    message: String,
}

/// Simple macro to reduce code duplication.
macro_rules! impl_err {
    ($error:expr, $status_code:expr) => {
        (
            $status_code,
            JSONError {
                code: $error.discriminant(),
                message: $error.to_string(),
            },
        )
    };
}

impl IntoResponse for APIError {
    fn into_response(self) -> Response {
        let (status_code, obj) = match &self {
            // 0 - Generic error
            Self::GenericError(status_code, _) => impl_err!(self, *status_code),

            // 10000 - Unknown entities
            Self::UnknownUser { .. } => impl_err!(self, StatusCode::NOT_FOUND),

            // 40000 - Authorization errors
            Self::MissingHeader { .. } => impl_err!(self, StatusCode::BAD_REQUEST),
            Self::InvalidHeader { .. } => impl_err!(self, StatusCode::BAD_REQUEST),
            Self::InvalidToken => impl_err!(self, StatusCode::UNAUTHORIZED),
            Self::FailedToGenerateToken => impl_err!(self, StatusCode::INTERNAL_SERVER_ERROR),
        };

        (status_code, Json(obj)).into_response()
    }
}
