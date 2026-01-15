//! Error handling utilities for gRPC.

use tonic::{Code, Status};

/// Trait for converting errors into gRPC Status responses.
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
    fn code(&self) -> Code;
    fn message(&self) -> &str;

    fn into_status(self) -> Status
    where
        Self: Sized,
    {
        Status::new(self.code(), self.message())
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
    fn error_display() {
        let err = Error::InvalidEndpoint("bad url".to_string());
        assert!(err.to_string().contains("Invalid endpoint"));

        let err = Error::Connection("connection refused".to_string());
        assert!(err.to_string().contains("Connection error"));
    }
}
