pub mod my_way;

#[derive(Debug)]
pub enum UseCaseError {
    BadRequest(String),          // 400
    Unauthorized,                // 401
    Forbidden,                   // 403
    NotFound(String),            // 404
    Conflict(String),            // 409
    InternalServerError(String), // 500
}
