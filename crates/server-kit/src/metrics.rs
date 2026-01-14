//! Metrics collection and endpoint.

use axum::body::Body;
use axum::extract::MatchedPath;
use axum::http::{header, Request, Response};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use metrics::{counter, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::future::Future;
use std::pin::Pin;
use std::sync::OnceLock;
use std::task::{Context, Poll};
use std::time::Instant;
use tower::{Layer, Service};

static PROMETHEUS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

/// Initialize the metrics recorder.
fn init_metrics() -> &'static PrometheusHandle {
    PROMETHEUS_HANDLE.get_or_init(|| {
        PrometheusBuilder::new()
            .install_recorder()
            .expect("failed to install Prometheus recorder")
    })
}

/// Metrics configuration.
///
/// # Example
///
/// ```rust,ignore
/// use server_kit::Metrics;
///
/// Metrics::new().wrap(
///     Router::new()
///         .nest("/api", api_routes())
///         .layer(default_layers())
/// )
///
/// // With custom path
/// Metrics::new()
///     .path("/internal/metrics")
///     .wrap(router)
/// ```
#[derive(Clone)]
pub struct Metrics {
    path: String,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    /// Create a new Metrics instance with default settings.
    pub fn new() -> Self {
        Self {
            path: "/metrics".to_string(),
        }
    }

    /// Set a custom path for the metrics endpoint.
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    /// Wrap a router with metrics collection and endpoint.
    ///
    /// This adds:
    /// - `MetricsLayer` to collect HTTP request metrics
    /// - A metrics endpoint at the configured path (default: `/metrics`)
    pub fn wrap(self, router: Router) -> Router {
        init_metrics();
        router
            .route(&self.path, get(metrics_handler))
            .layer(MetricsLayer)
    }
}

async fn metrics_handler() -> impl IntoResponse {
    let handle = PROMETHEUS_HANDLE.get().expect("metrics not initialized");

    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        handle.render(),
    )
}

/// Layer that records HTTP request metrics.
#[derive(Clone, Copy, Default)]
struct MetricsLayer;

impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsService { inner }
    }
}

#[derive(Clone)]
struct MetricsService<S> {
    inner: S,
}

impl<S, B> Service<Request<Body>> for MetricsService<S>
where
    S: Service<Request<Body>, Response = Response<B>> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        let method = req.method().to_string();
        let path = req
            .extensions()
            .get::<MatchedPath>()
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| req.uri().path().to_string());
        let start = Instant::now();

        Box::pin(async move {
            let response = inner.call(req).await?;
            let latency = start.elapsed().as_secs_f64();
            let status = response.status().as_u16().to_string();

            let labels = [
                ("method", method),
                ("path", path),
                ("status", status),
            ];

            counter!("http_requests_total", &labels).increment(1);
            histogram!("http_request_duration_seconds", &labels).record(latency);

            Ok(response)
        })
    }
}
