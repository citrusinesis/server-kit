//! Application environment types.

use serde::{Deserialize, Serialize};
use std::env;
use std::str::FromStr;

/// Application environment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub enum Environment {
    #[default]
    Development,
    Production,
}

impl FromStr for Environment {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "production" | "prod" => Self::Production,
            _ => Self::Development,
        })
    }
}

impl<'de> Deserialize<'de> for Environment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(s.parse().unwrap())
    }
}

impl Environment {
    /// Load from `APP_ENV` or `RUST_ENV` environment variable.
    pub fn from_env() -> Self {
        env::var("APP_ENV")
            .or_else(|_| env::var("RUST_ENV"))
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_default()
    }

    pub fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }

    pub fn is_development(&self) -> bool {
        matches!(self, Self::Development)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_from_str_production() {
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
    }

    #[test]
    fn environment_from_str_development() {
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
    fn environment_default() {
        assert_eq!(Environment::default(), Environment::Development);
    }
}
