use axum::http::HeaderMap;
use base64::prelude::*;
use hmac::{Hmac, Mac};
use sha2::Sha512;
use time::{Date, Time, UtcOffset};

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum TokenError {
    #[error("Failed to generate HMAC for token.")]
    HmacGeneration,

    #[error("Failed to decode HMAC for token.")]
    HmacDecoding,

    #[error("Failed to verify HMAC for token.")]
    HmacVerification,

    #[error("Failed to decode user ID from token.")]
    UserIdBase64Decoding,

    #[error("Failed to decode user ID from token.")]
    UserIdUtf8Decoding,

    #[error("Failed to parse user ID from token.")]
    UserIdParsing,

    #[error("Failed to decode generation time from token.")]
    GenerationTimeDecoding,

    #[error("Invalid token format.")]
    InvalidFormat,

    #[error("Invalid token.")]
    InvalidToken,

    #[error("Missing authorization header.")]
    MissingAuthorizationHeader,

    #[error("Invalid authorization header.")]
    InvalidAuthorizationHeader,

    #[error("Invalid authorization header format.")]
    InvalidAuthorizationHeaderFormat,
}

const BASE64: base64::engine::GeneralPurpose = BASE64_STANDARD;

type Result<T> = std::result::Result<T, TokenError>;

/// Authentication token.
///
/// it is composed from a user ID, a generation time and a HMAC. which is used to verify the token.
///
/// The token is encoded as follows:
/// ```text
/// // <user_id>         := Base64(<string>)
/// // <generation_time> := Base64(<u64>)
/// // <hmac>            := Base64(<string>)
///
/// <user_id>.<generation_time>.<hmac>
/// ```
///
#[derive(Debug, Clone)]
pub struct AuthenticationToken {
    /// The user ID of the user this token belongs to.
    pub user_id: u64,

    /// The time this token was generated. in milliseconds since the first epoch. [FIRST_EPOCH]
    pub generation_time: i64,

    /// The HMAC of the token. It is composed from the generation time and the user ID. + a secret key. [HMAC_SECURITY_KEY]
    pub hmac: Vec<u8>,
}

lazy_static! {
    /// The HMAC security key.
    static ref HMAC_SECURITY_KEY: Vec<u8> = std::env::var("HMAC_SECURITY_KEY")
        .expect("HMAC_SECURITY_KEY must be set")
        .into_bytes();

    /// Amount of time in seconds before a token expires.
    static ref TOKEN_EXPIRATION_TIME: i64 = std::env::var("TOKEN_EXPIRATION_TIME")
        .expect("TOKEN_EXPIRATION_TIME must be set")
        .parse()
        .expect("TOKEN_EXPIRATION_TIME must be a valid integer");
}

/// The first epoch is basically the first time when our first token was generated.
/// to cut down on the size of the token.
pub const FIRST_EPOCH: time::OffsetDateTime =
    match Date::from_calendar_date(2024, time::Month::January, 23) {
        Ok(date) => time::OffsetDateTime::new_in_offset(date, Time::MIDNIGHT, UtcOffset::UTC),
        Err(_) => panic!("Failed to create first epoch."),
    };

impl AuthenticationToken {
    pub fn new(user_id: u64) -> Result<Self> {
        let mut token = AuthenticationToken {
            user_id,
            generation_time: 0,
            hmac: Vec::new(),
        };
        token.update_secure_parts()?;
        Ok(token)
    }

    /// Update the secure parts of the token.
    ///
    pub fn update_secure_parts(&mut self) -> Result<()> {
        let current_based_on_epoch = time::OffsetDateTime::now_utc() - FIRST_EPOCH;

        let mut hmac = Hmac::<Sha512>::new_from_slice(&HMAC_SECURITY_KEY)
            .map_err(|_| TokenError::HmacGeneration)?;

        self.generation_time = current_based_on_epoch.whole_milliseconds() as i64; // This will overflow in 292 million years. I think we are good.

        hmac.update(
            format!(
                "{user_id}.{generation_time}",
                user_id = self.user_id,
                generation_time = self.generation_time
            )
            .as_bytes(),
        );

        self.hmac = hmac.finalize().into_bytes().to_vec();

        Ok(())
    }

    /// Verify the token.
    ///
    /// # Errors
    /// - [TokenError::HmacGeneration] Failed to create HMAC for validation.
    /// - [TokenError::HmacVerification] if the HMAC is not valid.
    pub fn verify(&self) -> Result<()> {
        let mut hmac = Hmac::<Sha512>::new_from_slice(&HMAC_SECURITY_KEY)
            .map_err(|_| TokenError::HmacGeneration)?;

        hmac.update(
            format!(
                "{user_id}.{generation_time}",
                user_id = self.user_id,
                generation_time = self.generation_time
            )
            .as_bytes(),
        );

        hmac.verify_slice(&self.hmac)
            .map_err(|_| TokenError::HmacVerification)?;

        Ok(())
    }

    /// Checks if the token is expired.
    pub fn expired(&self) -> bool {
        let current_based_on_epoch = time::OffsetDateTime::now_utc() - FIRST_EPOCH;
        let current_time = current_based_on_epoch.whole_milliseconds() as i64;

        current_time - self.generation_time > (*TOKEN_EXPIRATION_TIME * 1000)
    }

