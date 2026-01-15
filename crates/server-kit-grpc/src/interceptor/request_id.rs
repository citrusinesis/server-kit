//! Request ID interceptor and layer.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use tonic::{Request, Status};
use tower::{Layer, Service};
use uuid::Uuid;

pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Tower layer that ensures every request has a request ID.
#[derive(Clone, Copy, Default)]
pub struct RequestIdLayer;

impl RequestIdLayer {
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for RequestIdLayer {
    type Service = RequestIdService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequestIdService { inner }
    }
}

#[derive(Clone)]
pub struct RequestIdService<S> {
    inner: S,
}

impl<S, ReqBody> Service<http::Request<ReqBody>> for RequestIdService<S>
where
    S: Service<http::Request<ReqBody>> + Clone + Send + 'static,
    S::Future: Send,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: http::Request<ReqBody>) -> Self::Future {
        if req.headers().get(REQUEST_ID_HEADER).is_none() {
            let request_id = Uuid::new_v4().to_string();
            req.headers_mut().insert(
                REQUEST_ID_HEADER,
                request_id.parse().expect("UUID is valid header value"),
            );
        }

        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move { inner.call(req).await })
    }
}

/// gRPC interceptor that ensures every request has a request ID.
pub fn request_id_interceptor(mut req: Request<()>) -> Result<Request<()>, Status> {
    if req.metadata().get(REQUEST_ID_HEADER).is_none() {
        let request_id = Uuid::new_v4().to_string();
        req.metadata_mut()
            .insert(REQUEST_ID_HEADER, request_id.parse().unwrap());
    }
    Ok(req)
}

#[derive(Clone, Copy, Default)]
pub struct RequestIdInterceptor;

impl RequestIdInterceptor {
    pub fn new() -> Self {
        Self
    }

    pub fn intercept(&self, req: Request<()>) -> Result<Request<()>, Status> {
        request_id_interceptor(req)
    }

    pub fn into_fn(self) -> impl Fn(Request<()>) -> Result<Request<()>, Status> + Clone {
        move |req| request_id_interceptor(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Request as HttpRequest;
    use std::convert::Infallible;
    use tower::ServiceExt;

    #[test]
    fn interceptor_adds_header() {
        let req = Request::new(());
        let result = request_id_interceptor(req).unwrap();

        let request_id = result.metadata().get(REQUEST_ID_HEADER);
        assert!(request_id.is_some());

        let id_str = request_id.unwrap().to_str().unwrap();
        assert!(Uuid::parse_str(id_str).is_ok());
    }

    #[test]
    fn interceptor_preserves_existing() {
        let mut req = Request::new(());
        req.metadata_mut()
            .insert(REQUEST_ID_HEADER, "existing-id".parse().unwrap());

        let result = request_id_interceptor(req).unwrap();

        let request_id = result.metadata().get(REQUEST_ID_HEADER).unwrap();
        assert_eq!(request_id.to_str().unwrap(), "existing-id");
    }

    #[test]
    fn interceptor_struct_works() {
        let interceptor = RequestIdInterceptor::new();
        let req = Request::new(());

        let result = interceptor.intercept(req).unwrap();
        assert!(result.metadata().get(REQUEST_ID_HEADER).is_some());
    }

    #[derive(Clone)]
    struct MockService;

    impl<B> Service<HttpRequest<B>> for MockService {
        type Response = http::Response<String>;
        type Error = Infallible;
        type Future = std::future::Ready<Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: HttpRequest<B>) -> Self::Future {
            let request_id = req
                .headers()
                .get(REQUEST_ID_HEADER)
                .map(|v| v.to_str().unwrap_or("-"))
                .unwrap_or("-")
                .to_string();

            std::future::ready(Ok(http::Response::new(request_id)))
        }
    }

    #[tokio::test]
    async fn layer_adds_request_id() {
        let layer = RequestIdLayer::new();
        let service = layer.layer(MockService);

        let req = HttpRequest::builder().uri("/test").body(()).unwrap();

        let response = service.oneshot(req).await.unwrap();
        let body = response.into_body();

        assert!(Uuid::parse_str(&body).is_ok(), "Expected UUID, got: {}", body);
    }

    #[tokio::test]
    async fn layer_preserves_existing_request_id() {
        let layer = RequestIdLayer::new();
        let service = layer.layer(MockService);

        let req = HttpRequest::builder()
            .uri("/test")
            .header(REQUEST_ID_HEADER, "my-custom-id")
            .body(())
            .unwrap();

        let response = service.oneshot(req).await.unwrap();
        let body = response.into_body();

        assert_eq!(body, "my-custom-id");
    }

    #[test]
    fn layer_is_clone() {
        fn assert_clone<T: Clone>() {}
        assert_clone::<RequestIdLayer>();
    }
}
