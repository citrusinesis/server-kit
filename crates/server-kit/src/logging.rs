//! Logging initialization helpers.
//!
//! Provides convenient functions to initialize tracing with different formats.

use std::{env, str::FromStr};

/// Log output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogFormat {
    /// Human-readable text format (default).
    #[default]
    Text,
    /// JSON format for structured logging.
    Json,
}

impl FromStr for LogFormat {
    type Err = std::convert::Infallible;

    /// Parse from string (case-insensitive).
    ///
    /// - "json" -> Json
    /// - anything else -> Text
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "json" => Self::Json,
            _ => Self::Text,
        })
    }
}

impl LogFormat {
    /// Load from `LOG_FORMAT` environment variable.
    ///
    /// Defaults to `Text` if not set or invalid.
    pub fn from_env() -> Self {
        env::var("LOG_FORMAT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_default()
    }
}

/// Initialize tracing with the specified format.
///
/// # Arguments
///
/// * `format` - Log output format (Text or Json)
/// * `filter` - Log filter directive (e.g., "info", "debug", "my_app=debug")
#[cfg(feature = "tracing")]
pub fn init_logging(format: LogFormat, filter: &str) {
    use tracing_subscriber::{fmt, EnvFilter};

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter));

    let _ = match format {
        LogFormat::Text => fmt().with_env_filter(env_filter).try_init(),
        LogFormat::Json => fmt()
            .json()
            .with_current_span(false)
            .with_env_filter(env_filter)
            .try_init(),
    };
}

/// Initialize logging from environment variables.
///
/// Reads `LOG_FORMAT` for format (text/json) and `RUST_LOG` for filter directives.
#[cfg(feature = "tracing")]
pub fn init_logging_from_env() {
    init_logging(LogFormat::from_env(), "info");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_format_from_str() {
        assert_eq!("json".parse::<LogFormat>().unwrap(), LogFormat::Json);
        assert_eq!("JSON".parse::<LogFormat>().unwrap(), LogFormat::Json);
        assert_eq!("Json".parse::<LogFormat>().unwrap(), LogFormat::Json);
        assert_eq!("text".parse::<LogFormat>().unwrap(), LogFormat::Text);
        assert_eq!("TEXT".parse::<LogFormat>().unwrap(), LogFormat::Text);
        assert_eq!("anything".parse::<LogFormat>().unwrap(), LogFormat::Text);
        assert_eq!("".parse::<LogFormat>().unwrap(), LogFormat::Text);
    }

    #[test]
    fn log_format_default() {
        assert_eq!(LogFormat::default(), LogFormat::Text);
    }
}
