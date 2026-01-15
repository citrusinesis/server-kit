use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

/// Trait for converting errors into HTTP responses.
///
/// # Example
///
/// ```ignore
/// use server_kit::HttpError;
/// use axum::http::StatusCode;
/// use axum::response::{IntoResponse, Response};
///
/// #[derive(Debug)]
/// enum AppError {
///     NotFound,
///     InvalidInput(String),
/// }
///
/// impl HttpError for AppError {
///     fn status_code(&self) -> StatusCode {
///         match self {
///             Self::NotFound => StatusCode::NOT_FOUND,
///             Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
///         }
///     }
///
///     fn message(&self) -> &str {
///         match self {
///             Self::NotFound => "Resource not found",
///             Self::InvalidInput(msg) => msg,
///         }
///     }
/// }
///
/// impl IntoResponse for AppError {
///     fn into_response(self) -> Response {
///         self.into_http_response()
///     }
/// }
/// ```
pub trait HttpError: std::fmt::Debug {
    fn status_code(&self) -> StatusCode;
    fn message(&self) -> &str;

    fn error_code(&self) -> String {
        status_to_error_code(self.status_code())
    }

    fn into_http_response(self) -> Response
    where
        Self: Sized,
    {
        let body = ErrorResponse {
            code: self.error_code(),
            message: self.message().to_string(),
        };
        (self.status_code(), axum::Json(body)).into_response()
    }
}

/// Standard JSON error response format.
#[derive(Debug, Serialize, Clone)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Create an error response from a status code.
    pub fn from_status(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            code: status_to_error_code(status),
            message: message.into(),
        }
    }
}

/// Convert a status code to an error code string (e.g., "NOT_FOUND").
pub(crate) fn status_to_error_code(status: StatusCode) -> String {
    status
        .canonical_reason()
        .unwrap_or("ERROR")
        .to_uppercase()
        .replace(' ', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_to_error_code_common_codes() {
        assert_eq!(status_to_error_code(StatusCode::NOT_FOUND), "NOT_FOUND");
        assert_eq!(status_to_error_code(StatusCode::BAD_REQUEST), "BAD_REQUEST");
        assert_eq!(status_to_error_code(StatusCode::INTERNAL_SERVER_ERROR), "INTERNAL_SERVER_ERROR");
        assert_eq!(status_to_error_code(StatusCode::UNAUTHORIZED), "UNAUTHORIZED");
        assert_eq!(status_to_error_code(StatusCode::FORBIDDEN), "FORBIDDEN");
    }

    #[test]
    fn error_response_new() {
        let resp = ErrorResponse::new("TEST_CODE", "Test message");
        assert_eq!(resp.code, "TEST_CODE");
        assert_eq!(resp.message, "Test message");
    }

    #[test]
    fn error_response_from_status() {
        let resp = ErrorResponse::from_status(StatusCode::NOT_FOUND, "Resource not found");
        assert_eq!(resp.code, "NOT_FOUND");
        assert_eq!(resp.message, "Resource not found");
    }

    #[derive(Debug)]
    struct TestError {
        status: StatusCode,
        msg: String,
    }

    impl HttpError for TestError {
        fn status_code(&self) -> StatusCode {
            self.status
        }

        fn message(&self) -> &str {
            &self.msg
        }
    }

    #[test]
    fn http_error_error_code() {
        let err = TestError {
            status: StatusCode::BAD_REQUEST,
            msg: "Invalid input".to_string(),
        };
        assert_eq!(err.error_code(), "BAD_REQUEST");
    }

    #[test]
    fn http_error_into_response() {
        let err = TestError {
            status: StatusCode::NOT_FOUND,
            msg: "Not found".to_string(),
        };
        let response = err.into_http_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
