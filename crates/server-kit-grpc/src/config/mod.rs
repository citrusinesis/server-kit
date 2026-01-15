//! Configuration types for gRPC servers and clients.

mod channel;
mod server;

pub use channel::{ChannelConfig, ChannelConfigBuilder};
pub use server::GrpcServerConfig;

// Re-export from core
pub use server_kit::{ConfigBuilder, ConfigError, Environment};
