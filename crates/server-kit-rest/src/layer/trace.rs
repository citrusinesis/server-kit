use axum::http::Request;
use tower::Layer;
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::{DefaultOnResponse, MakeSpan, TraceLayer},
    LatencyUnit,
};
use tracing::{Level, Span};

/// Custom span maker that includes request ID and useful request info.
#[derive(Clone, Copy)]
pub struct RequestSpan;

impl<B> MakeSpan<B> for RequestSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let request_id = request
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("-");

        tracing::info_span!(
            "http",
            method = %request.method(),
            path = %request.uri().path(),
            request_id = %request_id,
        )
    }
}

/// Inner type alias for the configured TraceLayer.
pub type InnerTraceLayer =
    TraceLayer<SharedClassifier<ServerErrorsAsFailures>, RequestSpan, (), DefaultOnResponse>;

/// Pre-configured TraceLayer with request ID and latency logging.
///
/// - Uses `RequestSpan` for span creation (includes request ID)
/// - Logs responses at INFO level with latency in microseconds
#[derive(Clone, Copy)]
pub struct DefaultTraceLayer;

impl DefaultTraceLayer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultTraceLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for DefaultTraceLayer {
    type Service = <InnerTraceLayer as Layer<S>>::Service;

    fn layer(&self, inner: S) -> Self::Service {
        TraceLayer::new_for_http()
            .make_span_with(RequestSpan)
            .on_request(())
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .latency_unit(LatencyUnit::Micros),
            )
            .layer(inner)
    }
}
