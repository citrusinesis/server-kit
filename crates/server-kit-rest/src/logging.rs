//! Logging initialization helpers.

#[cfg(feature = "tracing")]
pub use server_kit::init_logging_from_env;

#[cfg(test)]
mod tests {
    use server_kit::LogFormat;

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
