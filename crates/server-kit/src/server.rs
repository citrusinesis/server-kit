//! Server utilities.

use crate::ServerConfig;
use axum::Router;
use std::{fmt, io};
use tokio::net::TcpListener;

/// Error type for server operations.
#[derive(Debug)]
pub enum ServerError {
    /// Failed to bind to address.
    Bind(io::Error),
    /// Server runtime error.
    Runtime(io::Error),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bind(e) => write!(f, "Failed to bind to address: {}", e),
            Self::Runtime(e) => write!(f, "Server error: {}", e),
        }
    }
}

impl std::error::Error for ServerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Bind(e) | Self::Runtime(e) => Some(e),
        }
    }
}

/// Serve a router with graceful shutdown support.
pub async fn serve_router(
    router: Router,
    config: &(impl AsRef<ServerConfig> + Sync),
) -> Result<(), ServerError> {
    let config = config.as_ref();
    let addr = config.addr();
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(ServerError::Bind)?;

    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(ServerError::Runtime)?;

    tracing::info!("Server shutdown complete");
    Ok(())
}

/// Waits for shutdown signals (SIGINT or SIGTERM).
async fn shutdown_signal() {
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
