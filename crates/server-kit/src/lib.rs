//! # server-kit
//!
//! A thin utility crate for reducing axum server boilerplate.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use axum::{Router, routing::get};
//! use server_kit::{RouterExt, ServerConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config: ServerConfig = ServerConfig::builder()
//!         .with_dotenv()
//!         .with_logging_from_env()
//!         .build()?;
//!
//!     Router::new()
//!         .route("/", get(|| async { "Hello!" }))
//!         .serve(&config)
//!         .await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## With Default Layers
//!
//! ```rust,ignore
//! use axum::Router;
//! use server_kit::{RouterExt, ServerConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config: ServerConfig = ServerConfig::builder()
//!         .with_dotenv()
//!         .with_logging_from_env()
//!         .build()?;
//!
//!     Router::new()
//!         .with_health_check()
//!         .with_fallback()
//!         .with_default_layers(&config)
//!         .serve(&config)
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

mod config;
mod error;
mod layer;
#[cfg(feature = "tracing")]
mod logging;
#[cfg(feature = "metrics")]
mod metrics;
mod router;
mod routes;
mod server;

pub use config::{ConfigBuilder, ConfigError, Environment, ServerConfig};
pub use error::{ErrorResponse, HttpError};
pub use router::RouterExt;
pub use routes::{fallback_handler, health_routes};
pub use server::ServerError;

#[cfg(feature = "metrics")]
pub use metrics::Metrics;

#[cfg(feature = "ratelimit")]
pub use layer::RateLimitLayer;

#[cfg(feature = "tracing")]
pub use logging::init_logging_from_env;

pub use axum::http::StatusCode;
