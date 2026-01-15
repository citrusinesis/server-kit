//! Channel extension trait for gRPC clients.

use std::time::Duration;
use tonic::transport::{Channel, Endpoint};

use crate::config::ChannelConfig;
use crate::error::Error;

/// Build an endpoint from configuration.
fn build_endpoint(config: &ChannelConfig) -> Result<Endpoint, Error> {
    let mut endpoint = Endpoint::from_shared(config.endpoint.clone())
        .map_err(|e| Error::InvalidEndpoint(e.to_string()))?
        .timeout(Duration::from_secs(config.timeout_secs))
        .connect_timeout(Duration::from_secs(config.connect_timeout_secs));

    if config.tcp_nodelay {
        endpoint = endpoint.tcp_nodelay(true);
    }

    if let Some(keepalive) = config.tcp_keepalive_secs {
        endpoint = endpoint.tcp_keepalive(Some(Duration::from_secs(keepalive)));
    }

    if let Some(interval) = config.http2_keepalive_interval_secs {
        endpoint = endpoint.http2_keep_alive_interval(Duration::from_secs(interval));
    }

    if let Some(timeout) = config.http2_keepalive_timeout_secs {
        endpoint = endpoint.keep_alive_timeout(Duration::from_secs(timeout));
    }

    #[cfg(feature = "tls")]
    if let Some(tls_config) = config.tls_config().map_err(Error::tls)? {
        endpoint = endpoint
            .tls_config(tls_config)
            .map_err(|e| Error::InvalidEndpoint(e.to_string()))?;
    }

    Ok(endpoint)
}

/// Extension trait for Channel (mirrors RouterExt pattern from server-kit).
///
/// # Example
///
/// ```ignore
/// use server_kit_grpc::{ChannelConfig, ChannelExt};
/// use tonic::transport::Channel;
///
/// let config: ChannelConfig = ChannelConfig::builder()
///     .endpoint("http://localhost:50051")
///     .build()?;
///
/// let channel = Channel::connect(&config).await?;
/// let client = MyServiceClient::new(channel);
/// ```
pub trait ChannelExt: Sized {
    /// Connect to server with config (eager connection).
    ///
    /// This establishes a connection immediately and fails if the server is unreachable.
    fn connect(
        config: &ChannelConfig,
    ) -> impl std::future::Future<Output = Result<Channel, Error>> + Send;

    /// Connect lazily with config (connects on first request).
    ///
    /// This creates a channel that will connect when the first request is made.
    /// Useful when you want to create the client but delay the actual connection.
    fn connect_lazy(config: &ChannelConfig) -> Result<Channel, Error>;
}

impl ChannelExt for Channel {
    async fn connect(config: &ChannelConfig) -> Result<Channel, Error> {
        let endpoint = build_endpoint(config)?;
        endpoint.connect().await.map_err(Error::from)
    }

    fn connect_lazy(config: &ChannelConfig) -> Result<Channel, Error> {
        let endpoint = build_endpoint(config)?;
        Ok(endpoint.connect_lazy())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn channel_connect_lazy_creates_channel() {
        let config = ChannelConfig {
            endpoint: "http://[::1]:50051".to_string(),
            timeout_secs: 30,
            connect_timeout_secs: 5,
            ..Default::default()
        };

        // connect_lazy should succeed without actual server
        let result = Channel::connect_lazy(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn channel_connect_lazy_invalid_endpoint() {
        let config = ChannelConfig {
            endpoint: "not a valid url".to_string(),
            ..Default::default()
        };

        // Invalid endpoint should fail at parsing, not requiring tokio runtime
        let result = Endpoint::from_shared(config.endpoint.clone());
        assert!(result.is_err());
    }

}
