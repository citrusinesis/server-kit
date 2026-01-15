//! Simplest possible server using the builder pattern.

use axum::{routing::get, Router};
use server_kit_rest::{init_logging_from_env, RouterExt, ServerConfig};

async fn hello() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: ServerConfig = ServerConfig::builder().with_dotenv().build()?;

    init_logging_from_env();

    Router::new()
        .route("/", get(hello))
        .serve(&config)
        .await?;

    Ok(())
}
