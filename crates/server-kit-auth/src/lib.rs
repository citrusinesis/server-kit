//! Authentication middleware for server-kit.
//!
//! Provides JWT authentication and authorization utilities.
//!
//! # Example
//!
//! ```rust,ignore
//! use axum::{Router, routing::get};
//! use server_kit_auth::{AuthExt, JwtConfig};
//!
//! let config = JwtConfig::new("your-secret-key");
//!
//! let app = Router::new()
//!     .route("/protected", get(handler))
//!     .with_jwt_auth(config);
//! ```

#[cfg(feature = "jwt")]
mod jwt;

mod error;
mod layer;

pub use error::AuthError;
pub use layer::{AuthExt, AuthLayer};

#[cfg(feature = "jwt")]
pub use jwt::{Claims, JwtConfig};
