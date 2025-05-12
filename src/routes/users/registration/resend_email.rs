use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use deadpool_redis::Pool;
use sea_orm::DbConn;
use services::user::Query as UserQuery;
use utils::emails::send_multipart_email;

use crate::utils::{response_404, response_500};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    email: String,
}
#[tracing::instrument(
    name = "Resending registration confirmation email",
    skip(db, redis_pool, req)
)]
#[post("/resend-email")]
pub async fn resend_email(
    db: Data<DbConn>,
    redis_pool: Data<Pool>,
    req: Json<RequestBody>,
) -> HttpResponse {
    match UserQuery::find_inactive_by_email(&db, req.email.clone()).await {
        Ok(user) => match user {
            Some(user) => {
                match redis_pool.get().await {
                    Ok(ref mut redis_con) => {
                        send_multipart_email(
                            "Let's get you verified".to_string(),
                            user.id,
                            user.email,
                            user.first_name,
                            user.last_name,
                            "verification_email.html",
                            redis_con,
                        )
                        .await
                        .unwrap();

                        tracing::event!(target: "backend", tracing::Level::INFO, "Verification email re-sent successfully.");
                        HttpResponse::Ok().json(::types::SuccessResponse { message: "Account activation link has been sent to your email address. Kindly take action before its expiration".to_string() })
                    },
                    Err(e) => response_500(e)
                }
            },
            None => response_404("User with this email was not found. This happens if you have already activated this user.")
        },
        Err(e) => response_500(e)
    }
}

#[cfg(test)]
mod tests {
    #[actix_web::test]
    #[ignore]
    async fn resend_email() -> Result<(), String> {
        todo!();
    }
}