    /// Create a token from a string.
    ///
    /// # Errors
    ///
    /// - [TokenError::InvalidFormat] The token is not in the correct format.
    /// - [TokenError::UserIdBase64Decoding] Failed to decode the user ID from Base64.
    /// - [TokenError::UserIdUtf8Decoding] Failed to decode the user ID from UTF8 (via Base64).
    /// - [TokenError::UserIdParsing] Failed to parse the user ID from string.
    /// - [TokenError::GenerationTimeDecoding] Failed to decode the generation time from Base64.
    /// - [TokenError::HmacDecoding] Failed to decode the HMAC from Base64.
    /// - everything that [AuthenticationToken::verify] can return.
    pub fn from_token<S>(token: &S) -> Result<Self>
    where
        S: AsRef<str> + ?Sized,
    {
        let token = token.as_ref();

        let components = token.split('.').collect::<Vec<&str>>();
        if components.len() < 3 {
            return Err(TokenError::InvalidFormat);
        }

        let user_id: u64 = {
            let base64_decoded = BASE64
                .decode(components[0]) //
                .map_err(|_| TokenError::UserIdBase64Decoding)?;

            let utf8_decoded =
                String::from_utf8(base64_decoded).map_err(|_| TokenError::UserIdUtf8Decoding)?;

            let u64_decoded = utf8_decoded
                .parse()
                .map_err(|_| TokenError::UserIdParsing)?;

            u64_decoded
        };

        //
        // Decode Generation time.
        //
        let generation_time: i64 = {
            let base64_decoded = BASE64
                .decode(components[1]) //
                .map_err(|_| TokenError::GenerationTimeDecoding)?;

            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&base64_decoded);

            i64::from_be_bytes(bytes)
        };

        //
        // Decode HMAC.
        //
        let hmac: Vec<u8> = {
            let base64_decoded = BASE64
                .decode(components[2]) //
                .map_err(|_| TokenError::HmacDecoding)?;

            base64_decoded
        };

        let token = Self {
            user_id,
            generation_time: generation_time as i64,
            hmac,
        };

        token.verify()?;

        Ok(token)
    }

    /// Shortcut to create a token from headers.
    ///
    /// # Errors
    /// - [TokenError::MissingAuthorizationHeader] Authorization header is missing.
    /// - [TokenError::InvalidAuthorizationHeader] Authorization header is invalid.
    /// - [TokenError::InvalidAuthorizationHeaderFormat] Authorization header is invalid.
    /// - everything that [AuthenticationToken::from_token] can return.
    pub fn from_headers(headers: &HeaderMap) -> Result<Self> {
        let auth_header = headers
            .get("Authorization")
            .ok_or(TokenError::MissingAuthorizationHeader)?
            .to_str()
            .map_err(|_| TokenError::InvalidAuthorizationHeader)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(TokenError::InvalidAuthorizationHeaderFormat);
        }

        let token = auth_header.trim_start_matches("Bearer ");

        Self::from_token(token)
    }
}

