//! gRPC client channel configuration.

use serde::{Deserialize, Serialize};
use std::time::Duration;

pub use server_kit::{ConfigBuilder, ConfigError};

/// Configuration for gRPC client channels.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ChannelConfig {
    /// Service endpoint URL.
    pub endpoint: String,
    /// Connection timeout in seconds.
    pub connect_timeout_secs: u64,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// TCP keepalive interval in seconds.
    pub tcp_keepalive_secs: Option<u64>,
    /// Enable TCP nodelay.
    pub tcp_nodelay: bool,
    /// HTTP/2 keep-alive interval in seconds.
    pub http2_keepalive_interval_secs: Option<u64>,
    /// HTTP/2 keep-alive timeout in seconds.
    pub http2_keepalive_timeout_secs: Option<u64>,
    /// Path to CA certificate for server verification (PEM format).
    #[cfg(feature = "tls")]
    pub tls_ca_path: Option<String>,
    /// Path to client certificate for mTLS (PEM format).
    #[cfg(feature = "tls")]
    pub tls_cert_path: Option<String>,
    /// Path to client private key for mTLS (PEM format).
    #[cfg(feature = "tls")]
    pub tls_key_path: Option<String>,
    /// Domain name for TLS verification (overrides endpoint host).
    #[cfg(feature = "tls")]
    pub tls_domain: Option<String>,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://[::1]:50051".to_string(),
            connect_timeout_secs: 10,
            timeout_secs: 30,
            tcp_keepalive_secs: Some(60),
            tcp_nodelay: true,
            http2_keepalive_interval_secs: Some(30),
            http2_keepalive_timeout_secs: Some(20),
            #[cfg(feature = "tls")]
            tls_ca_path: None,
            #[cfg(feature = "tls")]
            tls_cert_path: None,
            #[cfg(feature = "tls")]
            tls_key_path: None,
            #[cfg(feature = "tls")]
            tls_domain: None,
        }
    }
}

impl ChannelConfig {
    /// Create a new configuration builder.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config: ChannelConfig = ChannelConfig::builder()
    ///     .with_dotenv()
    ///     .build()?;
    /// ```
    pub fn builder() -> ChannelConfigBuilder {
        ChannelConfigBuilder::new()
    }

    /// Get the connection timeout duration.
    pub fn connect_timeout(&self) -> Duration {
        Duration::from_secs(self.connect_timeout_secs)
    }

    /// Get the request timeout duration.
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    /// Get the TCP keepalive duration.
    pub fn tcp_keepalive(&self) -> Option<Duration> {
        self.tcp_keepalive_secs.map(Duration::from_secs)
    }

    /// Get the HTTP/2 keep-alive interval.
    pub fn http2_keepalive_interval(&self) -> Option<Duration> {
        self.http2_keepalive_interval_secs.map(Duration::from_secs)
    }

    /// Get the HTTP/2 keep-alive timeout.
    pub fn http2_keepalive_timeout(&self) -> Option<Duration> {
        self.http2_keepalive_timeout_secs.map(Duration::from_secs)
    }

    /// Check if TLS is configured (CA certificate path set).
    #[cfg(feature = "tls")]
    pub fn is_tls_enabled(&self) -> bool {
        self.tls_ca_path.is_some()
    }

    /// Check if mTLS is configured (both client cert and key set).
    #[cfg(feature = "tls")]
    pub fn is_mtls_enabled(&self) -> bool {
        self.tls_cert_path.is_some() && self.tls_key_path.is_some()
    }

    /// Load CA certificate for server verification.
    #[cfg(feature = "tls")]
    pub fn ca_certificate(&self) -> Result<Option<tonic::transport::Certificate>, std::io::Error> {
        match &self.tls_ca_path {
            Some(path) => {
                let pem = std::fs::read(path)?;
                Ok(Some(tonic::transport::Certificate::from_pem(pem)))
            }
            None => Ok(None),
        }
    }

    /// Load client identity for mTLS.
    #[cfg(feature = "tls")]
    pub fn client_identity(&self) -> Result<Option<tonic::transport::Identity>, std::io::Error> {
        match (&self.tls_cert_path, &self.tls_key_path) {
            (Some(cert_path), Some(key_path)) => {
                let cert = std::fs::read(cert_path)?;
                let key = std::fs::read(key_path)?;
                Ok(Some(tonic::transport::Identity::from_pem(cert, key)))
            }
            _ => Ok(None),
        }
    }

