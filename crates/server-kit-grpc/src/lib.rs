//! # server-kit-grpc
//!
//! A thin extension crate for tonic gRPC servers and clients.
//!
//! This crate follows the same design philosophy as `server-kit` for HTTP servers:
//! **extend native tonic code with chainable methods, not replace it**.
//!
//! ## Quick Start - Server
//!
//! ```ignore
//! use server_kit_grpc::{GrpcServerConfig, RouterExt, ServerExt};
//! use tonic::transport::Server;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config: GrpcServerConfig = GrpcServerConfig::builder()
//!         .with_dotenv()
//!         .build()?;
//!
//!     Server::builder()
//!         .with_default_layers()  // ServerExt method
//!         .add_service(MyServiceServer::new(my_impl))
//!         .serve_with(&config)    // RouterExt method
//!         .await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Quick Start - Client
//!
//! ```ignore
//! use server_kit_grpc::{ChannelConfig, ChannelExt};
//! use tonic::transport::Channel;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config: ChannelConfig = ChannelConfig::builder()
//!         .endpoint("http://localhost:50051")
//!         .build()?;
//!
//!     let channel = Channel::connect(&config).await?;  // ChannelExt method
//!     let mut client = MyServiceClient::new(channel);
//!
//!     let response = client.my_method(MyRequest { ... }).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! - `tracing` - Enable logging initialization (default)
//! - `health` - Enable gRPC health checking service (default)
//! - `tls` - Enable TLS support
//! - `metrics` - Enable Prometheus metrics collection
//! - `reflection` - Enable gRPC server reflection
//! - `full` - Enable all features

mod channel;
pub mod config;
mod error;
pub mod interceptor;
mod request_ext;
mod server;

#[cfg(feature = "health")]
pub mod health;

#[cfg(feature = "reflection")]
pub mod reflection;

pub use config::{ChannelConfig, ChannelConfigBuilder, ConfigBuilder, ConfigError, Environment, GrpcServerConfig};
pub use channel::ChannelExt;
pub use server::{RouterExt, ServerExt, shutdown_signal};
pub use request_ext::{headers, HeaderKey, RequestExt};
pub use error::{Error, GrpcError, ServerError};

#[cfg(feature = "health")]
pub use health::{health_service, HealthReporter, ServingStatus};

pub use interceptor::{
    bearer_auth, request_id_interceptor, AuthInterceptor, RequestIdInterceptor, RequestIdLayer,
    TokenValidator, TraceLayer, REQUEST_ID_HEADER,
};

#[cfg(feature = "metrics")]
pub use interceptor::MetricsLayer;

#[cfg(feature = "reflection")]
pub use reflection::{reflection_service, reflection_service_v1alpha};

pub use tonic::{Code, Request, Response, Status};
pub use server_kit::LogFormat;

#[cfg(feature = "tracing")]
pub use server_kit::{init_logging, init_logging_from_env};
