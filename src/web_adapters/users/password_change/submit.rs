use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use common::settings::types::Settings;
use db_adapters::user_adapter::{UserAdapter, UserMutation, UserQuery};
use deadpool_redis::Pool;
use sea_orm::DbConn;

use crate::utils::{
    auth::{password, tokens::verify_confirmation_token_pasetor},
    response_400, response_404, response_500,
};

#[derive(serde::Deserialize)]
struct Parameters {
    token: String,
    password: String,
}

#[tracing::instrument(name = "Changing user's password", skip(db, redis_pool, req, settings))]
#[post("")]
pub async fn submit_password_change(
    db: Data<DbConn>,
    redis_pool: Data<Pool>,
    req: Json<Parameters>,
    settings: Data<Settings>,
) -> HttpResponse {
    match redis_pool.get().await {
        Ok(ref mut redis_con) => {
            match verify_confirmation_token_pasetor(
                req.token.clone(),
                redis_con,
                Some(true),
                &settings,
            )
            .await
            {
                Ok(confirmation_token) => {
                    let hashed_password = password::hash(req.password.as_bytes()).await;
                    let user = match UserAdapter::init(&db)
                        .get_by_id(confirmation_token.user_id)
                        .await
                    {
                        Ok(user) => match user {
                            Some(user) => user,
                            None => return response_404("User not found"),
                        },
                        Err(e) => return response_500(e),
                    };
                    match UserAdapter::init(&db).update_password(user, hashed_password).await {
                        Ok(_) => {
                            HttpResponse::Ok().json("Your password has been changed successfully. Kindly login with the new password")
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
