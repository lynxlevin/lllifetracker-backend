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

use crate::request_logger::RequestStart;

pub struct ResponseLogger;

impl<S: 'static, B> Transform<S, ServiceRequest> for ResponseLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ResponseLoggerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ResponseLoggerMiddleware { service: Rc::new(service) }))
    }
}

pub struct ResponseLoggerMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ResponseLoggerMiddleware<S>
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
            let req_start = req
                .extensions()
                .get::<RequestStart>()
                .and_then(|start| Some(start.start));
            let res = svc.call(req).await?;
            let now = Utc::now().timestamp_micros();
            let duration = req_start
                .and_then(|start| Some((now - start).to_string()))
                .unwrap_or("unknown".to_string());
            event!(
                Level::INFO,
                "ResponseLogger: {{ status: {}, duration_micro: {} }}",
                res.status(),
                duration,
            );
            Ok(res)
        })
    }
}
