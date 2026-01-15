# server-kit

A collection of thin utility crates for building Rust servers with minimal boilerplate.

## Philosophy

- **Minimal implementation**: Reuse well-established crates (axum, tonic, tower)
- **Thin wrapper**: Don't hide underlying APIs
- **Extension traits**: Add chainable methods to existing types
- **Opt-in features**: Include only what you need via feature flags

## Crates

| Crate | Description |
| ----- | ----------- |
| [server-kit](./crates/server-kit) | Shared utilities (config, logging, environment) |
| [server-kit-rest](./crates/server-kit-rest) | REST server utilities for axum |
| [server-kit-grpc](./crates/server-kit-grpc) | gRPC server/client utilities for tonic |

## Quick Start

### REST Server (axum)

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
        .with_health_check()
        .with_fallback()
        .with_default_layers(&config)
        .serve(&config)
        .await?;

    Ok(())
}
```

### gRPC Server (tonic)

```rust
use server_kit_grpc::{GrpcServerConfig, RouterExt, ServerExt};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: GrpcServerConfig = GrpcServerConfig::builder()
        .with_dotenv()
        .build()?;

    Server::builder()
        .with_default_layers()
        .add_service(MyServiceServer::new(my_impl))
        .serve_with(&config)
        .await?;

    Ok(())
}
```

## License

MIT
