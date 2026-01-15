//! Logging initialization.

use std::{env, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogFormat {
    #[default]
    Text,
    Json,
}

impl FromStr for LogFormat {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "json" => Self::Json,
            _ => Self::Text,
        })
    }
}

impl LogFormat {
    pub fn from_env() -> Self {
        env::var("LOG_FORMAT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_default()
    }
}

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
