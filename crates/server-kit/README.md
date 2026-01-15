# server-kit

Shared utilities for `server-kit-rest` and `server-kit-grpc`.

## Installation

```toml
[dependencies]
server-kit = { version = "0.1" }
```

### Features

| Feature   | Description             | Default |
| --------- | ----------------------- | ------- |
| `tracing` | Logging initialization  | No      |

## Configuration Builder

Load configuration from environment variables and config files.

```rust
use server_kit::ConfigBuilder;
use serde::Deserialize;

#[derive(Deserialize)]
struct MyConfig {
    host: String,
    port: u16,
}

let config: MyConfig = ConfigBuilder::new()
    .with_dotenv()                    // Load .env file
    .with_config_file("config.toml")  // Load config file
    .with_logging_from_env()          // Initialize logging (requires tracing feature)
    .build()?;
```

### Supported Formats

| Extension        | Format | Behavior                      |
| ---------------- | ------ | ----------------------------- |
| `.env`           | dotenv | Load as environment variables |
| `.toml`          | TOML   | Parse as config               |
| `.yaml` / `.yml` | YAML   | Parse as config               |
| `.json`          | JSON   | Parse as config               |

### Multiple Config Files

```rust
let config: MyConfig = ConfigBuilder::new()
    .with_dotenv()                    // .env
    .with_config_file(".env.local")   // Additional env vars
    .with_config_file("config.yaml")  // Main config (last file wins)
    .build()?;
```

- `.env` files are loaded into environment variables in order
- Config files (toml/yaml/json) use only the last one specified
- Environment variables override config file values

## Environment

Application environment type with parsing from env vars.

```rust
use server_kit::Environment;

// Load from APP_ENV or RUST_ENV
let env = Environment::from_env();

if env.is_production() {
    // Production-specific logic
}
```

| Value                    | Result        |
| ------------------------ | ------------- |
| `production` / `prod`    | `Production`  |
| anything else            | `Development` |

## Logging (feature: `tracing`)

Initialize tracing subscriber from environment variables.

```rust
use server_kit::{init_logging_from_env, LogFormat};

init_logging_from_env();  // Uses LOG_FORMAT and RUST_LOG

// Or manually
use server_kit::init_logging;
init_logging(LogFormat::Json, "info");
```

| Environment Variable | Description           | Default |
| -------------------- | --------------------- | ------- |
| `LOG_FORMAT`         | `text` or `json`      | `text`  |
| `RUST_LOG`           | Log filter directive  | `info`  |

## License

MIT
