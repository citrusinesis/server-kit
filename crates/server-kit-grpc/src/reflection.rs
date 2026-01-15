//! Server reflection utilities.
//!
//! gRPC server reflection allows clients to discover what services are
//! available on a server without having the .proto files. This is useful
//! for debugging tools like grpcurl, grpcui, and Postman.

/// Create a server reflection service from encoded file descriptor sets.
///
/// # Example
///
/// ```ignore
/// use server_kit_grpc::reflection::reflection_service;
/// use tonic::transport::Server;
///
/// // Include the compiled file descriptor set
/// const FILE_DESCRIPTOR_SET: &[u8] = include_bytes!("../proto/my_service_descriptor.bin");
///
/// let reflection_service = reflection_service(&[FILE_DESCRIPTOR_SET])?;
///
/// Server::builder()
///     .add_service(MyServiceServer::new(my_impl))
///     .add_service(reflection_service)
///     .serve(addr)
///     .await?;
/// ```
#[cfg(feature = "reflection")]
pub fn reflection_service(
    file_descriptor_sets: &[&[u8]],
) -> Result<
    tonic_reflection::server::ServerReflectionServer<
        impl tonic_reflection::server::ServerReflection,
    >,
    tonic_reflection::server::Error,
> {
    let mut builder = tonic_reflection::server::Builder::configure();

    for fds in file_descriptor_sets {
        builder = builder.register_encoded_file_descriptor_set(fds);
    }

    builder.build_v1()
}

/// Create a server reflection service with v1alpha support (legacy clients).
///
/// Some older gRPC clients may only support the v1alpha reflection API.
/// Use this if you need to support such clients.
#[cfg(feature = "reflection")]
pub fn reflection_service_v1alpha(
    file_descriptor_sets: &[&[u8]],
) -> Result<
    tonic_reflection::server::v1alpha::ServerReflectionServer<
        impl tonic_reflection::server::v1alpha::ServerReflection,
    >,
    tonic_reflection::server::Error,
> {
    let mut builder = tonic_reflection::server::Builder::configure();

    for fds in file_descriptor_sets {
        builder = builder.register_encoded_file_descriptor_set(fds);
    }

    builder.build_v1alpha()
}

#[cfg(test)]
#[cfg(feature = "reflection")]
mod tests {
    use super::*;

    #[test]
    fn reflection_service_empty_descriptors() {
        // Empty descriptor list should still create a service
        let result = reflection_service(&[]);
        assert!(result.is_ok());
    }
}
