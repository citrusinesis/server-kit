use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::AuthError;
use crate::layer::TokenValidator;

/// JWT configuration.
#[derive(Clone)]
pub struct JwtConfig {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtConfig {
    /// Create a new JWT configuration with a secret key.
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            validation: Validation::default(),
        }
    }

    /// Encode claims into a JWT token.
    pub fn encode<T: Serialize>(&self, claims: &T) -> Result<String, AuthError> {
        encode(&Header::default(), claims, &self.encoding_key)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))
    }

    /// Decode a JWT token into claims.
    pub fn decode<T: DeserializeOwned>(&self, token: &str) -> Result<T, AuthError> {
        decode::<T>(token, &self.decoding_key, &self.validation)
            .map(|data| data.claims)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                _ => AuthError::InvalidToken(e.to_string()),
            })
    }
}

impl TokenValidator for JwtConfig {
    fn validate(&self, token: &str) -> Result<(), AuthError> {
        self.decode::<Claims>(token).map(|_| ())
    }
}

/// Standard JWT claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (user ID).
    pub sub: String,
    /// Expiration time (Unix timestamp).
    pub exp: u64,
    /// Issued at (Unix timestamp).
    #[serde(default = "now")]
    pub iat: u64,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

impl Claims {
    /// Create new claims for a subject with expiration in seconds from now.
    pub fn new(sub: impl Into<String>, expires_in_secs: u64) -> Self {
        let now = now();
        Self {
            sub: sub.into(),
            exp: now + expires_in_secs,
            iat: now,
        }
    }
}
