//! Metrics layer for gRPC requests.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use metrics::{counter, histogram};
use tower::{Layer, Service};

/// Metrics layer for gRPC requests.
///
/// Records:
/// - `grpc_requests_total` - Counter of total requests
/// - `grpc_request_duration_seconds` - Histogram of request durations
///
/// # Example
///
/// ```ignore
/// use server_kit_grpc::interceptor::MetricsLayer;
/// use tonic::transport::Server;
///
/// Server::builder()
///     .layer(MetricsLayer::new())
///     .add_service(my_service)
///     .serve(addr)
///     .await?;
/// ```
#[derive(Clone, Copy, Default)]
pub struct MetricsLayer;

impl MetricsLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for MetricsLayer {
    type Service = MetricsService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MetricsService { inner }
    }
}

/// Metrics service wrapper.
#[derive(Clone)]
pub struct MetricsService<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<http::Request<ReqBody>> for MetricsService<S>
where
    S: Service<http::Request<ReqBody>, Response = http::Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<ReqBody>) -> Self::Future {
        let method = req.uri().path().to_string();
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let start = Instant::now();
            let result = inner.call(req).await;
            let duration = start.elapsed();

            let status = match &result {
                Ok(response) => response
                    .headers()
                    .get("grpc-status")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("0")
                    .to_string(),
                Err(_) => "error".to_string(),
            };

            counter!("grpc_requests_total", "method" => method.clone(), "status" => status.clone())
                .increment(1);
            histogram!("grpc_request_duration_seconds", "method" => method)
                .record(duration.as_secs_f64());

            result
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_layer_creates_service() {
        let layer = MetricsLayer::new();

        // Type check - layer should implement Layer trait
        fn assert_layer<L: Layer<DummyService>>(_: L) {}
        assert_layer(layer);
    }

    struct DummyService;
}
