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

    /// Adds rate limiting to the router.
    ///
    /// Limits the number of requests that can be processed concurrently.
    /// Requests exceeding the limit will wait until capacity is available.
    ///
    /// Requires feature: `ratelimit`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use server_kit::RouterExt;
    /// use std::time::Duration;
    ///
    /// let app = Router::new()
    ///     .route("/api", get(handler))
    ///     .with_rate_limit(100, Duration::from_secs(1)); // 100 req/sec
    /// ```
    #[cfg(feature = "ratelimit")]
    fn with_rate_limit(self, num_requests: u64, per_duration: std::time::Duration) -> Self;

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

    #[cfg(feature = "ratelimit")]
    fn with_rate_limit(self, num_requests: u64, per_duration: std::time::Duration) -> Self {
        self.layer(crate::layer::RateLimitLayer::new(
            num_requests as u32,
            per_duration,
        ))
    }

    async fn serve(
        self,
        config: &(impl AsRef<ServerConfig> + Sync),
    ) -> Result<(), crate::ServerError> {
        crate::server::serve_router(self, config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use tower::ServiceExt;

    #[tokio::test]
    async fn with_health_check_adds_health_route() {
        let app = Router::new().with_health_check();

        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn with_fallback_returns_404_json() {
        let app = Router::new().with_fallback();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn with_default_layers_applies_middleware() {
        let config = ServerConfig::default();
        let app = Router::new()
            .route("/", get(|| async { "OK" }))
            .with_default_layers(&config);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("x-request-id", "test-id-123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("x-request-id").unwrap(),
            "test-id-123"
        );
    }

    #[tokio::test]
    async fn chained_extensions() {
        let config = ServerConfig::default();
        let app = Router::new()
            .route("/api", get(|| async { "API" }))
            .with_health_check()
            .with_fallback()
            .with_default_layers(&config);

        let response = app
            .clone()
            .oneshot(Request::builder().uri("/api").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/unknown")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
