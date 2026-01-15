use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::fmt;

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken(String),
    TokenExpired,
    Forbidden,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingToken => write!(f, "Missing authorization token"),
            Self::InvalidToken(msg) => write!(f, "Invalid token: {}", msg),
            Self::TokenExpired => write!(f, "Token has expired"),
            Self::Forbidden => write!(f, "Insufficient permissions"),
        }
    }
}

impl std::error::Error for AuthError {}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::MissingToken | Self::InvalidToken(_) | Self::TokenExpired => {
                StatusCode::UNAUTHORIZED
            }
            Self::Forbidden => StatusCode::FORBIDDEN,
        };

        let body = serde_json::json!({
            "code": status.canonical_reason().unwrap_or("ERROR").to_uppercase().replace(' ', "_"),
            "message": self.to_string()
        });

        (status, axum::Json(body)).into_response()
    }
}
