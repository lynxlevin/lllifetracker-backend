use actix_web::{post, HttpResponse};
use utils::auth::session::get_user_id;

use crate::utils::response_400;

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
        Err(_) => response_400(
            "We currently have some issues. Kindly try again and ensure you are logged in.",
        ),
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
