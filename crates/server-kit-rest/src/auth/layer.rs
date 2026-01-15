use axum::body::Body;
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use axum::Router;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use super::AuthError;

pub trait TokenValidator: Clone + Send + Sync + 'static {
    fn validate(&self, token: &str) -> Result<(), AuthError>;
}

#[derive(Clone)]
pub struct AuthLayer<V> {
    validator: Arc<V>,
}

impl<V: TokenValidator> AuthLayer<V> {
    pub fn new(validator: V) -> Self {
        Self {
            validator: Arc::new(validator),
        }
    }
}

impl<S, V: TokenValidator> Layer<S> for AuthLayer<V> {
    type Service = AuthService<S, V>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthService {
            inner,
            validator: Arc::clone(&self.validator),
        }
    }
}

#[derive(Clone)]
pub struct AuthService<S, V> {
    inner: S,
    validator: Arc<V>,
}

impl<S, V> Service<Request<Body>> for AuthService<S, V>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send,
    V: TokenValidator,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let token = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(|s| s.to_string());

        let validator = Arc::clone(&self.validator);
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            let Some(token) = token else {
                return Ok(AuthError::MissingToken.into_response());
            };

            if let Err(e) = validator.validate(&token) {
                return Ok(e.into_response());
            }

            inner.call(req).await
        })
    }
}

pub trait AuthExt {
    fn with_auth<V: TokenValidator>(self, validator: V) -> Self;

    #[cfg(feature = "jwt")]
    fn with_jwt_auth(self, config: &impl AsRef<super::JwtConfig>) -> Self;
}

impl AuthExt for Router {
    fn with_auth<V: TokenValidator>(self, validator: V) -> Self {
        self.layer(AuthLayer::new(validator))
    }

    #[cfg(feature = "jwt")]
    fn with_jwt_auth(self, config: &impl AsRef<super::JwtConfig>) -> Self {
        self.with_auth(config.as_ref().clone())
    }
}

impl AsRef<super::JwtConfig> for super::JwtConfig {
    fn as_ref(&self) -> &super::JwtConfig {
        self
    }
}
