use actix_web::{post, HttpResponse};

use crate::{users::utils::auth::session::get_user_id, utils::response_400};

#[tracing::instrument(name = "Log out user", skip(session))]
#[post("/logout")]
pub async fn log_out(session: actix_session::Session) -> HttpResponse {
    match get_user_id(&session).await {
        Ok(_) => {
            tracing::event!(target: "backend", tracing::Level::INFO, "User_id retrieved from the session.");
            session.purge();
            HttpResponse::Ok().json("You have successfully logged out")
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
