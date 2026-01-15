//! gRPC server configuration.

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;

pub use server_kit::{ConfigBuilder, Environment};

/// gRPC server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GrpcServerConfig {
    pub environment: Environment,
    pub host: String,
    pub port: u16,
    /// Request timeout in seconds.
    pub request_timeout_secs: u64,
    /// Maximum concurrent streams per connection.
    pub max_concurrent_streams: Option<u32>,
    /// TCP keepalive interval in seconds.
    pub tcp_keepalive_secs: Option<u64>,
    /// Enable TCP nodelay.
    pub tcp_nodelay: bool,
    /// Path to TLS certificate (PEM format).
    #[cfg(feature = "tls")]
    pub tls_cert_path: Option<String>,
    /// Path to TLS private key (PEM format).
    #[cfg(feature = "tls")]
    pub tls_key_path: Option<String>,
    /// Path to CA certificate for client authentication (PEM format).
    #[cfg(feature = "tls")]
    pub tls_ca_path: Option<String>,
}

impl Default for GrpcServerConfig {
    fn default() -> Self {
        Self {
            environment: Environment::default(),
            host: "[::1]".to_string(),
            port: 50051,
            request_timeout_secs: 30,
            max_concurrent_streams: None,
            tcp_keepalive_secs: Some(60),
            tcp_nodelay: true,
            #[cfg(feature = "tls")]
            tls_cert_path: None,
            #[cfg(feature = "tls")]
            tls_key_path: None,
            #[cfg(feature = "tls")]
            tls_ca_path: None,
        }
    }
}

impl GrpcServerConfig {
    /// Create a new configuration builder.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config: GrpcServerConfig = GrpcServerConfig::builder()
    ///     .with_dotenv()
    ///     .build()?;
    /// ```
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    /// Get the server address string.
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Parse the address as a SocketAddr.
    pub fn socket_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        self.addr().parse()
    }

    /// Get the request timeout duration.
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_secs)
    }

    /// Get the TCP keepalive duration.
    pub fn tcp_keepalive(&self) -> Option<Duration> {
        self.tcp_keepalive_secs.map(Duration::from_secs)
    }

    /// Check if TLS is configured.
    #[cfg(feature = "tls")]
    pub fn is_tls_enabled(&self) -> bool {
        self.tls_cert_path.is_some() && self.tls_key_path.is_some()
    }

    /// Load server TLS identity from configured paths.
    ///
    /// Returns `None` if TLS is not configured.
    #[cfg(feature = "tls")]
    pub fn tls_identity(&self) -> Result<Option<tonic::transport::Identity>, std::io::Error> {
        match (&self.tls_cert_path, &self.tls_key_path) {
            (Some(cert_path), Some(key_path)) => {
                let cert = std::fs::read(cert_path)?;
                let key = std::fs::read(key_path)?;
                Ok(Some(tonic::transport::Identity::from_pem(cert, key)))
            }
            _ => Ok(None),
        }
    }

    /// Load client CA certificate for mTLS.
    ///
    /// Returns `None` if client auth is not configured.
    #[cfg(feature = "tls")]
    pub fn client_ca_cert(&self) -> Result<Option<tonic::transport::Certificate>, std::io::Error> {
        match &self.tls_ca_path {
            Some(ca_path) => {
                let ca = std::fs::read(ca_path)?;
                Ok(Some(tonic::transport::Certificate::from_pem(ca)))
            }
            None => Ok(None),
        }
    }

    /// Build TLS configuration for the server.
    #[cfg(feature = "tls")]
    pub fn tls_config(&self) -> Result<Option<tonic::transport::ServerTlsConfig>, std::io::Error> {
        if let Some(identity) = self.tls_identity()? {
            let mut tls = tonic::transport::ServerTlsConfig::new().identity(identity);

            if let Some(ca) = self.client_ca_cert()? {
                tls = tls.client_ca_root(ca);
            }

            Ok(Some(tls))
        } else {
            Ok(None)
        }
    }
}

impl AsRef<GrpcServerConfig> for GrpcServerConfig {
    fn as_ref(&self) -> &GrpcServerConfig {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grpc_server_config_defaults() {
        let config = GrpcServerConfig::default();
        assert_eq!(config.host, "[::1]");
        assert_eq!(config.port, 50051);
        assert_eq!(config.request_timeout_secs, 30);
        assert!(config.tcp_nodelay);
        assert_eq!(config.tcp_keepalive_secs, Some(60));
    }

    #[test]
    fn grpc_server_config_addr() {
        let config = GrpcServerConfig {
            host: "127.0.0.1".to_string(),
            port: 9000,
            ..Default::default()
        };
        assert_eq!(config.addr(), "127.0.0.1:9000");
    }

    #[test]
    fn grpc_server_config_socket_addr() {
        let config = GrpcServerConfig {
            host: "127.0.0.1".to_string(),
            port: 50051,
            ..Default::default()
        };

        let addr: SocketAddr = config.socket_addr().unwrap();
        assert_eq!(addr.to_string(), "127.0.0.1:50051");
    }

    #[test]
    fn grpc_server_config_request_timeout() {
        let config = GrpcServerConfig {
            request_timeout_secs: 60,
            ..Default::default()
        };
        assert_eq!(config.request_timeout(), Duration::from_secs(60));
    }

    #[test]
    fn grpc_server_config_tcp_keepalive() {
        let config = GrpcServerConfig::default();
        assert_eq!(config.tcp_keepalive(), Some(Duration::from_secs(60)));

        let config = GrpcServerConfig {
            tcp_keepalive_secs: None,
            ..Default::default()
        };
        assert_eq!(config.tcp_keepalive(), None);
    }

    #[test]
    fn grpc_server_config_builder() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "host = \"0.0.0.0\"\nport = 50052").unwrap();

        let config: GrpcServerConfig = GrpcServerConfig::builder()
            .with_config_file(&path)
            .build()
            .unwrap();

        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 50052);
    }

    #[test]
    fn grpc_server_config_from_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        std::fs::write(
            &path,
            "host: \"0.0.0.0\"\nport: 9000\nenvironment: production",
        )
        .unwrap();

        let config: GrpcServerConfig = GrpcServerConfig::builder()
            .with_config_file(&path)
            .build()
            .unwrap();

        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 9000);
        assert!(config.environment.is_production());
    }
}