impl From<AuthenticationToken> for String {
    fn from(token: AuthenticationToken) -> Self {
        format!(
            "{user_id}.{generation_time}.{hmac}",
            user_id = BASE64.encode(token.user_id.to_string()),
            generation_time = BASE64.encode(token.generation_time.to_be_bytes()),
            hmac = BASE64.encode(token.hmac),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use tracing_test::traced_test;

    pub fn setup() {
        std::env::set_var("HMAC_SECURITY_KEY", "TODO: secret key");
        std::env::set_var("TOKEN_EXPIRATION_TIME", "3600");
    }

    const VALID_TOKEN: &str = "MTgzNzE4MjYwNjc0NTI3MjMy.AAAAAAN9aas=.k+eOfjZ/xAvzdAO9Tmfidj4NPtJT1FEyh9EMegZLhDGufawSO3Q+PD1EGZiGv7rpoFL9v4h/8TwLq9IWVxE9wA==";
    const INVALID_HMAC_TOKEN: &str = "MTgzNzE4MjYwNjc0NTI3MjMy.AAAAAAN9aas=.k+eOfjx/xAvzdAO9Tmfidj4NPtJT1FEyh9EMegZLhDGufawSO3Q+PD1EGZiGv7rpoFL9v4h/8TwLq9IWVxE9wA==";

    #[tokio::test]
    #[traced_test]
    async fn test_token_generation() {
        setup();

        let result = AuthenticationToken::new(1);
        match result {
            Ok(token) => {
                assert_eq!(token.user_id, 1);
                assert_ne!(token.generation_time, 0);
                assert_ne!(token.hmac.len(), 0);
            }
            Err(err) => panic!("Failed to generate token: {}", err),
        }
    }

    #[tokio::test]
    #[traced_test]
    async fn test_token_verification() {
        setup();

        // Synthetic token.
        let result = AuthenticationToken::new(1);
        match result {
            Ok(token) => assert!(token.verify().is_ok()),
            Err(err) => panic!("Failed to generate token: {}", err),
        }

        // Real world token.
        let result = AuthenticationToken::from_token(VALID_TOKEN);
        match result {
            Ok(token) => assert!(token.verify().is_ok()),
            Err(err) => panic!("Failed to read valid token: {}", err),
        }

        // Real world token with invalid HMAC.
        let result = AuthenticationToken::from_token(INVALID_HMAC_TOKEN);
        match result {
            Ok(token) => panic!("Token should be invalid: {:?}", token),
            Err(err) => assert_eq!(err, TokenError::HmacVerification),
        }
    }

    #[tokio::test]
    #[traced_test]
    async fn test_token_from_token() {
        setup();

        let first_token = AuthenticationToken::new(1).unwrap();
        let first_token_string: String = first_token.clone().into();

        let token = AuthenticationToken::from_token(&first_token_string).unwrap();

        assert_eq!(token.user_id, 1);
        assert_eq!(token.generation_time, first_token.generation_time);
        assert_eq!(token.hmac, first_token.hmac);

        assert!(token.verify().is_ok());
    }

    #[tokio::test]
    #[traced_test]
    async fn test_token_from_invalid_token_components() {
        setup();

        // invalid token format
        assert!(AuthenticationToken::from_token("invalid token")
            .is_err_and(|e| e == TokenError::InvalidFormat));

        // Not enough components.
        assert!(AuthenticationToken::from_token("invalid.token")
            .is_err_and(|e| e == TokenError::InvalidFormat));

        // Enough components but invalid format. UID shouldnt work.
        assert!(AuthenticationToken::from_token("invalid.token.invalid")
            .is_err_and(|e| e == TokenError::UserIdBase64Decoding));
    }

    #[tokio::test]
    #[traced_test]
    async fn test_token_from_invalid_token_base64() {
        setup();

        // Valid token buth with invalid user ID Base64.
        assert!(AuthenticationToken::from_token("MTgzNzE4MjYwNjc0NTI3MjMy!.AAAAAAAA0Fw=.ijhqOyJ7NX+oia4iDUt+T9uC5RpJcIRq/5Xx7ClQQ1HiP2yRSzkw0nckaacw3dzmmj5OGx8zEQu7GF6h/l5Fjw==").is_err_and(|e| e == TokenError::UserIdBase64Decoding));

        // Valid token but with invalid generation time Base64.
        assert!(AuthenticationToken::from_token("MTgzNzE4MjYwNjc0NTI3MjMy.AAAAAAAA0Fw!=.ijhqOyJ7NX+oia4iDUt+T9uC5RpJcIRq/5Xx7ClQQ1HiP2yRSzkw0nckaacw3dzmmj5OGx8zEQu7GF6h/l5Fjw==").is_err_and(|e| e == TokenError::GenerationTimeDecoding));

        // Valid token but with invalid HMAC Base64.
        assert!(AuthenticationToken::from_token("MTgzNzE4MjYwNjc0NTI3MjMy.AAAAAAAA0Fw=.ijhqOyJ7NX+oia4iDUt+T9uC5RpJcIRq/5Xx7ClQQ1HiP2yRSzkw0nckaacw3dzmmj5OGx8zEQu7GF6h/l5Fjw!=").is_err_and(|e| e == TokenError::HmacDecoding));
    }

    #[tokio::test]
    #[traced_test]
    async fn test_token_from_headers() {
        setup();

        let token = AuthenticationToken::new(1).unwrap();
        let token_string: String = token.clone().into();

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", token_string).parse().unwrap(),
        );

        let token = match AuthenticationToken::from_headers(&headers) {
            Ok(token) => token,
            Err(err) => panic!("Failed to read token from headers: {}", err),
        };

        assert_eq!(token.user_id, 1);
        assert_eq!(token.generation_time, token.generation_time);
        assert_eq!(token.hmac, token.hmac);

        assert!(token.verify().is_ok());
    }

    #[tokio::test]
    #[traced_test]
    async fn test_token_from_invalid_header() {
        setup();

        let token = AuthenticationToken::new(1).unwrap();
        let token_string: String = token.clone().into();

        //
        // Missing Bearer.
        //
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("{}", token_string).parse().unwrap(),
        );

        assert!(AuthenticationToken::from_headers(&headers)
            .is_err_and(|e| e == TokenError::InvalidAuthorizationHeaderFormat));

        //
        // Invalid UTF8.
        //
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_bytes(b"Bearer \xc3\x28").unwrap(),
        );

        assert!(AuthenticationToken::from_headers(&headers)
            .is_err_and(|e| e == TokenError::InvalidAuthorizationHeader));
    }

    #[tokio::test]
    #[traced_test]
    async fn test_token_from_missing_header() {
        setup();

        let headers = HeaderMap::new();

        assert!(AuthenticationToken::from_headers(&headers)
            .is_err_and(|e| e == TokenError::MissingAuthorizationHeader));
    }

    #[tokio::test]
    #[traced_test]
    async fn test_token_expired() {
        setup();

        let mut token = AuthenticationToken::new(1).unwrap();
        assert!(!token.expired());

        token.generation_time = 0;
        assert!(token.expired());
    }
}
