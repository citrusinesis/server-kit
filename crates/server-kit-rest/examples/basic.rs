use axum::{
    extract::Path,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use server_kit_rest::{HttpError, RouterExt, ServerConfig, StatusCode};

#[derive(Serialize)]
struct User {
    id: i64,
    name: String,
}

#[derive(Deserialize)]
struct CreateUser {
    name: String,
}

#[derive(Debug)]
enum AppError {
    NotFound,
}

impl HttpError for AppError {
    fn status_code(&self) -> StatusCode {
        StatusCode::NOT_FOUND
    }

    fn message(&self) -> &str {
        "User not found"
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        self.into_http_response()
    }
}

async fn get_user(Path(id): Path<i64>) -> Result<Json<User>, AppError> {
    let user = (id == 1)
        .then(|| User {
            id: 1,
            name: "Alice".into(),
        })
        .ok_or(AppError::NotFound)?;

    Ok(Json(user))
}

async fn create_user(Json(payload): Json<CreateUser>) -> Json<User> {
    Json(User {
        id: 2,
        name: payload.name,
    })
}

fn api_routes() -> Router {
    Router::new()
        .route("/users/{id}", get(get_user))
        .route("/users", post(create_user))
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
        environment = ?config.environment,
        "Starting server"
    );

    Router::new()
        .nest("/api", api_routes())
        .with_health_check()
        .with_fallback()
        .with_default_layers(&config)
        .serve(&config)
        .await?;

    Ok(())
}
