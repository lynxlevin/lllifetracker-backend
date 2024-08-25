use actix_web::{post, HttpResponse};

#[tracing::instrument(name = "Log out user", skip(session))]
#[post("/logout")]
pub async fn log_out(session: actix_session::Session) -> HttpResponse {
    println!("{:?}", session.entries());
    match session_user_id(&session).await {
        Ok(_) => {
            tracing::event!(target: "backend", tracing::Level::INFO, "User_id retrieved from the session.");
            session.purge();
            HttpResponse::Ok().json(crate::types::SuccessResponse {
                message: "You have successfully logged out".to_string(),
            })
        }
        Err(e) => {
            tracing::event!(target: "backend", tracing::Level::ERROR, "Failed to get user from session: {:#?}", e);
            HttpResponse::BadRequest().json(crate::types::ErrorResponse {
                error:
                    "We currently have some issues. Kindly try again and ensure you are logged in."
                        .to_string(),
            })
        }
    }
}

async fn session_user_id(session: &actix_session::Session) -> Result<uuid::Uuid, String> {
    match session.get(crate::types::USER_ID_KEY) {
        Ok(user_id) => match user_id {
            None => Err("You are not authenticated".to_string()),
            Some(id) => {
                println!("your user_id: {}", id);
                Ok(id)
            }
        },
        Err(e) => Err(format!("{e}")),
    }
}
