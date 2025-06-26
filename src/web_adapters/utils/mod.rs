use std::fmt::Debug;

use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};
use tracing::{event, Level};

pub mod auth;
pub mod emails;

#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

///Bad Request
pub fn response_400(error_message: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(ErrorResponse {
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
    HttpResponse::NotFound().json(ErrorResponse {
        error: error_message.to_string(),
    })
}

/// Conflict
pub fn response_409(error_message: &str) -> HttpResponse {
    HttpResponse::Conflict().json(ErrorResponse {
        error: error_message.to_string(),
    })
}

/// Internal Server Error: with logging
pub fn response_500<T: Debug>(e: T) -> HttpResponse {
    event!(target: "backend", Level::ERROR, "{:?}", e);
    HttpResponse::InternalServerError().json(ErrorResponse {
        error: "Some unexpected error happened. Please try again later.".to_string(),
    })
}
