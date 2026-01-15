//! gRPC Greeter Server Example
//!
//! Run with:
//! ```bash
//! cargo run -p server-kit-grpc --example greeter_server --features "health,reflection"
//! ```
//!
//! Test with grpcurl:
//! ```bash
//! # List services (requires reflection)
//! grpcurl -plaintext localhost:50051 list
//!
//! # Call SayHello
//! grpcurl -plaintext -d '{"name": "World"}' localhost:50051 greeter.Greeter/SayHello
//!
//! # Check health
//! grpcurl -plaintext localhost:50051 grpc.health.v1.Health/Check
//! ```

use server_kit_grpc::{
    health_service, init_logging_from_env, reflection_service, GrpcServerConfig, HealthReporter,
    Request, Response, RouterExt, ServerExt, ServingStatus, Status,
};
use tonic::transport::Server;

// Include the generated protobuf code
pub mod greeter {
    tonic::include_proto!("greeter");

    // Include file descriptor set for reflection
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("greeter_descriptor");
}

use greeter::greeter_server::{Greeter, GreeterServer};
use greeter::{HelloReply, HelloRequest};

/// Our Greeter service implementation
#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let name = &request.get_ref().name;

        let reply = HelloReply {
            message: format!("Hello, {}!", name),
        };

        Ok(Response::new(reply))
    }

    type SayHelloStreamStream =
        std::pin::Pin<Box<dyn tokio_stream::Stream<Item = Result<HelloReply, Status>> + Send>>;

    async fn say_hello_stream(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<Self::SayHelloStreamStream>, Status> {
        let name = request.into_inner().name;

        let stream = async_stream::stream! {
            for i in 1..=5 {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                yield Ok(HelloReply {
                    message: format!("Hello #{} {}!", i, name),
                });
            }
        };

        Ok(Response::new(Box::pin(stream)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging_from_env();

    // Load configuration
    let config: GrpcServerConfig = GrpcServerConfig::builder().with_dotenv().build()?;

    tracing::info!(
        host = %config.host,
        port = %config.port,
        "Starting gRPC server"
    );

    // Create health service
    let (mut health_reporter, health_service) = health_service();

    // Set service health status
    set_health_status(&mut health_reporter).await;

    // Create reflection service for grpcurl/grpcui discovery
    let reflection_service =
        reflection_service(&[greeter::FILE_DESCRIPTOR_SET]).expect("Failed to create reflection");

    // Create our greeter service
    let greeter = MyGreeter::default();

    // Build and serve with graceful shutdown
    Server::builder()
        .with_default_layers()
        .add_service(health_service)
        .add_service(reflection_service)
        .add_service(GreeterServer::new(greeter))
        .serve_with(&config)
        .await?;

    Ok(())
}

async fn set_health_status(health_reporter: &mut HealthReporter) {
    // Set the greeter service as serving
    health_reporter
        .set_serving::<GreeterServer<MyGreeter>>()
        .await;

    // You can also set status for a custom service name
    health_reporter
        .set_service_status("greeter.Greeter", ServingStatus::Serving)
        .await;
}
