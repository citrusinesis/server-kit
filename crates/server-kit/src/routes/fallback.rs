use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::error::ErrorResponse;

/// Returns a JSON 404 response for unmatched routes.
pub async fn fallback_handler() -> Response {
    let body = ErrorResponse::from_status(StatusCode::NOT_FOUND, "The requested resource was not found");
    (StatusCode::NOT_FOUND, axum::Json(body)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fallback_returns_404() {
        let response = fallback_handler().await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
