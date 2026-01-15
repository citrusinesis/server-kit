//! Authentication interceptor.

use tonic::{Request, Status};

/// Trait for validating authentication tokens.
pub trait TokenValidator: Clone + Send + Sync + 'static {
    /// Validate a token, returning Ok(()) if valid or an error status.
    fn validate(&self, token: &str) -> Result<(), Status>;
}

/// Authentication interceptor.
///
/// Extracts the bearer token from the `authorization` header
/// and validates it using the provided validator.
///
/// # Example
///
/// ```ignore
/// use server_kit_grpc::interceptor::{AuthInterceptor, TokenValidator};
/// use tonic::Status;
///
/// #[derive(Clone)]
/// struct MyValidator {
///     secret: String,
/// }
///
/// impl TokenValidator for MyValidator {
///     fn validate(&self, token: &str) -> Result<(), Status> {
///         if token == self.secret {
///             Ok(())
///         } else {
///             Err(Status::unauthenticated("Invalid token"))
///         }
///     }
/// }
///
/// let interceptor = AuthInterceptor::new(MyValidator { secret: "secret".into() });
/// let svc = MyServiceServer::with_interceptor(my_impl, interceptor.into_fn());
/// ```
#[derive(Clone)]
pub struct AuthInterceptor<V> {
    validator: V,
}

impl<V: TokenValidator> AuthInterceptor<V> {
    pub fn new(validator: V) -> Self {
        Self { validator }
    }

    /// Create an interceptor function for use with `with_interceptor`.
    pub fn into_fn(self) -> impl Fn(Request<()>) -> Result<Request<()>, Status> + Clone {
        move |req: Request<()>| {
            let token = req
                .metadata()
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.strip_prefix("Bearer "))
                .ok_or_else(|| Status::unauthenticated("Missing authorization header"))?;

            self.validator.validate(token)?;
            Ok(req)
        }
    }
}

/// Create a simple bearer token validation interceptor.
///
/// # Example
///
/// ```ignore
/// use server_kit_grpc::interceptor::bearer_auth;
///
/// let interceptor = bearer_auth(|token| {
///     if token == "valid-token" {
///         Ok(())
///     } else {
///         Err(tonic::Status::unauthenticated("Invalid token"))
///     }
/// });
///
/// let svc = MyServiceServer::with_interceptor(my_impl, interceptor);
/// ```
pub fn bearer_auth<F>(validate: F) -> impl Fn(Request<()>) -> Result<Request<()>, Status> + Clone
where
    F: Fn(&str) -> Result<(), Status> + Clone + Send + Sync + 'static,
{
    move |req: Request<()>| {
        let token = req
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| Status::unauthenticated("Missing authorization header"))?;

        validate(token)?;
        Ok(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tonic::Code;

    #[test]
    fn auth_interceptor_valid_token() {
        let interceptor = bearer_auth(|token| {
            if token == "valid" {
                Ok(())
            } else {
                Err(Status::unauthenticated("invalid"))
            }
        });

        let mut req = Request::new(());
        req.metadata_mut()
            .insert("authorization", "Bearer valid".parse().unwrap());

        assert!(interceptor(req).is_ok());
    }

    #[test]
    fn auth_interceptor_invalid_token() {
        let interceptor = bearer_auth(|token| {
            if token == "valid" {
                Ok(())
            } else {
                Err(Status::unauthenticated("invalid"))
            }
        });

        let mut req = Request::new(());
        req.metadata_mut()
            .insert("authorization", "Bearer invalid".parse().unwrap());

        let result = interceptor(req);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), Code::Unauthenticated);
    }

    #[test]
    fn auth_interceptor_missing_token() {
        let interceptor = bearer_auth(|_| Ok(()));
        let req = Request::new(());

        let result = interceptor(req);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), Code::Unauthenticated);
    }

    #[test]
    fn auth_interceptor_wrong_scheme() {
        let interceptor = bearer_auth(|_| Ok(()));

        let mut req = Request::new(());
        req.metadata_mut()
            .insert("authorization", "Basic dXNlcjpwYXNz".parse().unwrap());

        let result = interceptor(req);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), Code::Unauthenticated);
    }

    #[test]
    fn auth_interceptor_struct() {
        #[derive(Clone)]
        struct TestValidator;

        impl TokenValidator for TestValidator {
            fn validate(&self, token: &str) -> Result<(), Status> {
                if token == "secret" {
                    Ok(())
                } else {
                    Err(Status::unauthenticated("bad token"))
                }
            }
        }

        let interceptor = AuthInterceptor::new(TestValidator).into_fn();

        let mut req = Request::new(());
        req.metadata_mut()
            .insert("authorization", "Bearer secret".parse().unwrap());

        assert!(interceptor(req).is_ok());
    }
}
