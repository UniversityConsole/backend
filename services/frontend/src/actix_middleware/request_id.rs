use std::future::{ready, Ready};

use actix_service::{forward_ready, Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{Error, HttpMessage};
use futures_util::future::LocalBoxFuture;
use tracing_actix_web::RequestId;

/// The header set by the middleware.
pub const REQUEST_ID_HEADER: &str = "x-uc-request-id";

/// The wrapper for request ID headers.
pub struct RequestIdHeader;

impl<S, B> Transform<S, ServiceRequest> for RequestIdHeader
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestIdHeaderMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdHeaderMiddleware { service }))
    }
}

pub struct RequestIdHeaderMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdHeaderMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let request_id = req
            .extensions()
            .get::<RequestId>()
            .expect("request id extension not injected")
            .to_string();

        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;
            res.headers_mut().insert(
                HeaderName::from_static(REQUEST_ID_HEADER),
                HeaderValue::from_str(request_id.as_str()).expect("invalid request id"),
            );
            Ok(res)
        })
    }
}
