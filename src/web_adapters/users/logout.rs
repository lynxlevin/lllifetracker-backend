use actix_web::{post, HttpResponse};

#[tracing::instrument(name = "Log out user", skip(session))]
#[post("/logout")]
pub async fn log_out(session: actix_session::Session) -> HttpResponse {
    tracing::event!(target: "backend", tracing::Level::INFO, "User_id retrieved from the session.");
    session.purge();
    HttpResponse::Ok().json("You have successfully logged out")
}
