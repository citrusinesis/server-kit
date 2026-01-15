mod json_error;
#[cfg(feature = "ratelimit")]
mod ratelimit;
mod trace;

use axum::http::StatusCode;
use axum::Router;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;

#[cfg(feature = "compression")]
use tower_http::compression::CompressionLayer;

#[cfg(feature = "cors")]
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::ServerConfig;
use trace::DefaultTraceLayer;

pub use json_error::JsonErrorLayer;
#[cfg(feature = "ratelimit")]
pub use ratelimit::RateLimitLayer;

pub(crate) fn default_layers(router: Router, config: &ServerConfig) -> Router {
    let router = router
        .layer(CatchPanicLayer::new())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(DefaultTraceLayer::new())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            config.request_timeout(),
        ));

    #[cfg(feature = "compression")]
    let router = router.layer(CompressionLayer::new());

    #[cfg(feature = "cors")]
    let router = {
        if config.cors_origins.is_empty() {
            router
        } else {
            let origins: Vec<_> = config
                .cors_origins
                .iter()
                .filter_map(|s| s.parse().ok())
                .collect();
            router.layer(CorsLayer::new().allow_origin(AllowOrigin::list(origins)))
        }
    };

    router.layer(JsonErrorLayer::new(config.environment))
}
