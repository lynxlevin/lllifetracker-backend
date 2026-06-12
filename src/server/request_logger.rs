use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use chrono::Utc;
use futures::future::LocalBoxFuture;
use tracing::{event, Level};

pub struct RequestLogger;

impl<S: 'static, B> Transform<S, ServiceRequest> for RequestLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestLoggerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestLoggerMiddleware { service: Rc::new(service) }))
    }
}

pub struct RequestLoggerMiddleware<S> {
    service: Rc<S>,
}

pub struct RequestStart {
    pub start: i64,
}

impl<S, B> Service<ServiceRequest> for RequestLoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        Box::pin(async move {
            let now = Utc::now().timestamp_micros();
            event!(
                Level::INFO,
                "RequestLogger: {{ path: {}, query: {} }}",
                req.path(),
                req.query_string()
            );
            req.extensions_mut().insert(RequestStart { start: now });
            let res = svc.call(req).await?;
            Ok(res)
        })
    }
}
