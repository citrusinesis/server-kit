use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use axum::response::IntoResponse;
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use std::future::Future;
use std::num::NonZeroU32;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use tower::{Layer, Service};

/// Rate limiter layer using the governor crate.
#[derive(Clone)]
pub struct RateLimitLayer {
    limiter: Arc<DefaultDirectRateLimiter>,
}

impl RateLimitLayer {
    /// Create a new rate limiter with the given quota.
    ///
    /// # Arguments
    ///
    /// * `num_requests` - Maximum number of requests allowed in the period
    /// * `per_duration` - The time period for the rate limit
    pub fn new(num_requests: u32, per_duration: Duration) -> Self {
        let quota = Quota::with_period(per_duration)
            .expect("invalid duration")
            .allow_burst(NonZeroU32::new(num_requests).expect("num_requests must be > 0"));

        Self {
            limiter: Arc::new(RateLimiter::direct(quota)),
        }
    }

    /// Create a rate limiter allowing `n` requests per second.
    pub fn per_second(n: u32) -> Self {
        Self::new(n, Duration::from_secs(1))
    }

    /// Create a rate limiter allowing `n` requests per minute.
    pub fn per_minute(n: u32) -> Self {
        Self::new(n, Duration::from_secs(60))
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitService {
            inner,
            limiter: Arc::clone(&self.limiter),
        }
    }
}

/// Rate limiter service.
#[derive(Clone)]
pub struct RateLimitService<S> {
    inner: S,
    limiter: Arc<DefaultDirectRateLimiter>,
}

impl<S> Service<Request<Body>> for RateLimitService<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let limiter = Arc::clone(&self.limiter);
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            if limiter.check().is_err() {
                let body = serde_json::json!({
                    "code": "TOO_MANY_REQUESTS",
                    "message": "Rate limit exceeded"
                });
                return Ok((StatusCode::TOO_MANY_REQUESTS, axum::Json(body)).into_response());
            }

            inner.call(req).await
        })
    }
}
