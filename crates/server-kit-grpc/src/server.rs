//! Server extension traits for tonic.

use std::net::SocketAddr;
use tonic::transport::server::Router;

use crate::config::GrpcServerConfig;
use crate::error::ServerError;
use crate::interceptor::{RequestIdLayer, TraceLayer};

/// Extension trait for `tonic::transport::Server`.
pub trait ServerExt: Sized {
    type WithLayers;

    /// Applies the default middleware stack (RequestIdLayer + TraceLayer).
    fn with_default_layers(self) -> Self::WithLayers;
}

impl<L> ServerExt for tonic::transport::server::Server<L> {
    type WithLayers = tonic::transport::server::Server<
        tower::layer::util::Stack<TraceLayer, tower::layer::util::Stack<RequestIdLayer, L>>,
    >;

    fn with_default_layers(self) -> Self::WithLayers {
        self.layer(RequestIdLayer::new()).layer(TraceLayer::new())
    }
}

/// Extension trait for `tonic::transport::server::Router`.
pub trait RouterExt<L>: Sized {
    /// Serve the router using config with graceful shutdown.
    fn serve_with(
        self,
        config: &(impl AsRef<GrpcServerConfig> + Sync),
    ) -> impl std::future::Future<Output = Result<(), ServerError>> + Send;

    /// Serve at a specific address with graceful shutdown.
    fn serve_at(
        self,
        addr: SocketAddr,
    ) -> impl std::future::Future<Output = Result<(), ServerError>> + Send;
}

impl<L> RouterExt<L> for Router<L>
where
    L: tower::Layer<tonic::service::Routes> + Clone + Send + 'static,
    L::Service: tower::Service<
            http::Request<tonic::body::BoxBody>,
            Response = http::Response<tonic::body::BoxBody>,
        > + Clone
        + Send
        + 'static,
    <L::Service as tower::Service<http::Request<tonic::body::BoxBody>>>::Future: Send,
    <L::Service as tower::Service<http::Request<tonic::body::BoxBody>>>::Error:
        Into<Box<dyn std::error::Error + Send + Sync>> + Send,
{
    async fn serve_with(
        self,
        config: &(impl AsRef<GrpcServerConfig> + Sync),
    ) -> Result<(), ServerError> {
        let addr: SocketAddr = config
            .as_ref()
            .socket_addr()
            .map_err(ServerError::InvalidAddress)?;
        self.serve_at(addr).await
    }

    async fn serve_at(self, addr: SocketAddr) -> Result<(), ServerError> {
        tracing::info!(addr = %addr, "gRPC server listening");

        self.serve_with_shutdown(addr, shutdown_signal())
            .await
            .map_err(ServerError::Transport)?;

        tracing::info!("gRPC server shutdown complete");
        Ok(())
    }
}

/// Wait for shutdown signals (SIGINT, SIGTERM).
pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received SIGINT, starting graceful shutdown...");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, starting graceful shutdown...");
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_socket_addr_parsing() {
        let config = GrpcServerConfig {
            host: "127.0.0.1".to_string(),
            port: 50051,
            ..Default::default()
        };

        let addr: SocketAddr = config.socket_addr().unwrap();
        assert_eq!(addr.to_string(), "127.0.0.1:50051");
    }

    #[test]
    fn config_socket_addr_ipv6() {
        let config = GrpcServerConfig {
            host: "[::1]".to_string(),
            port: 50051,
            ..Default::default()
        };

        let addr: SocketAddr = config.socket_addr().unwrap();
        assert_eq!(addr.to_string(), "[::1]:50051");
    }

    #[test]
    fn config_socket_addr_invalid() {
        let config = GrpcServerConfig {
            host: "not-an-ip".to_string(),
            port: 50051,
            ..Default::default()
        };

        assert!(config.socket_addr().is_err());
    }
}