    /// Build TLS configuration for the client.
    #[cfg(feature = "tls")]
    pub fn tls_config(&self) -> Result<Option<tonic::transport::ClientTlsConfig>, std::io::Error> {
        if let Some(ca) = self.ca_certificate()? {
            let mut tls = tonic::transport::ClientTlsConfig::new().ca_certificate(ca);

            if let Some(domain) = &self.tls_domain {
                tls = tls.domain_name(domain);
            }

            if let Some(identity) = self.client_identity()? {
                tls = tls.identity(identity);
            }

            Ok(Some(tls))
        } else {
            Ok(None)
        }
    }
}

/// Builder for ChannelConfig with additional convenience methods.
#[derive(Default)]
pub struct ChannelConfigBuilder {
    inner: ConfigBuilder,
    endpoint: Option<String>,
    timeout_secs: Option<u64>,
    connect_timeout_secs: Option<u64>,
}

impl ChannelConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load environment variables from `.env` file.
    pub fn with_dotenv(mut self) -> Self {
        self.inner = self.inner.with_dotenv();
        self
    }

    /// Load a configuration file.
    pub fn with_config_file(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.inner = self.inner.with_config_file(path);
        self
    }

    /// Set the endpoint URL.
    pub fn endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Set the request timeout in seconds.
    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    /// Set the connection timeout in seconds.
    pub fn connect_timeout_secs(mut self, secs: u64) -> Self {
        self.connect_timeout_secs = Some(secs);
        self
    }

    /// Build the configuration.
    pub fn build(self) -> Result<ChannelConfig, ConfigError> {
        let mut config: ChannelConfig = self.inner.build()?;

        if let Some(endpoint) = self.endpoint {
            config.endpoint = endpoint;
        }
        if let Some(timeout) = self.timeout_secs {
            config.timeout_secs = timeout;
        }
        if let Some(connect_timeout) = self.connect_timeout_secs {
            config.connect_timeout_secs = connect_timeout;
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_config_defaults() {
        let config = ChannelConfig::default();
        assert_eq!(config.endpoint, "http://[::1]:50051");
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.connect_timeout_secs, 10);
        assert!(config.tcp_nodelay);
    }

    #[test]
    fn channel_config_timeouts() {
        let config = ChannelConfig {
            timeout_secs: 60,
            connect_timeout_secs: 15,
            ..Default::default()
        };
        assert_eq!(config.timeout(), Duration::from_secs(60));
        assert_eq!(config.connect_timeout(), Duration::from_secs(15));
    }

    #[test]
    fn channel_config_keepalive() {
        let config = ChannelConfig::default();
        assert_eq!(config.tcp_keepalive(), Some(Duration::from_secs(60)));
        assert_eq!(
            config.http2_keepalive_interval(),
            Some(Duration::from_secs(30))
        );
        assert_eq!(
            config.http2_keepalive_timeout(),
            Some(Duration::from_secs(20))
        );
    }

    #[test]
    fn channel_config_builder_with_endpoint() {
        let config: ChannelConfig = ChannelConfig::builder()
            .endpoint("http://localhost:9000")
            .timeout_secs(60)
            .build()
            .unwrap();

        assert_eq!(config.endpoint, "http://localhost:9000");
        assert_eq!(config.timeout_secs, 60);
    }

    #[test]
    fn channel_config_builder_with_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
            endpoint = "http://api.example.com:50051"
            timeout_secs = 45
            "#,
        )
        .unwrap();

        let config: ChannelConfig = ChannelConfig::builder()
            .with_config_file(&path)
            .build()
            .unwrap();

        assert_eq!(config.endpoint, "http://api.example.com:50051");
        assert_eq!(config.timeout_secs, 45);
    }

    #[test]
    fn channel_config_builder_override_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "endpoint = \"http://file.example.com:50051\"").unwrap();

        let config: ChannelConfig = ChannelConfig::builder()
            .with_config_file(&path)
            .endpoint("http://override.example.com:9000")
            .build()
            .unwrap();

        // Builder endpoint should override file
        assert_eq!(config.endpoint, "http://override.example.com:9000");
    }
}
