# server-kit-grpc

A thin extension crate for tonic gRPC servers and clients.

## Design Philosophy

This crate follows the same design philosophy as `server-kit-rest` for HTTP servers:

**Extend native tonic code with chainable methods, not replace it.**

### API Design Rules

All features in this crate **MUST** follow these rules:

1. **Extension traits over standalone functions** - Add methods to existing tonic types via traits
2. **Method chaining** - All operations should be chainable (`.method().method()`)
3. **No wrapper types** - Users work directly with `tonic::transport::Server`, `Router`, `Channel`
4. **Preserve native APIs** - Never hide or replace tonic's native methods

```rust
// GOOD - Extension trait adds method to native type
Server::builder()
    .with_default_layers()           // ServerExt method
    .add_service(MyServiceServer::new(impl))
    .serve_with(&config)             // RouterExt method
    .await?;

// BAD - Standalone function breaks chaining
let router = Server::builder().add_service(...);
serve(router, &config).await?;  // Don't do this
```

## Installation

```toml
[dependencies]
server-kit-grpc = { version = "0.1", features = ["full"] }
```

### Features

| Feature      | Description                    | Default |
| ------------ | ------------------------------ | ------- |
| `tracing`    | Logging initialization         | Yes     |
| `health`     | gRPC health checking service   | Yes     |
| `tls`        | TLS support                    | No      |
| `metrics`    | Prometheus metrics             | No      |
| `reflection` | gRPC server reflection         | No      |
| `full`       | All features                   | No      |

## Quick Start - Server

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

## Quick Start - Client

```rust
use server_kit_grpc::{ChannelConfig, ChannelExt};
use tonic::transport::Channel;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: ChannelConfig = ChannelConfig::builder()
        .endpoint("http://localhost:50051")
        .build()?;

    let channel = Channel::connect(&config).await?;  // ChannelExt method
    let mut client = MyServiceClient::new(channel);

    let response = client.my_method(MyRequest { ... }).await?;
    Ok(())
}
```

## API Reference

### Extension Traits

#### ServerExt

Extension trait for `tonic::transport::Server`.

```rust
use server_kit_grpc::ServerExt;
use tonic::transport::Server;

Server::builder()
    .with_default_layers()  // Adds TraceLayer for request logging
    .add_service(...)
```

| Method                 | Description                          |
| ---------------------- | ------------------------------------ |
| `with_default_layers()` | Apply default middleware (TraceLayer) |

#### RouterExt

Extension trait for `tonic::transport::server::Router`.

```rust
use server_kit_grpc::RouterExt;

Server::builder()
    .add_service(MyServiceServer::new(impl))
    .serve_with(&config)  // Serve with graceful shutdown
    .await?;
```

| Method              | Description                                |
| ------------------- | ------------------------------------------ |
| `serve_with(&config)` | Serve with config and graceful shutdown    |
| `serve_at(addr)`    | Serve at specific address with shutdown    |

#### ChannelExt

Extension trait for `tonic::transport::Channel`.

```rust
use server_kit_grpc::ChannelExt;
use tonic::transport::Channel;

let channel = Channel::connect(&config).await?;
let channel = Channel::connect_lazy(&config)?;
```

| Method              | Description                        |
| ------------------- | ---------------------------------- |
| `connect(&config)`  | Eager connection (fails if unreachable) |
| `connect_lazy(&config)` | Lazy connection (on first request) |

### Configuration

#### GrpcServerConfig

Server configuration.

```rust
let config: GrpcServerConfig = GrpcServerConfig::builder()
    .with_dotenv()
    .build()?;
```

| Environment Variable | Default   | Description |
| -------------------- | --------- | ----------- |
| `GRPC_HOST`          | `[::1]`   | Bind host   |
| `GRPC_PORT`          | `50051`   | Port        |

#### ChannelConfig

Client channel configuration.

```rust
let config: ChannelConfig = ChannelConfig::builder()
    .endpoint("http://localhost:50051")
    .timeout_secs(30)
    .build()?;
```

| Field                | Default | Description                |
| -------------------- | ------- | -------------------------- |
| `endpoint`           | -       | Server URL (required)      |
| `timeout_secs`       | 30      | Request timeout            |
| `connect_timeout_secs` | 5     | Connection timeout         |
| `tcp_nodelay`        | true    | TCP_NODELAY option         |

### Health Service (feature: `health`)

Standard gRPC health checking protocol.

```rust
use server_kit_grpc::{health_service, HealthReporter, ServingStatus};

let (mut health_reporter, health_service) = health_service();

// Set service status
health_reporter
    .set_serving::<MyServiceServer<MyImpl>>()
    .await;

Server::builder()
    .with_default_layers()
    .add_service(health_service)
    .add_service(MyServiceServer::new(impl))
    .serve_with(&config)
    .await?;
```

### Reflection Service (feature: `reflection`)

Enable service discovery for tools like grpcurl.

```rust
use server_kit_grpc::reflection_service;

const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("descriptor.bin");

let reflection = reflection_service(&[FILE_DESCRIPTOR_SET])?;

Server::builder()
    .add_service(reflection)
    .add_service(MyServiceServer::new(impl))
    .serve_with(&config)
    .await?;
```

### Error Handling

#### GrpcError Trait

Trait for unified gRPC error responses.

```rust
use server_kit_grpc::{GrpcError, Code, Status};

#[derive(Debug)]
enum AppError {
    NotFound,
    InvalidInput(String),
}

impl GrpcError for AppError {
    fn code(&self) -> Code {
        match self {
            Self::NotFound => Code::NotFound,
            Self::InvalidInput(_) => Code::InvalidArgument,
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::NotFound => "Resource not found",
            Self::InvalidInput(msg) => msg,
        }
    }
}

impl From<AppError> for Status {
    fn from(e: AppError) -> Status {
        e.into_status()
    }
}
```

## Full Example

```rust
use server_kit_grpc::{
    health_service, init_logging_from_env, reflection_service,
    GrpcServerConfig, HealthReporter, Request, Response,
    RouterExt, ServerExt, ServingStatus, Status,
};
use tonic::transport::Server;

pub mod greeter {
    tonic::include_proto!("greeter");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("greeter_descriptor");
}

use greeter::greeter_server::{Greeter, GreeterServer};

#[derive(Default)]
pub struct MyGreeter;

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<greeter::HelloRequest>,
    ) -> Result<Response<greeter::HelloReply>, Status> {
        Ok(Response::new(greeter::HelloReply {
            message: format!("Hello, {}!", request.get_ref().name),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging_from_env();

    let config: GrpcServerConfig = GrpcServerConfig::builder()
        .with_dotenv()
        .build()?;

    let (mut health_reporter, health_service) = health_service();
    health_reporter.set_serving::<GreeterServer<MyGreeter>>().await;

    let reflection = reflection_service(&[greeter::FILE_DESCRIPTOR_SET])?;

    Server::builder()
        .with_default_layers()
        .add_service(health_service)
        .add_service(reflection)
        .add_service(GreeterServer::new(MyGreeter))
        .serve_with(&config)
        .await?;

    Ok(())
}
```

## License

MIT
