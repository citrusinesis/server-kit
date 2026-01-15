//! Router extension traits for axum-like API.

use axum::Router;

use crate::routes::{fallback_handler, health_routes};
use crate::ServerConfig;

/// Extension trait for Router that provides server-kit functionality.
///
/// This trait makes server-kit feel like native axum code by providing
/// chainable methods on Router.
///
/// # Example
///
/// ```rust,ignore
/// use axum::{Router, routing::get};
/// use server_kit::RouterExt;
///
/// let app = Router::new()
///     .route("/api/users", get(list_users))
///     .with_health_check()
///     .with_fallback()
///     .with_default_layers(&config);
///
/// app.serve(&config).await?;
/// ```
pub trait RouterExt: Sized {
    /// Adds health check routes (`/health` and `/ready`).
    ///
    /// Equivalent to `.merge(health_routes())`.
    fn with_health_check(self) -> Self;

    /// Adds a JSON 404 fallback handler for unmatched routes.
    ///
    /// Equivalent to `.fallback(fallback_handler)`.
    fn with_fallback(self) -> Self;

    /// Applies the default middleware stack.
    ///
    /// Layers applied (innermost to outermost):
    /// - `CatchPanicLayer` - Converts panics to 500 responses
    /// - `SetRequestIdLayer` / `PropagateRequestIdLayer` - X-Request-Id handling
    /// - `TraceLayer` - Request/response logging with latency
    /// - `TimeoutLayer` - Request timeout from config
    /// - `CompressionLayer` - Response compression (feature: `compression`)
    /// - `CorsLayer` - CORS support (feature: `cors`, when origins configured)
    /// - `JsonErrorLayer` - Converts error responses to JSON (outermost)
    fn with_default_layers(self, config: &impl AsRef<ServerConfig>) -> Self;

    /// Adds Prometheus metrics collection and endpoint.
    ///
    /// This adds:
    /// - `MetricsLayer` to collect HTTP request metrics
    /// - A `/metrics` endpoint for Prometheus scraping
    ///
    /// Requires feature: `metrics`
    #[cfg(feature = "metrics")]
    fn with_metrics(self) -> Self;

    /// Adds Prometheus metrics with a custom endpoint path.
    ///
    /// Requires feature: `metrics`
    #[cfg(feature = "metrics")]
    fn with_metrics_at(self, path: impl Into<String>) -> Self;

    /// Serve the router with graceful shutdown support.
    ///
    /// Handles `SIGINT` (Ctrl+C) and `SIGTERM` signals, waiting for
    /// in-flight requests to complete before shutting down.
    fn serve(
        self,
        config: &(impl AsRef<ServerConfig> + Sync),
    ) -> impl std::future::Future<Output = Result<(), crate::ServerError>> + Send;
}

impl RouterExt for Router {
    fn with_health_check(self) -> Self {
        self.merge(health_routes())
    }

    fn with_fallback(self) -> Self {
        self.fallback(fallback_handler)
    }

    fn with_default_layers(self, config: &impl AsRef<ServerConfig>) -> Self {
        crate::layer::default_layers(self, config.as_ref())
    }

    #[cfg(feature = "metrics")]
    fn with_metrics(self) -> Self {
        crate::metrics::Metrics::new().wrap(self)
    }

    #[cfg(feature = "metrics")]
    fn with_metrics_at(self, path: impl Into<String>) -> Self {
        crate::metrics::Metrics::new().path(path).wrap(self)
    }

    async fn serve(
        self,
        config: &(impl AsRef<ServerConfig> + Sync),
    ) -> Result<(), crate::ServerError> {
        crate::server::serve_router(self, config).await
    }
}
