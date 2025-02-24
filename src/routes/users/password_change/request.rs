use crate::utils::emails::send_multipart_email;
use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use deadpool_redis::Pool;
use sea_orm::DbConn;
use services::user::Query as UserQuery;

#[derive(serde::Deserialize, Debug)]
struct UserEmail {
    email: String,
}

#[tracing::instrument(name = "Requesting a password change", skip(db, redis_pool))]
#[actix_web::post("/email-verification")]
pub async fn request_password_change(
    db: Data<DbConn>,
    redis_pool: Data<Pool>,
    req: Json<UserEmail>,
) -> HttpResponse {
    match UserQuery::find_active_by_email(&db, req.email.clone()).await {
        Ok(_user) => match _user {
            Some(user) => match redis_pool.get().await {
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
                    HttpResponse::Ok().json(::types::SuccessResponse {
                            message: "Password reset instructions have been sent to your email address.Kindly take action before its expiration.".to_string()
                        })
                }
                Err(e) => {
                    tracing::event!(target: "backend", tracing::Level::ERROR, "{}", e);
                    return HttpResponse::InternalServerError().json(::types::ErrorResponse {
                        error: "Something happened. Please try again.".to_string(),
                    });
                }
            },
            None => {
                return HttpResponse::NotFound().json(::types::ErrorResponse {
                    error: format!("An active user with this email does not exist."),
                })
            }
        },
        Err(e) => {
            tracing::event!(target: "db", tracing::Level::ERROR, "User not found:{:#?}", e);
            HttpResponse::NotFound().json(::types::ErrorResponse {
                error: format!("An active user with this email does not exist."),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    #[actix_web::test]
    #[ignore]
    async fn request_password_change() -> Result<(), String> {
        todo!();
    }
}
