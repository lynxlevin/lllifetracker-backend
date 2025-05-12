use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use deadpool_redis::Pool;
use sea_orm::DbConn;

use ::types;
use services::user as user_service;
use utils::auth::{password, tokens::verify_confirmation_token_pasetor};

use crate::utils::{response_400, response_500};

#[derive(serde::Deserialize)]
struct Parameters {
    token: String,
    password: String,
}

#[tracing::instrument(name = "Changing user's password", skip(db, redis_pool, req))]
#[post("")]
pub async fn submit_password_change(
    db: Data<DbConn>,
    redis_pool: Data<Pool>,
    req: Json<Parameters>,
) -> HttpResponse {
    match redis_pool.get().await {
        Ok(ref mut redis_con) => {
            match verify_confirmation_token_pasetor(req.token.clone(), redis_con, Some(true)).await
            {
                Ok(confirmation_token) => {
                    let hashed_password = password::hash(req.password.as_bytes()).await;
                    match user_service::Mutation::update_user_password(&db, confirmation_token.user_id, hashed_password).await {
                        Ok(_) => {
                            HttpResponse::Ok().json(types::SuccessResponse {
                                message: "Your password has been changed successfully. Kindly login with the new password"
                                    .to_string(),
                            })
                        }
                        Err(e) => response_500(e)
                    }
                }
                Err(_) => response_400(
                    "It appears that your password request token has expired or previously used",
                ),
            }
        }
        Err(e) => response_500(e),
    }
}

#[cfg(test)]
mod tests {
    #[actix_web::test]
    #[ignore]
    async fn submit_password_change() -> Result<(), String> {
        todo!();
    }
}
