# server-kit-rest

A thin utility crate for reducing axum server boilerplate.

## Installation

```toml
[dependencies]
server-kit-rest = { version = "0.1", features = ["full"] }
```

### Features

| Feature       | Description               | Default |
| ------------- | ------------------------- | ------- |
| `tracing`     | Request tracing layer     | Yes     |
| `compression` | gzip/br compression       | Yes     |
| `cors`        | CORS layer                | No      |
| `metrics`     | Prometheus metrics        | No      |
| `ratelimit`   | Rate limiting             | No      |
| `auth`        | Authentication middleware | No      |
| `jwt`         | JWT authentication        | No      |
| `full`        | All features              | No      |

## Quick Start

```rust
use axum::{Router, routing::get};
use server_kit_rest::{RouterExt, ServerConfig};

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
Router::new()
    .with_health_check()
    .with_fallback()
    .with_default_layers(&config)
    .serve(&config)
    .await?;
```

## API Reference

### ServerConfig

Server configuration loaded from environment variables or config files.

```rust
let config: ServerConfig = ServerConfig::builder()
    .with_dotenv()
    .with_logging_from_env()
    .build()?;
```

| Environment Variable               | Default       | Description                          |
| ---------------------------------- | ------------- | ------------------------------------ |
| `APP_ENV`/`RUST_ENV`/`ENVIRONMENT` | `development` | Environment mode                     |
| `HOST`                             | `0.0.0.0`     | Bind host                            |
| `PORT`                             | `3000`        | Port                                 |
| `REQUEST_TIMEOUT_SECS`             | `30`          | Request timeout in seconds           |
| `CORS_ORIGINS`                     | `[]`          | Allowed origins (requires `cors`)    |

#### Custom Config Extension

Use `AsRef<ServerConfig>` for custom configuration types:

```rust
use serde::Deserialize;
use server_kit_rest::{ServerConfig, RouterExt};

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

let config: AppConfig = ServerConfig::builder()
    .with_config_file("config.toml")
    .build()?;

Router::new()
    .with_default_layers(&config)  // Works with AppConfig
    .serve(&config)
    .await?;
```

### RouterExt

Extension trait for `Router`.

```rust
Router::new()
    .route("/", get(handler))
    .with_health_check()              // Add /health endpoint
    .with_fallback()                  // JSON 404 handler
    .with_default_layers(&config)     // Apply standard middleware
    .serve(&config)                   // Start server
    .await?;
```

### with_default_layers

Applies commonly used middleware:

1. `CatchPanicLayer` - Converts panics to 500 responses
2. `RequestIdLayer` - Generates/propagates X-Request-Id header
3. `TraceLayer` - Request/response logging
4. `TimeoutLayer` - Request timeout
5. `CompressionLayer` - Response compression (feature: `compression`)
6. `CorsLayer` - CORS support (feature: `cors`)
7. `JsonErrorLayer` - Converts error responses to JSON

### Rate Limiting (feature: `ratelimit`)

```rust
Router::new()
    .route("/api", get(handler))
    .with_rate_limit(100, Duration::from_secs(1));  // 100 req/sec
```

Response when rate limited:

```json
{ "code": "TOO_MANY_REQUESTS", "message": "Rate limit exceeded" }
```

### Metrics (feature: `metrics`)

```rust
Router::new()
    .with_default_layers(&config)
    .with_metrics();  // Exposes /metrics endpoint
```

Collected metrics:

- `http_requests_total` - Request count (method, path, status)
- `http_request_duration_seconds` - Response time

### Authentication (feature: `auth`)

Custom token validation:

```rust
use server_kit_rest::auth::{AuthExt, TokenValidator, AuthError};

#[derive(Clone)]
struct MyValidator;

impl TokenValidator for MyValidator {
    fn validate(&self, token: &str) -> Result<(), AuthError> {
        if token == "valid" {
            Ok(())
        } else {
            Err(AuthError::InvalidToken("bad token".into()))
        }
    }
}

Router::new()
    .route("/protected", get(handler))
    .with_auth(MyValidator);
```

### JWT Authentication (feature: `jwt`)

```rust
use server_kit_rest::auth::{AuthExt, JwtConfig, Claims};

let jwt = JwtConfig::new("your-secret-key");

// Create token
let claims = Claims::new("user-123", 3600);  // expires in 1 hour
let token = jwt.encode(&claims)?;

// Protect routes
Router::new()
    .route("/protected", get(handler))
    .with_jwt_auth(&jwt);
```

JwtConfig can be loaded from config files:

```rust
#[derive(Deserialize)]
struct AppConfig {
    #[serde(flatten)]
    server: ServerConfig,
    jwt: JwtConfig,  // { "secret": "..." }
}

impl AsRef<JwtConfig> for AppConfig {
    fn as_ref(&self) -> &JwtConfig {
        &self.jwt
    }
}

let config: AppConfig = ServerConfig::builder()
    .with_config_file("config.toml")
    .build()?;

Router::new()
    .with_jwt_auth(&config)  // Works with AppConfig
    .serve(&config)
    .await?;
```

### HttpError Trait

Trait for unified error response format:

```rust
use server_kit_rest::{HttpError, StatusCode};
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
```

Response format:

```json
{ "code": "NOT_FOUND", "message": "Resource not found" }
```

## Full Example

```rust
use axum::{Router, routing::{get, post}, Json, extract::Path};
use axum::response::{IntoResponse, Response};
use server_kit_rest::{RouterExt, ServerConfig, HttpError, StatusCode};
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
