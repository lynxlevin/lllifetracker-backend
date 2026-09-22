use std::fmt::{Debug, Display};

use actix_web::{error, http::StatusCode, HttpResponse};
use tracing::{event, Level};

pub mod auth;
pub mod emails;

#[derive(Debug)]
struct ErrorResponse {
    pub message: String,
    pub status_code: StatusCode,
}
impl Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl error::ResponseError for ErrorResponse {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code).body(self.to_string())
    }
}

///Bad Request
pub fn response_400(error_message: &str) -> HttpResponse {
    HttpResponse::from_error(ErrorResponse {
        message: error_message.to_string(),
        status_code: StatusCode::BAD_REQUEST,
    })
}

/// Unauthorized
pub fn response_401() -> HttpResponse {
    HttpResponse::from_error(ErrorResponse {
        message: "You are not logged in.".to_string(),
        status_code: StatusCode::UNAUTHORIZED,
    })
}

/// NotFound
pub fn response_404(error_message: &str) -> HttpResponse {
    HttpResponse::from_error(ErrorResponse {
        message: error_message.to_string(),
        status_code: StatusCode::NOT_FOUND,
    })
}

/// Conflict
pub fn response_409(error_message: &str) -> HttpResponse {
    HttpResponse::from_error(ErrorResponse {
        message: error_message.to_string(),
        status_code: StatusCode::CONFLICT,
    })
}

/// Internal Server Error: with logging
pub fn response_500<T: Debug>(e: T) -> HttpResponse {
    event!(target: "backend", Level::ERROR, "{:?}", e);
    HttpResponse::from_error(ErrorResponse {
        message: "Some unexpected error happened. Please try again later.".to_string(),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    })
}
