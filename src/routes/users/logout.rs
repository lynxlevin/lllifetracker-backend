use utils::auth::session::get_user_id;
use actix_web::{post, HttpResponse};

#[tracing::instrument(name = "Log out user", skip(session))]
#[post("/logout")]
pub async fn log_out(session: actix_session::Session) -> HttpResponse {
    match get_user_id(&session).await {
        Ok(_) => {
            tracing::event!(target: "backend", tracing::Level::INFO, "User_id retrieved from the session.");
            session.purge();
            HttpResponse::Ok().json(::types::SuccessResponse {
                message: "You have successfully logged out".to_string(),
            })
        }
        Err(e) => {
            tracing::event!(target: "backend", tracing::Level::ERROR, "Failed to get user from session: {:#?}", e);
            HttpResponse::BadRequest().json(::types::ErrorResponse {
                error:
                    "We currently have some issues. Kindly try again and ensure you are logged in."
                        .to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    #[actix_web::test]
    #[ignore]
    async fn log_out() -> Result<(), String> {
        todo!();
    }
}
