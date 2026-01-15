use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;

/// Returns a router with `GET /health` endpoint.
pub fn health_routes() -> Router {
    Router::new().route("/health", get(|| async { StatusCode::OK }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_endpoint_returns_ok() {
        let app = health_routes();
        let response = app
            .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
