//! Configuration loading utilities.

use serde::de::DeserializeOwned;
use std::env;
use std::path::{Path, PathBuf};

#[cfg(feature = "tracing")]
use crate::logging::{init_logging, LogFormat};

/// Error type for configuration operations.
#[derive(Debug)]
pub enum ConfigError {
    /// Configuration file not found.
    NotFound(PathBuf),
    /// Failed to parse configuration.
    Parse(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(path) => write!(f, "Config file not found: {}", path.display()),
            Self::Parse(msg) => write!(f, "Failed to parse config: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Supported config file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFormat {
    DotEnv,
    Toml,
    Yaml,
    Json,
}

impl ConfigFormat {
    pub fn from_path(path: impl AsRef<Path>) -> Option<Self> {
        let ext = path.as_ref().extension()?.to_str()?;
        match ext.to_lowercase().as_str() {
            "env" => Some(Self::DotEnv),
            "toml" => Some(Self::Toml),
            "yaml" | "yml" => Some(Self::Yaml),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

/// Configuration builder.
///
/// # Example
///
/// ```ignore
/// use server_kit::ConfigBuilder;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct MyConfig {
///     host: String,
///     port: u16,
/// }
///
/// let config: MyConfig = ConfigBuilder::new()
///     .with_dotenv()
///     .with_config_file("config.toml")
///     .build()?;
/// ```
#[derive(Default)]
pub struct ConfigBuilder {
    load_default_dotenv: bool,
    config_files: Vec<PathBuf>,
    #[cfg(feature = "tracing")]
    init_logging: bool,
}

impl ConfigBuilder {
    /// Create a new configuration builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load environment variables from `.env` file in current directory.
    pub fn with_dotenv(mut self) -> Self {
        self.load_default_dotenv = true;
        self
    }

    /// Load a configuration file.
    ///
    /// File format is detected from extension:
    /// - `.env` - Environment variables (multiple allowed)
    /// - `.toml` / `.yaml` / `.json` - Config file (last one used)
    pub fn with_config_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_files.push(path.into());
        self
    }

    /// Initialize logging from environment variables (`LOG_FORMAT`, `RUST_LOG`).
    #[cfg(feature = "tracing")]
    pub fn with_logging_from_env(mut self) -> Self {
        self.init_logging = true;
        self
    }

    /// Build and return the configuration.
    pub fn build<C: DeserializeOwned>(self) -> Result<C, ConfigError> {
        if self.load_default_dotenv {
            let _ = dotenvy::dotenv();
        }

        let mut main_config_file: Option<PathBuf> = None;

        for path in &self.config_files {
            match ConfigFormat::from_path(path) {
                Some(ConfigFormat::DotEnv) => {
                    if path.exists() {
                        let _ = dotenvy::from_path(path);
                    }
                }
                Some(_) => {
                    main_config_file = Some(path.clone());
                }
                None => {
                    let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                    let is_dotenv = file_name.starts_with(".env") || file_name == "env";
                    if is_dotenv && path.exists() {
                        let _ = dotenvy::from_path(path);
                    }
                }
            }
        }

        #[cfg(feature = "tracing")]
        if self.init_logging {
            init_logging(LogFormat::from_env(), "info");
        }

        match main_config_file {
            Some(path) => load_config_file(&path),
            None => load_from_env(),
        }
    }
}

/// Load config from environment variables only.
pub fn load_from_env<C: DeserializeOwned>() -> Result<C, ConfigError> {
    use config::Config;

    Config::builder()
        .add_source(EnvSource)
        .build()
        .and_then(|c| c.try_deserialize::<C>())
        .map_err(|e| ConfigError::Parse(e.to_string()))
}

/// Load config from file with env var overrides.
pub fn load_config_file<C: DeserializeOwned>(path: &Path) -> Result<C, ConfigError> {
    use config::{Config, File};

    if !path.exists() {
        return Err(ConfigError::NotFound(path.to_path_buf()));
    }

    Config::builder()
        .add_source(File::from(path))
        .add_source(EnvSource)
        .build()
        .and_then(|c| c.try_deserialize())
        .map_err(|e| ConfigError::Parse(e.to_string()))
}

/// Custom environment source that maps APP_ENV/RUST_ENV to environment field.
#[derive(Debug, Clone)]
struct EnvSource;

impl config::Source for EnvSource {
    fn clone_into_box(&self) -> Box<dyn config::Source + Send + Sync> {
        Box::new(self.clone())
    }

    fn collect(&self) -> Result<config::Map<String, config::Value>, config::ConfigError> {
        use config::{Environment, Value, ValueKind};

        // Start with default environment source
        let mut map = Environment::default()
            .separator("__")
            .try_parsing(true)
            .collect()?;

        // Map APP_ENV/RUST_ENV to environment if not already set
        if !map.contains_key("environment") {
            if let Ok(val) = env::var("ENVIRONMENT")
                .or_else(|_| env::var("APP_ENV"))
                .or_else(|_| env::var("RUST_ENV"))
            {
                map.insert(
                    "environment".to_string(),
                    Value::new(None, ValueKind::String(val)),
                );
            }
        }

        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;
    use std::io::Write;

    #[derive(Debug, Deserialize, Default)]
    struct TestConfig {
        #[serde(default)]
        host: String,
        #[serde(default)]
        port: u16,
    }

    #[test]
    fn config_format_from_path() {
        assert_eq!(
            ConfigFormat::from_path("config.toml"),
            Some(ConfigFormat::Toml)
        );
        assert_eq!(
            ConfigFormat::from_path("config.yaml"),
            Some(ConfigFormat::Yaml)
        );
        assert_eq!(
            ConfigFormat::from_path("config.yml"),
            Some(ConfigFormat::Yaml)
        );
        assert_eq!(
            ConfigFormat::from_path("config.json"),
            Some(ConfigFormat::Json)
        );
        assert_eq!(
            ConfigFormat::from_path("settings.env"),
            Some(ConfigFormat::DotEnv)
        );
        assert_eq!(ConfigFormat::from_path("config.txt"), None);
        assert_eq!(ConfigFormat::from_path("noextension"), None);
        assert_eq!(ConfigFormat::from_path(".env"), None);
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
            "#,
        )
        .unwrap();

        let config: TestConfig = ConfigBuilder::new()
            .with_config_file(&config_path)
            .build()
            .unwrap();

        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
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
"#,
        )
        .unwrap();

        let config: TestConfig = ConfigBuilder::new()
            .with_config_file(&config_path)
            .build()
            .unwrap();

        assert_eq!(config.host, "192.168.1.1");
        assert_eq!(config.port, 9000);
    }

    #[test]
    fn config_builder_loads_json() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.json");

        std::fs::write(&config_path, r#"{"host": "10.0.0.1", "port": 5000}"#).unwrap();

        let config: TestConfig = ConfigBuilder::new()
            .with_config_file(&config_path)
            .build()
            .unwrap();

        assert_eq!(config.host, "10.0.0.1");
        assert_eq!(config.port, 5000);
    }

    #[test]
    fn config_builder_file_not_found() {
        let result: Result<TestConfig, _> = ConfigBuilder::new()
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

        let _: TestConfig = ConfigBuilder::new()
            .with_config_file(&env_path)
            .build()
            .unwrap();
    }
}
