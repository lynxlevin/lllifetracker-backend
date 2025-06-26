use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use common::settings::types::Settings;
use db_adapters::user_adapter::{UserAdapter, UserFilter, UserQuery};
use deadpool_redis::Pool;
use sea_orm::DbConn;

use crate::utils::{emails::send_multipart_email, response_404, response_500};

#[derive(serde::Deserialize, Debug)]
struct UserEmail {
    email: String,
}

#[tracing::instrument(name = "Requesting a password change", skip(db, redis_pool, settings))]
#[actix_web::post("/email-verification")]
pub async fn request_password_change(
    db: Data<DbConn>,
    redis_pool: Data<Pool>,
    req: Json<UserEmail>,
    settings: Data<Settings>,
) -> HttpResponse {
    match UserAdapter::init(&db)
        .filter_eq_is_active(true)
        .get_by_email(req.email.clone())
        .await
    {
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
                        &settings,
                    )
                    .await
                    .unwrap();
                    HttpResponse::Ok().json("Password reset instructions have been sent to your email address.Kindly take action before its expiration.")
                }
                Err(e) => response_500(e),
            },
            None => response_404("An active user with this email does not exist."),
        },
        Err(_) => response_404("An active user with this email does not exist."),
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
