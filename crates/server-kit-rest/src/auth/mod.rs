//! Authentication middleware.

mod error;
mod layer;

#[cfg(feature = "jwt")]
mod jwt;

pub use error::AuthError;
pub use layer::{AuthExt, AuthLayer, TokenValidator};

#[cfg(feature = "jwt")]
pub use jwt::{Claims, JwtConfig};
