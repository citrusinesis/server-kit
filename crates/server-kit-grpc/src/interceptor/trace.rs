//! Tracing layer for gRPC requests.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use tower::{Layer, Service};
use tracing::Instrument;

use super::REQUEST_ID_HEADER;

/// Tracing layer for gRPC requests.
///
/// Creates a span for each request with `method` and `request_id` fields.
/// On completion, logs a single line with status and latency.
#[derive(Clone, Copy, Default)]
pub struct TraceLayer;

impl TraceLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for TraceLayer {
    type Service = TraceService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        TraceService { inner }
    }
}

#[derive(Clone)]
pub struct TraceService<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<http::Request<ReqBody>> for TraceService<S>
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
        let request_id = req
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("-")
            .to_string();

        let method = req.uri().path().to_string();

        let span = tracing::info_span!(
            "grpc",
            method = %method,
            request_id = %request_id,
        );

        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(
            async move {
                let start = Instant::now();
                let result = inner.call(req).await;
                let latency_ms = start.elapsed().as_millis();

                match &result {
                    Ok(response) => {
                        let status = response
                            .headers()
                            .get("grpc-status")
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("0");

                        tracing::info!(status = %status, latency_ms = %latency_ms, "gRPC");
                    }
                    Err(_) => {
                        tracing::error!(latency_ms = %latency_ms, "gRPC error");
                    }
                }

                result
            }
            .instrument(span),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Request as HttpRequest;
    use std::convert::Infallible;
    use tower::ServiceExt;

    #[derive(Clone)]
    struct MockService {
        grpc_status: &'static str,
    }

    impl MockService {
        fn new() -> Self {
            Self { grpc_status: "0" }
        }

        fn with_status(status: &'static str) -> Self {
            Self { grpc_status: status }
        }
    }

    impl<B> Service<HttpRequest<B>> for MockService {
        type Response = http::Response<String>;
        type Error = Infallible;
        type Future = std::future::Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, _req: HttpRequest<B>) -> Self::Future {
            let response = http::Response::builder()
                .header("grpc-status", self.grpc_status)
                .body("ok".to_string())
                .unwrap();

            std::future::ready(Ok(response))
        }
    }

    #[test]
    fn trace_layer_creates_service() {
        let layer = TraceLayer::new();

        fn assert_layer<L: Layer<MockService>>(_: L) {}
        assert_layer(layer);
    }

    #[test]
    fn trace_layer_is_clone() {
        fn assert_clone<T: Clone>() {}
        assert_clone::<TraceLayer>();
    }

    #[tokio::test]
    async fn trace_service_passes_through() {
        let layer = TraceLayer::new();
        let service = layer.layer(MockService::new());

        let req = HttpRequest::builder()
            .uri("/greeter.Greeter/SayHello")
            .header(REQUEST_ID_HEADER, "test-id-123")
            .body(())
            .unwrap();

        let response = service.oneshot(req).await.unwrap();
        assert_eq!(response.headers().get("grpc-status").unwrap(), "0");
    }

    #[tokio::test]
    async fn trace_service_handles_missing_request_id() {
        let layer = TraceLayer::new();
        let service = layer.layer(MockService::new());

        let req = HttpRequest::builder()
            .uri("/greeter.Greeter/SayHello")
            .body(())
            .unwrap();

        let response = service.oneshot(req).await.unwrap();
        assert_eq!(response.headers().get("grpc-status").unwrap(), "0");
    }

    #[tokio::test]
    async fn trace_service_handles_error_status() {
        let layer = TraceLayer::new();
        let service = layer.layer(MockService::with_status("13"));

        let req = HttpRequest::builder()
            .uri("/greeter.Greeter/SayHello")
            .header(REQUEST_ID_HEADER, "test-id")
            .body(())
            .unwrap();

        let response = service.oneshot(req).await.unwrap();
        assert_eq!(response.headers().get("grpc-status").unwrap(), "13");
    }
}
