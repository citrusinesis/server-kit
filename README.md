# server-kit

A thin utility crate for reducing axum server boilerplate.

## Philosophy

- **Minimal implementation**: Reuse well-established crates
- **Thin wrapper**: Don't hide axum/tower APIs
- **Opt-in features**: Include only what you need via feature flags

## Installation

```toml
[dependencies]
server-kit = { version = "0.1", features = ["full"] }
```

### Features

| Feature       | Description               | Default |
| ------------- | ------------------------- | ------- |
| `tracing`     | Request tracing layer     | Yes     |
| `compression` | gzip/br compression       | Yes     |
| `cors`        | CORS layer                | No      |
| `metrics`     | Prometheus metrics        | No      |
| `ratelimit`   | Rate limiting             | No      |
| `full`        | All features              | No      |

## Quick Start

```rust
use axum::{Router, routing::get};
use server_kit::{RouterExt, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: ServerConfig = ServerConfig::builder()
        .with_dotenv()
        .with_logging_from_env()
        .build()?;

    Router::new()
        .route("/", get(|| async { "Hello!" }))
        .serve(&config)
        .await?;

    Ok(())
}
```

With default layers:

```rust
use axum::Router;
use server_kit::{RouterExt, ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: ServerConfig = ServerConfig::builder()
        .with_dotenv()
        .with_logging_from_env()
        .build()?;

    Router::new()
        .with_health_check()
        .with_fallback()
        .with_default_layers(&config)
        .serve(&config)
        .await?;

    Ok(())
}
```

## API Reference

### ServerConfig::builder()

Creates a configuration builder.

```rust
use server_kit::{ServerConfig, ServerError};

let config: ServerConfig = ServerConfig::builder()
    .with_dotenv()
    .with_logging_from_env()
    .build()?;
```

| Method                    | Description                                         |
| ------------------------- | --------------------------------------------------- |
| `with_dotenv()`           | Load `.env` file from current directory             |
| `with_config_file(path)`  | Load config file (format auto-detected by extension)|
| `with_logging_from_env()` | Configure logging from env vars (`LOG_FORMAT`, `RUST_LOG`) |
| `build::<T>()`            | Build and return config as type `T`                 |

#### Supported Config Formats

| Extension        | Format | Behavior                              |
| ---------------- | ------ | ------------------------------------- |
| `.env`           | dotenv | Load as environment variables         |
| `.toml`          | TOML   | Parse as ServerConfig                 |
| `.yaml` / `.yml` | YAML   | Parse as ServerConfig                 |
| `.json`          | JSON   | Parse as ServerConfig                 |

```rust
// Combining multiple config files
let config: ServerConfig = ServerConfig::builder()
    .with_dotenv()                    // .env
    .with_config_file(".env.local")   // .env.local (additional env vars)
    .with_config_file("config.yaml")  // main config (last config file wins)
    .build()?;
```

- `.env` files are loaded into environment variables in order
- Config files (toml/yaml/json) use only the last one specified
- Environment variables override config file values

#### Custom Config Extension

Implement `AsRef<ServerConfig>` to use custom configuration types.

```rust
use serde::Deserialize;
use server_kit::{ServerConfig, RouterExt};
use axum::Router;

#[derive(Deserialize)]
struct AppConfig {
    #[serde(flatten)]
    server: ServerConfig,
    database_url: String,
}

impl AsRef<ServerConfig> for AppConfig {
    fn as_ref(&self) -> &ServerConfig {
        &self.server
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: AppConfig = ServerConfig::builder()
        .with_config_file("config.toml")
        .build()?;

    println!("DB: {}", config.database_url);

    Router::new()
        .with_default_layers(&config)
        .serve(&config)
        .await?;

    Ok(())
}
```

### RouterExt

Extension trait for `Router` providing server-kit functionality.

```rust
use server_kit::{RouterExt, ServerConfig};

Router::new()
    .route("/", get(handler))
    .with_health_check()      // Add /health endpoint
    .with_fallback()          // JSON 404 handler
    .with_default_layers(&config)
    .serve(&config)
    .await?;
```

### ServerConfig

Server configuration struct.

| Environment Variable               | Default       | Description                                   |
| ---------------------------------- | ------------- | --------------------------------------------- |
| `APP_ENV`/`RUST_ENV`/`ENVIRONMENT` | `development` | Environment mode (`production` or `development`) |
| `HOST`                             | `0.0.0.0`     | Bind host                                     |
| `PORT`                             | `3000`        | Port                                          |
| `REQUEST_TIMEOUT_SECS`             | `30`          | Request timeout in seconds                    |
| `CORS_ORIGINS`                     | `[]`          | Allowed origins (requires `cors` feature)     |

**Environment priority**: `ENVIRONMENT` > `APP_ENV` > `RUST_ENV` (case-insensitive)

```rust
pub struct ServerConfig {
    pub environment: Environment,  // Production mode hides error details
    pub host: String,
    pub port: u16,
    pub request_timeout_secs: u64,
    pub cors_origins: Vec<String>,
}
```

