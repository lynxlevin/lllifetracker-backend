use std::fmt::Debug;

use actix_web::HttpResponse;
use types::{ErrorResponse, INTERNAL_SERVER_ERROR_MESSAGE};

///Bad Request
pub fn response_400(error_message: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(types::ErrorResponse {
        error: error_message.to_string(),
    })
}

/// Unauthorized
pub fn response_401() -> HttpResponse {
    HttpResponse::Unauthorized().json(ErrorResponse {
        error: "You are not logged in.".to_string(),
    })
}

/// NotFound
pub fn response_404(error_message: &str) -> HttpResponse {
    HttpResponse::NotFound().json(types::ErrorResponse {
        error: error_message.to_string(),
    })
}

/// Conflict
pub fn response_409(error_message: &str) -> HttpResponse {
    HttpResponse::Conflict().json(types::ErrorResponse {
        error: error_message.to_string(),
    })
}

/// Internal Server Error: with logging
pub fn response_500<T: Debug>(e: T) -> HttpResponse {
    tracing::event!(target: "backend", tracing::Level::ERROR, "{:#?}", e);
    HttpResponse::InternalServerError().json(types::ErrorResponse {
        error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
    })
}
