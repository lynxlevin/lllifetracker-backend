use crate::{services::user::Query as UserQuery, utils::emails::send_multipart_email};
use actix_web::{
    web::{Data, Json},
    HttpResponse,
};

#[derive(serde::Deserialize, Debug)]
struct UserEmail {
    email: String,
}

#[tracing::instrument(name = "Requesting a password change", skip(data))]
#[actix_web::post("/email-verification")]
pub async fn request_password_change(
    data: Data<crate::startup::AppState>,
    req: Json<UserEmail>,
) -> HttpResponse {
    match UserQuery::find_active_by_email(&data.conn, req.email.clone()).await {
        Ok(_user) => match _user {
            Some(user) => match data.redis_pool.get().await {
                Ok(ref mut redis_con) => {
                    send_multipart_email(
                        "Password Reset Instructions".to_string(),
                        user.id,
                        user.email,
                        user.first_name,
                        user.last_name,
                        "password_reset_email.html",
                        redis_con,
                    )
                    .await
                    .unwrap();
                    HttpResponse::Ok().json(crate::types::SuccessResponse {
                            message: "Password reset instructions have been sent to your email address.Kindly take action before its expiration.".to_string()
                        })
                }
                Err(e) => {
                    tracing::event!(target: "backend", tracing::Level::ERROR, "{}", e);
                    return HttpResponse::InternalServerError().json(crate::types::ErrorResponse {
                        error: "Something happened. Please try again.".to_string(),
                    });
                }
            },
            None => {
                return HttpResponse::NotFound().json(crate::types::ErrorResponse {
                    error: format!("An active user with this email does not exist."),
                })
            }
        },
        Err(e) => {
            tracing::event!(target: "db", tracing::Level::ERROR, "User not found:{:#?}", e);
            HttpResponse::NotFound().json(crate::types::ErrorResponse {
                error: format!("An active user with this email does not exist."),
            })
        }
    }
}
