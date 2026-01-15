use axum::{routing::get, Json, Router};
use serde::Serialize;
use server_kit_rest::{RouterExt, ServerConfig};

#[derive(Serialize)]
struct Status {
    status: String,
}

async fn status() -> Json<Status> {
    // Custom metrics can be recorded anywhere
    metrics::counter!("status_endpoint_calls").increment(1);

    Json(Status {
        status: "ok".into(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: ServerConfig = ServerConfig::builder()
        .with_dotenv()
        .with_logging_from_env()
        .build()?;

    tracing::info!(
        host = %config.host,
        port = %config.port,
        "Starting server with metrics"
    );

    Router::new()
        .route("/status", get(status))
        .with_health_check()
        .with_fallback()
        .with_default_layers(&config)
        .with_metrics()
        .serve(&config)
        .await?;

    Ok(())
}