#### Config File Example

**config.toml**

```toml
environment = "production"
host = "0.0.0.0"
port = 8080
request_timeout_secs = 60
cors_origins = ["https://example.com"]  # requires cors feature
```

### with_default_layers

Applies commonly used middleware via the `RouterExt` trait.

```rust
use server_kit::RouterExt;

let app = Router::new()
    .route("/", get(handler))
    .with_default_layers(&config);
```

Included layers:

1. `CatchPanicLayer` - Converts panics to 500 responses
2. `RequestIdLayer` - Generates/propagates X-Request-Id header
3. `TraceLayer` - Request/response logging
4. `TimeoutLayer` - Request timeout (`request_timeout_secs`)
5. `CompressionLayer` - Response compression (feature: `compression`)
6. `CorsLayer` - CORS support (feature: `cors`, when `cors_origins` configured)
7. `JsonErrorLayer` - Converts error responses to JSON

### Rate Limiting (feature: `ratelimit`)

Add rate limiting to your routes.

```rust
use server_kit::RouterExt;
use std::time::Duration;

let app = Router::new()
    .route("/api", get(handler))
    .with_rate_limit(100, Duration::from_secs(1));  // 100 req/sec
```

Or use `RateLimitLayer` directly:

```rust
use server_kit::RateLimitLayer;

let app = Router::new()
    .route("/api", get(handler))
    .layer(RateLimitLayer::per_second(100))
    .layer(RateLimitLayer::per_minute(1000));
```

Rate limited responses return:

```json
{ "code": "TOO_MANY_REQUESTS", "message": "Rate limit exceeded" }
```

### Health Routes

Provides basic health check endpoints.

```rust
use server_kit::RouterExt;

let app = Router::new().with_health_check();  // same as .merge(health_routes())
```

| Path          | Response                     |
| ------------- | ---------------------------- |
| `GET /health` | `200 OK` - Server is running |

### Fallback Handler

Returns JSON error for non-existent routes.

```rust
use server_kit::RouterExt;

let app = Router::new()
    .route("/", get(handler))
    .with_fallback();  // same as .fallback(fallback_handler)
```

Response:

```json
{ "code": "NOT_FOUND", "message": "The requested resource was not found" }
```

### HttpError Trait

Trait for unified error response format.

```rust
use server_kit::{HttpError, StatusCode};
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
enum AppError {
    NotFound,
    InvalidInput(String),
}

impl HttpError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::NotFound => "Resource not found",
            Self::InvalidInput(msg) => msg,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        self.into_http_response()
    }
}

// Usage in handlers
async fn get_user(Path(id): Path<i64>) -> Result<Json<User>, AppError> {
    find_user(id).await.ok_or(AppError::NotFound)
}
```

Response format:

```json
{ "code": "NOT_FOUND", "message": "Resource not found" }
```

### Metrics (feature: `metrics`)

Collect Prometheus metrics.

```rust
use server_kit::RouterExt;

let app = Router::new()
    .route("/", get(handler))
    .with_default_layers(&config)
    .with_metrics();  // default path: /metrics

// Or with custom path
let app = Router::new()
    .route("/", get(handler))
    .with_metrics_at("/internal/metrics");
```

Collected metrics:

- `http_requests_total` - Request count (method, path, status)
- `http_request_duration_seconds` - Response time (method, path, status)

## Workspace Crates

### server-kit-auth

Authentication utilities for axum servers.

```toml
[dependencies]
server-kit-auth = { version = "0.1" }
```

See [server-kit-auth](./crates/server-kit-auth) for details.

## Full Example

```rust
use axum::{Router, routing::{get, post}, Json, extract::Path};
use axum::response::{IntoResponse, Response};
use server_kit::{RouterExt, ServerConfig, HttpError, StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct User { id: i64, name: String }

#[derive(Deserialize)]
struct CreateUser { name: String }

#[derive(Debug)]
enum AppError { NotFound }

impl HttpError for AppError {
    fn status_code(&self) -> StatusCode { StatusCode::NOT_FOUND }
    fn message(&self) -> &str { "User not found" }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response { self.into_http_response() }
}

async fn get_user(Path(id): Path<i64>) -> Result<Json<User>, AppError> {
    (id == 1)
        .then(|| Json(User { id: 1, name: "Alice".into() }))
        .ok_or(AppError::NotFound)
}

async fn create_user(Json(payload): Json<CreateUser>) -> Json<User> {
    Json(User { id: 2, name: payload.name })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: ServerConfig = ServerConfig::builder()
        .with_dotenv()
        .with_logging_from_env()
        .build()?;

    Router::new()
        .route("/users/{id}", get(get_user))
        .route("/users", post(create_user))
        .with_health_check()
        .with_fallback()
        .with_default_layers(&config)
        .serve(&config)
        .await?;

    Ok(())
}
```

## License

MIT
