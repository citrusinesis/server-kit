//! # server-kit-core
//!
//! Shared utilities for server-kit crates.
//!
//! This crate provides common functionality used by both `server-kit` (HTTP/Axum)
//! and `server-kit-grpc` (gRPC/Tonic).
//!
//! ## Features
//!
//! - `tracing` - Enable logging initialization with tracing-subscriber

mod config;
mod environment;
mod logging;

pub use config::{ConfigBuilder, ConfigError, ConfigFormat};
pub use environment::Environment;
pub use logging::LogFormat;

#[cfg(feature = "tracing")]
pub use logging::{init_logging, init_logging_from_env};
