//! Simple gRPC Server Example (minimal dependencies)
//!
//! This example shows the basic server-kit-grpc patterns without
//! optional features like health checks or reflection.
//!
//! Run with:
//! ```bash
//! cargo run -p server-kit-grpc --example simple_server
//! ```
//!
//! Test with grpcurl (requires you know the proto schema):
//! ```bash
//! grpcurl -plaintext -d '{"name": "World"}' \
//!   -proto crates/server-kit-grpc/proto/greeter.proto \
//!   localhost:50051 greeter.Greeter/SayHello
//! ```

use server_kit_grpc::{
    init_logging_from_env, GrpcServerConfig, Request, Response, RouterExt, ServerExt, Status,
};
use tonic::transport::Server;

// Include the generated protobuf code
pub mod greeter {
    tonic::include_proto!("greeter");
}

use greeter::greeter_server::{Greeter, GreeterServer};
use greeter::{HelloReply, HelloRequest};

/// Simple Greeter implementation
#[derive(Debug, Default)]
pub struct SimpleGreeter;

#[tonic::async_trait]
impl Greeter for SimpleGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let name = &request.get_ref().name;
        tracing::info!(name = %name, "Received SayHello request");

        Ok(Response::new(HelloReply {
            message: format!("Hello, {}!", name),
        }))
    }

    type SayHelloStreamStream =
        std::pin::Pin<Box<dyn tokio_stream::Stream<Item = Result<HelloReply, Status>> + Send>>;

    async fn say_hello_stream(
        &self,
        _request: Request<HelloRequest>,
    ) -> Result<Response<Self::SayHelloStreamStream>, Status> {
        Err(Status::unimplemented("Streaming not implemented"))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging_from_env();

    // Simple configuration - uses defaults ([::1]:50051)
    let config = GrpcServerConfig::default();

    tracing::info!(addr = %config.addr(), "Starting simple gRPC server");

    // Build and serve with graceful shutdown (Ctrl+C)
    Server::builder()
        .with_default_layers()
        .add_service(GreeterServer::new(SimpleGreeter))
        .serve_with(&config)
        .await?;

    tracing::info!("Server shutdown");
    Ok(())
}
