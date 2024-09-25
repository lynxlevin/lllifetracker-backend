use crate::{entities::user, services::user as user_service, utils::emails::send_multipart_email};
use actix_web::{
    web::{Data, Json},
    HttpResponse,
};

#[derive(serde::Deserialize, Debug)]
pub struct UserEmail {
    email: String,
}

#[tracing::instrument(name = "Requesting a password change", skip(data))]
#[actix_web::post("")]
pub async fn request(data: Data<crate::startup::AppState>, req: Json<UserEmail>) -> HttpResponse {
    match user_service::Query::find_active_by_email(&data.conn, req.email.clone()).await {
        Ok(_user) => match _user {
            Some(user) => {
                let user_model: user::ActiveModel = user.into();
                match data.redis_pool.get().await {
                    Ok(ref mut redis_conn) => {
                        send_multipart_email(
                            "Password Reset Instructions".to_string(),
                            user_model.id.unwrap(),
                            user_model.email.unwrap(),
                            user_model.first_name.unwrap(),
                            user_model.last_name.unwrap(),
                            "password_reset_email.html",
                            redis_conn,
                        )
                        .await
                        .unwrap();
                        HttpResponse::Ok().json(crate::types::SuccessResponse {
                            message: "Password reset instructions have been sent to your email address.Kindly take action before its expiration.".to_string()
                        })
                    }
                    Err(e) => {
                        tracing::event!(target: "backend", tracing::Level::ERROR, "{}", e);
                        return HttpResponse::InternalServerError().json(
                            crate::types::ErrorResponse {
                                error: "Something happened. Please try again.".to_string(),
                            },
                        );
                    }
                }
            }
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
