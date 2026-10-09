use std::fmt::{Debug, Display};

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

impl Display for UseCaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::BadRequest(message) => message,
                Self::Unauthorized => "Unauthorized",
                Self::Forbidden => "Forbidden",
                Self::NotFound(message) => message,
                Self::Conflict(message) => message,
                Self::Gone => "Gone",
                Self::InternalServerError(message) => message,
            }
        )
    }
}
