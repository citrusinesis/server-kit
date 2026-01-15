//! Server configuration.

use serde::{Deserialize, Serialize};
use std::time::Duration;

pub use server_kit::{ConfigBuilder, ConfigError, Environment};

/// Server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub environment: Environment,
    pub host: String,
    pub port: u16,
    pub request_timeout_secs: u64,
    /// CORS allowed origins. Empty means CORS is disabled.
    /// Only used when `cors` feature is enabled.
    #[serde(default)]
    pub cors_origins: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            environment: Environment::default(),
            host: "0.0.0.0".to_string(),
            port: 3000,
            request_timeout_secs: 30,
            cors_origins: Vec::new(),
        }
    }
}

impl ServerConfig {
    /// Create a new configuration builder.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config: ServerConfig = ServerConfig::builder()
    ///     .with_dotenv()
    ///     .with_config_file("config.toml")
    ///     .build()?;
    /// ```
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_secs)
    }

    pub(crate) fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl AsRef<ServerConfig> for ServerConfig {
    fn as_ref(&self) -> &ServerConfig {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    #[test]
    fn environment_from_str() {
        assert_eq!(
            "production".parse::<Environment>().unwrap(),
            Environment::Production
        );
        assert_eq!(
            "Production".parse::<Environment>().unwrap(),
            Environment::Production
        );
        assert_eq!(
            "PRODUCTION".parse::<Environment>().unwrap(),
            Environment::Production
        );
        assert_eq!(
            "prod".parse::<Environment>().unwrap(),
            Environment::Production
        );
        assert_eq!(
            "development".parse::<Environment>().unwrap(),
            Environment::Development
        );
        assert_eq!(
            "dev".parse::<Environment>().unwrap(),
            Environment::Development
        );
        assert_eq!(
            "anything".parse::<Environment>().unwrap(),
            Environment::Development
        );
    }

    #[test]
    fn environment_is_methods() {
        assert!(Environment::Production.is_production());
        assert!(!Environment::Production.is_development());
        assert!(Environment::Development.is_development());
        assert!(!Environment::Development.is_production());
    }

    #[test]
    fn server_config_defaults() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 3000);
        assert_eq!(config.request_timeout_secs, 30);
        assert!(config.cors_origins.is_empty());
        assert!(config.environment.is_development());
    }

    #[test]
    fn server_config_addr() {
        let config = ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            ..Default::default()
        };
        assert_eq!(config.addr(), "127.0.0.1:8080");
    }

    #[test]
    fn server_config_request_timeout() {
        let config = ServerConfig {
            request_timeout_secs: 60,
            ..Default::default()
        };
        assert_eq!(config.request_timeout(), Duration::from_secs(60));
    }

    #[test]
    fn config_builder_loads_toml() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        std::fs::write(
            &config_path,
            r#"
            host = "127.0.0.1"
            port = 8080
            request_timeout_secs = 60
            "#,
        )
        .unwrap();

        let config: ServerConfig = ServerConfig::builder()
            .with_config_file(&config_path)
            .build()
            .unwrap();

        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.request_timeout_secs, 60);
    }

    #[test]
    fn config_builder_loads_yaml() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.yaml");

        std::fs::write(
            &config_path,
            r#"
host: "192.168.1.1"
port: 9000
environment: production
"#,
        )
        .unwrap();

        let config: ServerConfig = ServerConfig::builder()
            .with_config_file(&config_path)
            .build()
            .unwrap();

        assert_eq!(config.host, "192.168.1.1");
        assert_eq!(config.port, 9000);
        assert!(config.environment.is_production());
    }

    #[test]
    fn config_builder_loads_json() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.json");

        std::fs::write(&config_path, r#"{"host": "10.0.0.1", "port": 5000}"#).unwrap();

        let config: ServerConfig = ServerConfig::builder()
            .with_config_file(&config_path)
            .build()
            .unwrap();

        assert_eq!(config.host, "10.0.0.1");
        assert_eq!(config.port, 5000);
    }

    #[test]
    fn config_builder_file_not_found() {
        let result: Result<ServerConfig, _> = ServerConfig::builder()
            .with_config_file("/nonexistent/path/config.toml")
            .build();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::NotFound(_)));
    }

    #[test]
    fn config_error_display() {
        let err = ConfigError::NotFound(PathBuf::from("/test/path"));
        assert!(err.to_string().contains("/test/path"));

        let err = ConfigError::Parse("invalid syntax".to_string());
        assert!(err.to_string().contains("invalid syntax"));
    }

    #[test]
    fn config_builder_loads_dotenv() {
        let dir = tempfile::tempdir().unwrap();
        let env_path = dir.path().join(".env.test");

        let mut file = std::fs::File::create(&env_path).unwrap();
        writeln!(file, "TEST_VAR_FOR_DOTENV=hello").unwrap();

        let _: ServerConfig = ServerConfig::builder()
            .with_config_file(&env_path)
            .build()
            .unwrap();
    }
}
