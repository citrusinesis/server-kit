//! # server-kit
//!
//! Shared utilities for `server-kit-rest` and `server-kit-grpc`.

mod config;
mod environment;
mod logging;

pub use config::{ConfigBuilder, ConfigError, ConfigFormat};
pub use environment::Environment;
pub use logging::LogFormat;

#[cfg(feature = "tracing")]
pub use logging::{init_logging, init_logging_from_env};
