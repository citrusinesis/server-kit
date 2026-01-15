use axum::body::Body;
use axum::http::{header, Request, Response};
use axum::response::IntoResponse;
use http_body_util::BodyExt;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

use crate::error::ErrorResponse;
use crate::Environment;

/// Layer that converts error responses to JSON format.
#[derive(Clone, Copy)]
pub struct JsonErrorLayer {
    environment: Environment,
}

impl JsonErrorLayer {
    pub fn new(environment: Environment) -> Self {
        Self { environment }
    }
}

impl<S> Layer<S> for JsonErrorLayer {
    type Service = JsonErrorService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        JsonErrorService {
            inner,
            environment: self.environment,
        }
    }
}

#[derive(Clone)]
pub struct JsonErrorService<S> {
    inner: S,
    environment: Environment,
}

impl<S, B> Service<Request<Body>> for JsonErrorService<S>
where
    S: Service<Request<Body>, Response = Response<B>> + Clone + Send + 'static,
    S::Future: Send,
    B: axum::body::HttpBody<Data = axum::body::Bytes> + Send + 'static,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    type Response = Response<Body>;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        let is_production = self.environment.is_production();

        Box::pin(async move {
            let response = inner.call(req).await?;

            if !response.status().is_client_error() && !response.status().is_server_error() {
                let (parts, body) = response.into_parts();
                return Ok(Response::from_parts(parts, Body::new(body)));
            }

            let is_json = response
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .is_some_and(|v| v.contains("application/json"));

            if is_json {
                let (parts, body) = response.into_parts();
                return Ok(Response::from_parts(parts, Body::new(body)));
            }

            let status = response.status();
            let (parts, body) = response.into_parts();
            let bytes = body
                .collect()
                .await
                .map(|b| b.to_bytes())
                .unwrap_or_default();
            let body_text = String::from_utf8_lossy(&bytes);

            let message = if body_text.is_empty() || is_production {
                status.canonical_reason().unwrap_or("Error")
            } else {
                &body_text
            };

            let error = ErrorResponse::from_status(status, message);
            let mut response = (status, axum::Json(error)).into_response();
            *response.headers_mut() = parts.headers;
            response
                .headers_mut()
                .insert(header::CONTENT_TYPE, "application/json".parse().unwrap());

            Ok(response)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    #[tokio::test]
    async fn passes_through_success_responses() {
        let app = Router::new()
            .route("/", get(|| async { "OK" }))
            .layer(JsonErrorLayer::new(Environment::Development));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn converts_error_to_json() {
        let app = Router::new()
            .route("/", get(|| async { StatusCode::NOT_FOUND }))
            .layer(JsonErrorLayer::new(Environment::Development));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
    }

    #[tokio::test]
    async fn preserves_existing_json_responses() {
        let app = Router::new()
            .route(
                "/",
                get(|| async {
                    (
                        StatusCode::BAD_REQUEST,
                        [(header::CONTENT_TYPE, "application/json")],
                        r#"{"custom":"error"}"#,
                    )
                }),
            )
            .layer(JsonErrorLayer::new(Environment::Development));

        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("custom"));
    }
}
