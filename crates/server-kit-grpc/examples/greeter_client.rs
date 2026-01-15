//! gRPC Greeter Client Example
//!
//! Run the server first:
//! ```bash
//! cargo run -p server-kit-grpc --example greeter_server --features "health,reflection"
//! ```
//!
//! Then run this client:
//! ```bash
//! cargo run -p server-kit-grpc --example greeter_client
//! ```

use server_kit_grpc::{init_logging_from_env, ChannelConfig, ChannelExt};
use tonic::transport::Channel;

// Include the generated protobuf code
pub mod greeter {
    tonic::include_proto!("greeter");
}

use greeter::greeter_client::GreeterClient;
use greeter::HelloRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging_from_env();

    // Create channel configuration
    let config = ChannelConfig::builder()
        .endpoint("http://[::1]:50051")
        .timeout_secs(30)
        .build()?;

    tracing::info!(endpoint = %config.endpoint, "Connecting to gRPC server");

    // Connect to the server using the ChannelExt trait
    let channel: Channel = Channel::connect(&config).await?;

    // Create the client
    let mut client = GreeterClient::new(channel);

    // Call SayHello
    println!("\n--- Unary Call: SayHello ---");
    let request = tonic::Request::new(HelloRequest {
        name: "World".to_string(),
    });

    let response = client.say_hello(request).await?;
    println!("Response: {}", response.get_ref().message);

    // Call SayHello with different names
    for name in &["Alice", "Bob", "Charlie"] {
        let request = tonic::Request::new(HelloRequest {
            name: name.to_string(),
        });

        let response = client.say_hello(request).await?;
        println!("Response: {}", response.get_ref().message);
    }

    // Call streaming endpoint
    println!("\n--- Server Streaming: SayHelloStream ---");
    let request = tonic::Request::new(HelloRequest {
        name: "Stream".to_string(),
    });

    let mut stream = client.say_hello_stream(request).await?.into_inner();

    while let Some(reply) = stream.message().await? {
        println!("Stream response: {}", reply.message);
    }

    println!("\n--- Done ---");

    Ok(())
}
