//! Health check service utilities.
//!
//! Provides integration with the standard gRPC health checking protocol.

#[cfg(feature = "health")]
pub use tonic_health::server::HealthReporter;
#[cfg(feature = "health")]
pub use tonic_health::ServingStatus;

/// Create a health service and reporter.
///
/// Returns a tuple of (HealthReporter, HealthService) that can be used with
/// tonic's Server builder.
///
/// # Example
///
/// ```ignore
/// use server_kit_grpc::health::health_service;
/// use tonic::transport::Server;
///
/// let (mut health_reporter, health_service) = health_service();
///
/// // Set service as serving
/// health_reporter
///     .set_serving::<MyServiceServer<MyImpl>>()
///     .await;
///
/// Server::builder()
///     .add_service(health_service)
///     .add_service(MyServiceServer::new(my_impl))
///     .serve(addr)
///     .await?;
/// ```
#[cfg(feature = "health")]
pub fn health_service() -> (
    tonic_health::server::HealthReporter,
    tonic_health::pb::health_server::HealthServer<impl tonic_health::pb::health_server::Health>,
) {
    tonic_health::server::health_reporter()
}

#[cfg(test)]
#[cfg(feature = "health")]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_service_creates_reporter() {
        let (mut reporter, _service) = health_service();

        // Should be able to set status without error
        reporter
            .set_service_status("test.service", ServingStatus::Serving)
            .await;
    }

    #[tokio::test]
    async fn health_service_status_changes() {
        let (mut reporter, _service) = health_service();

        reporter
            .set_service_status("test.service", ServingStatus::Serving)
            .await;

        reporter
            .set_service_status("test.service", ServingStatus::NotServing)
            .await;
    }
}
