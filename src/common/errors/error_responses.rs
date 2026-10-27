use std::fmt::{Debug, Display};

use actix_web::{error, http::StatusCode, HttpResponse};
use tracing::{event, Level};

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

/// Bad Request
pub fn response_400(message: &str) -> HttpResponse {
    HttpResponse::from_error(ErrorResponse { message: message.to_string(), status_code: StatusCode::BAD_REQUEST })
}

/// Unauthorized
pub fn response_401() -> HttpResponse {
    HttpResponse::from_error(ErrorResponse {
        message: "You are not authorized.".to_string(),
        status_code: StatusCode::UNAUTHORIZED,
    })
}

/// Forbidden
pub fn response_403(message: &str) -> HttpResponse {
    HttpResponse::from_error(ErrorResponse { message: message.to_string(), status_code: StatusCode::FORBIDDEN })
}

/// Not Found
pub fn response_404(message: impl ToString) -> HttpResponse {
    HttpResponse::from_error(ErrorResponse { message: message.to_string(), status_code: StatusCode::NOT_FOUND })
}

/// Internal Server Error
pub fn response_500<T: Debug>(e: T) -> HttpResponse {
    event!(target: "backend", Level::ERROR, "{:?}", e);
    HttpResponse::from_error(ErrorResponse {
        message: "Some unexpected error happened. Please try again later.".to_string(),
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
    })
}
