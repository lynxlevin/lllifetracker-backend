use std::{
    fmt,
    future::{ready, Ready},
    io::Read,
    rc::Rc,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::HeaderValue,
    Error,
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
            let req_start = Utc::now().timestamp_micros();
            let mut http = Http {
                path: req.path().to_string(),
                method: req.method().to_string(),
                useragent: req
                    .headers()
                    .get("user-agent")
                    .and_then(header_value_to_string)
                    .unwrap_or("null".to_string()),
                referer: req
                    .headers()
                    .get("referer")
                    .and_then(header_value_to_string)
                    .unwrap_or("null".to_string()),
                status_code: String::new(),
            };
            let query = req.query_string().to_string();

            let res = svc.call(req).await?;

            let req_end = Utc::now().timestamp_micros();
            http.status_code = res.status().to_string();
            event!(
                Level::INFO,
                "RequestLogger: {{ duration_micro: {}, http: {}, query: {}  }}",
                (req_end - req_start).to_string(),
                http,
                query
            );
            Ok(res)
        })
    }
}

struct Http {
    path: String,
    method: String,
    useragent: String,
    referer: String,
    status_code: String,
}
impl fmt::Display for Http {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{{ path: {}, method: {}, useragent: {}, referer: {}, status_code: {} }}",
            self.path, self.method, self.useragent, self.referer, self.status_code,
        )
    }
}

fn header_value_to_string(value: &HeaderValue) -> Option<String> {
    let mut buff = String::new();
    let result = value.as_bytes().read_to_string(&mut buff);
    if result.is_ok() {
        Some(buff)
    } else {
        None
    }
}
