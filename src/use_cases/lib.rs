use std::fmt::Debug;

pub mod journal;
pub mod my_way;
pub mod notification;
pub mod tags;
pub mod users;

#[derive(Debug)]
pub enum UseCaseError {
    BadRequest(String),          // 400
    Unauthorized,                // 401
    Forbidden,                   // 403
    NotFound(String),            // 404
    Conflict(String),            // 409
    Gone,                        // 410
    InternalServerError(String), // 500
}

pub(crate) fn error_500(e: impl Debug) -> UseCaseError {
    UseCaseError::InternalServerError(format!("{:?}", e))
}
