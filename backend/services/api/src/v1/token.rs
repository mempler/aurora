use axum::http::HeaderMap;
use base64::prelude::*;
use hmac::{Hmac, Mac};
use sha2::Sha512;
use time::{Date, Time, UtcOffset};

use super::error::{APIError, APIResult};

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
#[derive(Debug)]
pub struct AuthenticationToken {
    /// The user ID of the user this token belongs to.
    pub user_id: u64,

    /// The time this token was generated. in seconds since the first epoch. [FIRST_EPOCH]
    pub generation_time: i64,

    /// The HMAC of the token. It is composed from the generation time and the user ID. + a secret key. [HMAC_SECURITY_KEY]
    pub hmac: String,
}

pub const HMAC_SECURITY_KEY: &[u8] = b"TODO: secret key";

/// The first epoch is basically the first time when our first token was generated.
/// to cut down on the size of the token.
pub const FIRST_EPOCH: time::OffsetDateTime =
    match Date::from_calendar_date(2024, time::Month::January, 23) {
        Ok(date) => time::OffsetDateTime::new_in_offset(date, Time::MIDNIGHT, UtcOffset::UTC),
        Err(_) => panic!("Failed to create first epoch."),
    };

impl AuthenticationToken {
    pub fn new(user_id: u64) -> APIResult<Self> {
        let mut token = AuthenticationToken {
            user_id,
            generation_time: 0,
            hmac: "".to_string(),
        };
        token.update_secure_parts()?;
        Ok(token)
    }

    /// Update the secure parts of the token.
    ///
    pub fn update_secure_parts(&mut self) -> APIResult<()> {
        // Calculate the current time based on the first epoch.
        let current_based_on_epoch = time::OffsetDateTime::now_utc() - FIRST_EPOCH;

        let mut hmac = Hmac::<Sha512>::new_from_slice(HMAC_SECURITY_KEY).map_err(|err| {
            warn!("Failed to generate HMAC for token: {err}.");
            APIError::FailedToGenerateToken
        })?;

        self.generation_time = current_based_on_epoch.whole_seconds();

        hmac.update(
            format!(
                "{user_id}.{generation_time}",
                user_id = BASE64_STANDARD.encode(&self.user_id.to_string()),
                generation_time = BASE64_STANDARD.encode(self.generation_time.to_be_bytes())
            )
            .as_bytes(),
        );

        self.hmac = BASE64_STANDARD.encode(&hmac.finalize().into_bytes());

        Ok(())
    }

    /// Verify the token.
    ///
    pub fn verify(&self) -> APIResult<()> {
        let mut hmac = Hmac::<Sha512>::new_from_slice(HMAC_SECURITY_KEY).map_err(|err| {
            warn!("Failed to generate HMAC for token: {err}.");
            APIError::FailedToGenerateToken
        })?;

        hmac.update(
            format!(
                "{user_id}.{generation_time}",
                user_id = BASE64_STANDARD.encode(&self.user_id.to_string()),
                generation_time = BASE64_STANDARD.encode(self.generation_time.to_be_bytes())
            )
            .as_bytes(),
        );

        hmac.verify_slice(
            BASE64_STANDARD
                .decode(&self.hmac)
                .map_err(|err| {
                    warn!("Failed to decode HMAC for token: {err}.");
                    APIError::InvalidToken
                })?
                .as_slice(),
        )
        .map_err(|err| {
            warn!("Failed to verify HMAC for token: {err}.");
            APIError::InvalidToken
        })?;

        Ok(())
    }

    pub fn from_token(token: &str) -> APIResult<Self> {
        let components = token.split('.').collect::<Vec<&str>>();
        if components.len() < 3 {
            warn!("Invalid token format: {token}.");
            return Err(APIError::InvalidToken);
        }

        let user_id: u64 = String::from_utf8(
            BASE64_STANDARD //
                .decode(components[0])
                .map_err(|err| {
                    warn!("Failed to decode user ID from token >> {token}: {err}.");
                    APIError::InvalidToken
                })?,
        )
        .map_err(|err| {
            warn!("Failed to decode user ID from token >> {token}: {err}.");
            APIError::InvalidToken
        })?
        .parse()
        .map_err(|err| {
            warn!("Failed to parse user ID from token >> {token}: {err}.");
            APIError::InvalidToken
        })?;

        // FIXME: generation time
        // FIXME: hmac

        Ok(Self {
            user_id,
            generation_time: 0,
            hmac: components[2].to_string(),
        })
    }

    pub fn from_headers(headers: &HeaderMap) -> APIResult<Self> {
        let auth_header = headers
            .get("Authorization")
            .ok_or(APIError::MissingHeader {
                header: "Authorization",
            })?
            .to_str()
            .map_err(|_| APIError::InvalidHeader {
                header: "Authorization",
                format: "valid UTF-8 string",
            })?;

        if !auth_header.starts_with("Bearer ") {
            return Err(APIError::InvalidHeader {
                header: "Authorization",
                format: "Authorization: Bearer <token>",
            });
        }

        let token = auth_header.trim_start_matches("Bearer ");

        Self::from_token(token)
    }
}

impl Into<String> for AuthenticationToken {
    fn into(self) -> String {
        format!(
            "{user_id}.{generation_time}.{hmac}",
            user_id = BASE64_STANDARD.encode(&self.user_id.to_string()),
            generation_time = BASE64_STANDARD.encode(self.generation_time.to_be_bytes()),
            hmac = self.hmac
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation() {
        let token = AuthenticationToken::new(1).unwrap();
        assert_eq!(token.user_id, 1);
        assert_ne!(token.generation_time, 0);
        assert_ne!(token.hmac, "");
    }

    #[test]
    fn test_token_verification() {
        let token = AuthenticationToken::new(1).unwrap();
        assert!(token.verify().is_ok());
    }
}
