//! Error handling utilities for gRPC.

use tonic::{Code, Status};

/// Trait for converting errors into gRPC Status responses.
///
/// This mirrors the `HttpError` trait from `server-kit` but maps to gRPC status codes.
///
/// # Example
///
/// ```ignore
/// use server_kit_grpc::GrpcError;
/// use tonic::{Code, Status};
///
/// #[derive(Debug)]
/// enum AppError {
///     NotFound(String),
///     InvalidArgument(String),
/// }
///
/// impl GrpcError for AppError {
///     fn code(&self) -> Code {
///         match self {
///             Self::NotFound(_) => Code::NotFound,
///             Self::InvalidArgument(_) => Code::InvalidArgument,
///         }
///     }
///
///     fn message(&self) -> &str {
///         match self {
///             Self::NotFound(msg) => msg,
///             Self::InvalidArgument(msg) => msg,
///         }
///     }
/// }
///
/// impl From<AppError> for Status {
///     fn from(err: AppError) -> Self {
///         err.into_status()
///     }
/// }
/// ```
pub trait GrpcError: std::fmt::Debug {
    /// The gRPC status code.
    fn code(&self) -> Code;

    /// The error message.
    fn message(&self) -> &str;

    /// Convert to tonic::Status.
    fn into_status(self) -> Status
    where
        Self: Sized,
    {
        Status::new(self.code(), self.message())
    }
}

/// Create a Status from a code and message.
pub fn status(code: Code, message: impl Into<String>) -> Status {
    Status::new(code, message)
}

/// Common error constructors.
pub mod errors {
    use tonic::Status;

    pub fn not_found(message: impl Into<String>) -> Status {
        Status::not_found(message)
    }

    pub fn invalid_argument(message: impl Into<String>) -> Status {
        Status::invalid_argument(message)
    }

    pub fn internal(message: impl Into<String>) -> Status {
        Status::internal(message)
    }

    pub fn unauthenticated(message: impl Into<String>) -> Status {
        Status::unauthenticated(message)
    }

    pub fn permission_denied(message: impl Into<String>) -> Status {
        Status::permission_denied(message)
    }

    pub fn already_exists(message: impl Into<String>) -> Status {
        Status::already_exists(message)
    }

    pub fn unavailable(message: impl Into<String>) -> Status {
        Status::unavailable(message)
    }

    pub fn deadline_exceeded(message: impl Into<String>) -> Status {
        Status::deadline_exceeded(message)
    }
}

/// Crate-level error type.
#[derive(Debug)]
pub enum Error {
    Config(server_kit::ConfigError),
    InvalidEndpoint(String),
    Connection(String),
    #[cfg(feature = "tls")]
    Tls(String),
    Server(ServerError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(e) => write!(f, "Configuration error: {}", e),
            Self::InvalidEndpoint(e) => write!(f, "Invalid endpoint: {}", e),
            Self::Connection(e) => write!(f, "Connection error: {}", e),
            #[cfg(feature = "tls")]
            Self::Tls(e) => write!(f, "TLS error: {}", e),
            Self::Server(e) => write!(f, "Server error: {}", e),
        }
    }
}

impl std::error::Error for Error {}

impl From<server_kit::ConfigError> for Error {
    fn from(err: server_kit::ConfigError) -> Self {
        Self::Config(err)
    }
}

impl From<tonic::transport::Error> for Error {
    fn from(err: tonic::transport::Error) -> Self {
        Self::Connection(err.to_string())
    }
}

#[cfg(feature = "tls")]
impl Error {
    /// Create a TLS error from std::io::Error.
    pub fn tls(err: std::io::Error) -> Self {
        Self::Tls(err.to_string())
    }
}

#[cfg(feature = "tls")]
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Tls(err.to_string())
    }
}

/// Server-specific errors.
#[derive(Debug)]
pub enum ServerError {
    InvalidAddress(std::net::AddrParseError),
    Transport(tonic::transport::Error),
    Bind(std::io::Error),
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAddress(e) => write!(f, "Invalid address: {}", e),
            Self::Transport(e) => write!(f, "Transport error: {}", e),
            Self::Bind(e) => write!(f, "Failed to bind: {}", e),
        }
    }
}

impl std::error::Error for ServerError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestError(Code, String);

    impl GrpcError for TestError {
        fn code(&self) -> Code {
            self.0
        }
        fn message(&self) -> &str {
            &self.1
        }
    }

    #[test]
    fn grpc_error_into_status() {
        let err = TestError(Code::NotFound, "User not found".to_string());
        let status = err.into_status();

        assert_eq!(status.code(), Code::NotFound);
        assert_eq!(status.message(), "User not found");
    }

    #[test]
    fn grpc_error_codes() {
        let cases = vec![
            (Code::Ok, "OK"),
            (Code::NotFound, "Not found"),
            (Code::InvalidArgument, "Invalid"),
            (Code::Internal, "Internal error"),
            (Code::Unauthenticated, "Auth required"),
            (Code::PermissionDenied, "Forbidden"),
        ];

        for (code, msg) in cases {
            let err = TestError(code, msg.to_string());
            let status = err.into_status();
            assert_eq!(status.code(), code);
            assert_eq!(status.message(), msg);
        }
    }

    #[test]
    fn error_constructors() {
        let status = errors::not_found("User not found");
        assert_eq!(status.code(), Code::NotFound);

        let status = errors::invalid_argument("Invalid ID");
        assert_eq!(status.code(), Code::InvalidArgument);

        let status = errors::internal("Database error");
        assert_eq!(status.code(), Code::Internal);

        let status = errors::unauthenticated("Token required");
        assert_eq!(status.code(), Code::Unauthenticated);

        let status = errors::permission_denied("Admin only");
        assert_eq!(status.code(), Code::PermissionDenied);
    }

    #[test]
    fn status_helper() {
        let s = status(Code::Aborted, "Operation aborted");
        assert_eq!(s.code(), Code::Aborted);
        assert_eq!(s.message(), "Operation aborted");
    }

    #[test]
    fn error_display() {
        let err = Error::InvalidEndpoint("bad url".to_string());
        assert!(err.to_string().contains("Invalid endpoint"));

        let err = Error::Connection("connection refused".to_string());
        assert!(err.to_string().contains("Connection error"));
    }
}
