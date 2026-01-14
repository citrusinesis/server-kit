# server-kit

axum 기반 서버의 보일러플레이트를 줄여주는 얇은 유틸리티 크레이트.

## 철학

- **최소한의 구현**: 이미 잘 만들어진 크레이트들을 재사용
- **얇은 래퍼**: axum/tower의 API를 가리지 않음
- **선택적 사용**: feature flag로 필요한 것만 포함

## 설치

```toml
[dependencies]
server-kit = { version = "0.1", features = ["full"] }
```

### Features

| Feature       | 설명            | 기본값 |
| ------------- | --------------- | ------ |
| `tracing`     | TraceLayer 포함 | ✓      |
| `compression` | gzip/br 압축    | ✓      |
| `cors`        | CORS 레이어     | ✗      |
| `metrics`     | Prometheus 메트릭 | ✗    |
| `full`        | 모든 기능       | ✗      |

## 빠른 시작

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

레이어와 함께 사용:

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

## 제공 기능

### ServerConfig::builder()

설정 빌더를 생성합니다.

```rust
use server_kit::{ServerConfig, ServerError};

let config: ServerConfig = ServerConfig::builder()
    .with_dotenv()
    .with_logging_from_env()
    .build()?;
```

| 메서드 | 설명 |
| ------ | ---- |
| `with_dotenv()` | 현재 디렉토리의 `.env` 파일 로드 |
| `with_config_file(path)` | 설정 파일 로드 (확장자로 형식 자동 감지) |
| `with_logging_from_env()` | 환경변수에서 로깅 설정 (`LOG_FORMAT`, `RUST_LOG`) |
| `build::<T>()` | 설정을 `T` 타입으로 반환 |

#### 지원하는 설정 파일 형식

| 확장자 | 형식 | 동작 |
| ------ | ---- | ---- |
| `.env` | dotenv | 환경변수로 로드 (여러 개 가능) |
| `.toml` | TOML | ServerConfig로 파싱 |
| `.yaml` / `.yml` | YAML | ServerConfig로 파싱 |
| `.json` | JSON | ServerConfig로 파싱 |

```rust
// 여러 설정 파일 조합
let config: ServerConfig = ServerConfig::builder()
    .with_dotenv()                    // .env
    .with_config_file(".env.local")   // .env.local (환경변수 추가)
    .with_config_file("config.yaml")  // 메인 설정 (마지막 설정 파일 사용)
    .build()?;
```

- `.env` 파일들은 순서대로 환경변수에 로드됩니다
- 설정 파일 (toml/yaml/json)은 마지막 것만 사용됩니다
- 환경변수가 설정 파일 값을 오버라이드합니다

#### 커스텀 설정 확장

`AsRef<ServerConfig>`를 구현하면 커스텀 설정을 사용할 수 있습니다.

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

### RouterExt::serve()

Router에 대한 확장 trait으로, graceful shutdown을 지원하는 서버를 시작합니다.

```rust
use server_kit::{RouterExt, ServerConfig};

Router::new()
    .route("/", get(handler))
    .serve(&config)
    .await?;
```

`SIGINT` (Ctrl+C)와 `SIGTERM` 신호를 처리하며, 진행 중인 요청이 완료될 때까지 기다린 후 종료합니다.

### ServerConfig

서버 설정을 담는 구조체입니다.

| 환경변수               | 기본값        | 설명               |
| ---------------------- | ------------- | ------------------ |
| `APP_ENV`/`RUST_ENV`/`ENVIRONMENT` | `development` | 환경 모드 (`production`/`prod` 또는 `development`) |
| `HOST`                 | `0.0.0.0`     | 바인딩 호스트      |
| `PORT`                 | `3000`        | 포트               |
| `REQUEST_TIMEOUT_SECS` | `30`          | 요청 타임아웃 (초) |
| `CORS_ORIGINS`         | `[]`          | 허용 오리진 (`cors` feature 활성화 시 적용) |

**Environment 우선순위**: `ENVIRONMENT` > `APP_ENV` > `RUST_ENV` (대소문자 무관)

```rust
pub struct ServerConfig {
    pub environment: Environment,  // Production 모드에서는 에러 상세 메시지 숨김
    pub host: String,
    pub port: u16,
    pub request_timeout_secs: u64,
    pub cors_origins: Vec<String>,
}
```

#### 설정 파일 예시

**config.toml**
```toml
environment = "production"
host = "0.0.0.0"
port = 8080
request_timeout_secs = 60
cors_origins = ["https://example.com"]  # feature: cors
```

### with_default_layers

`RouterExt` trait을 통해 자주 사용하는 미들웨어를 적용합니다.

```rust
use server_kit::RouterExt;

let app = Router::new()
    .route("/", get(handler))
    .with_default_layers(&config);
```

포함된 레이어:

1. `CatchPanicLayer` - panic을 500 응답으로 변환
2. `RequestIdLayer` - X-Request-Id 헤더 생성/전파
3. `TraceLayer` - 요청/응답 로깅
4. `TimeoutLayer` - 요청 타임아웃 (`request_timeout_secs`)
5. `CompressionLayer` - 응답 압축 (feature: `compression`)
6. `CorsLayer` - CORS (feature: `cors`, `cors_origins` 설정시)
7. `JsonErrorLayer` - 에러 응답을 JSON으로 변환

### fallback_handler

존재하지 않는 경로 접근 시 JSON 에러를 반환합니다.

```rust
use server_kit::RouterExt;

let app = Router::new()
    .route("/", get(handler))
    .with_fallback();  // .fallback(fallback_handler) 와 동일
```

응답:
```json
{"code":"NOT_FOUND","message":"The requested resource was not found"}
```

### health_routes

기본 헬스체크 엔드포인트를 제공합니다.

```rust
use server_kit::RouterExt;

let app = Router::new().with_health_check();  // .merge(health_routes()) 와 동일
```

| 경로          | 응답                        |
| ------------- | --------------------------- |
| `GET /health` | `200 OK` - 서버 동작 확인   |
| `GET /ready`  | `200 OK` - 서비스 준비 상태 |

### HttpError trait

통일된 에러 응답 포맷을 위한 trait입니다.

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

// 핸들러에서 사용
async fn get_user(Path(id): Path<i64>) -> Result<Json<User>, AppError> {
    find_user(id).await.ok_or(AppError::NotFound)
}
```

응답 포맷:
```json
{"code": "NOT_FOUND", "message": "Resource not found"}
```

### Metrics (feature: `metrics`)

Prometheus 메트릭을 수집합니다.

```rust
use server_kit::RouterExt;

let app = Router::new()
    .route("/", get(handler))
    .with_default_layers(&config)
    .with_metrics();  // 기본 경로: /metrics

// 또는 커스텀 경로 사용
let app = Router::new()
    .route("/", get(handler))
    .with_metrics_at("/internal/metrics");
```

수집되는 메트릭:
- `http_requests_total` - 요청 수 (method, path, status)
- `http_request_duration_seconds` - 응답 시간 (method, path, status)

경로는 axum의 `MatchedPath`를 사용하여 라우트 템플릿으로 기록됩니다:
- `GET /users/123` → `path="/users/{id}"` (라우트가 `/users/{id}`로 정의된 경우)
- 매칭되지 않는 경로는 원본 URI 경로가 사용됩니다

## 전체 예시

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

## 라이센스

MIT
