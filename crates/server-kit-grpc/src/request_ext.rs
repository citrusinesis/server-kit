//! Request extension trait for easy metadata access.

use tonic::Request;

/// A type-safe header key.
///
/// Use predefined constants from the [`headers`] module for common headers,
/// or create custom keys with [`HeaderKey::new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeaderKey(&'static str);

impl HeaderKey {
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    pub const fn as_str(&self) -> &'static str {
        self.0
    }
}

/// Common header keys for gRPC requests.
pub mod headers {
    use super::HeaderKey;

    // Standard HTTP
    pub const AUTHORIZATION: HeaderKey = HeaderKey::new("authorization");
    pub const CONTENT_TYPE: HeaderKey = HeaderKey::new("content-type");
    pub const USER_AGENT: HeaderKey = HeaderKey::new("user-agent");
    pub const ACCEPT_LANGUAGE: HeaderKey = HeaderKey::new("accept-language");

    // gRPC-specific
    pub const GRPC_TIMEOUT: HeaderKey = HeaderKey::new("grpc-timeout");
    pub const GRPC_ENCODING: HeaderKey = HeaderKey::new("grpc-encoding");
    pub const GRPC_ACCEPT_ENCODING: HeaderKey = HeaderKey::new("grpc-accept-encoding");

    // Request Tracking
    pub const REQUEST_ID: HeaderKey = HeaderKey::new("x-request-id");
    pub const CORRELATION_ID: HeaderKey = HeaderKey::new("x-correlation-id");

    // W3C Trace Context
    pub const TRACEPARENT: HeaderKey = HeaderKey::new("traceparent");
    pub const TRACESTATE: HeaderKey = HeaderKey::new("tracestate");

    // B3 (Zipkin)
    pub const B3_TRACE_ID: HeaderKey = HeaderKey::new("x-b3-traceid");
    pub const B3_SPAN_ID: HeaderKey = HeaderKey::new("x-b3-spanid");
    pub const B3_PARENT_SPAN_ID: HeaderKey = HeaderKey::new("x-b3-parentspanid");
    pub const B3_SAMPLED: HeaderKey = HeaderKey::new("x-b3-sampled");
    pub const B3: HeaderKey = HeaderKey::new("b3");

    // Proxy
    pub const X_FORWARDED_FOR: HeaderKey = HeaderKey::new("x-forwarded-for");
    pub const X_FORWARDED_HOST: HeaderKey = HeaderKey::new("x-forwarded-host");
    pub const X_FORWARDED_PROTO: HeaderKey = HeaderKey::new("x-forwarded-proto");
    pub const X_REAL_IP: HeaderKey = HeaderKey::new("x-real-ip");

    // Authentication & Identity
    pub const API_KEY: HeaderKey = HeaderKey::new("x-api-key");
    pub const CLIENT_ID: HeaderKey = HeaderKey::new("x-client-id");
    pub const TENANT_ID: HeaderKey = HeaderKey::new("x-tenant-id");

    // Idempotency
    pub const IDEMPOTENCY_KEY: HeaderKey = HeaderKey::new("idempotency-key");
}

/// Extension trait for `tonic::Request<T>`.
///
/// Provides type-safe header access using [`HeaderKey`] constants.
pub trait RequestExt<T> {
    /// Get a header value using a type-safe [`HeaderKey`].
    fn header(&self, key: HeaderKey) -> Option<&str>;
}

impl<T> RequestExt<T> for Request<T> {
    fn header(&self, key: HeaderKey) -> Option<&str> {
        self.metadata()
            .get(key.as_str())
            .and_then(|v| v.to_str().ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_request_id() {
        let mut request = Request::new(());
        request
            .metadata_mut()
            .insert(headers::REQUEST_ID.as_str(), "test-id-123".parse().unwrap());

        assert_eq!(request.header(headers::REQUEST_ID), Some("test-id-123"));
    }

    #[test]
    fn header_authorization() {
        let mut request = Request::new(());
        request
            .metadata_mut()
            .insert(headers::AUTHORIZATION.as_str(), "Bearer token".parse().unwrap());

        assert_eq!(request.header(headers::AUTHORIZATION), Some("Bearer token"));
    }

    #[test]
    fn header_custom_key() {
        const CUSTOM: HeaderKey = HeaderKey::new("x-custom-header");

        let mut request = Request::new(());
        request
            .metadata_mut()
            .insert(CUSTOM.as_str(), "custom-value".parse().unwrap());

        assert_eq!(request.header(CUSTOM), Some("custom-value"));
    }

    #[test]
    fn header_returns_none_when_missing() {
        let request = Request::new(());

        assert_eq!(request.header(headers::REQUEST_ID), None);
        assert_eq!(request.header(headers::AUTHORIZATION), None);
    }

    #[test]
    fn header_key_equality() {
        const A: HeaderKey = HeaderKey::new("x-test");
        const B: HeaderKey = HeaderKey::new("x-test");
        const C: HeaderKey = HeaderKey::new("x-other");

        assert_eq!(A, B);
        assert_ne!(A, C);
    }
}
