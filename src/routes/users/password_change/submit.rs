use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};

use crate::{
    services::user as user_service,
    startup::AppState,
    types,
    utils::auth::{password, tokens::verify_confirmation_token_pasetor},
};

#[derive(serde::Deserialize)]
struct Parameters {
    token: String,
    password: String,
}

#[tracing::instrument(name = "Changing user's password", skip(data, req))]
#[post("")]
pub async fn submit_password_change(data: Data<AppState>, req: Json<Parameters>) -> HttpResponse {
    match data.redis_pool.get().await {
        Ok(ref mut redis_con) => {
            match verify_confirmation_token_pasetor(req.token.clone(), redis_con, Some(true)).await
            {
                Ok(confirmation_token) => {
                    let hashed_password = password::hash(req.password.as_bytes()).await;
                    match user_service::Mutation::update_user_password(&data.conn, confirmation_token.user_id, hashed_password).await {
                        Ok(_) => {
                            HttpResponse::Ok().json(types::SuccessResponse {
                                message: "Your password has been changed successfully. Kindly login with the new password"
                                    .to_string(),
                            })
                        }
                        Err(e) => {
                            tracing::event!(target: "backend", tracing::Level::ERROR, "Failed to change user password: {:#?}", e);
                            HttpResponse::InternalServerError().json(types::ErrorResponse {
                                error: "Something unexpected happened. Kindly try again".to_string(),
                            })
                        }
                    }
                }
                Err(e) => {
                    tracing::event!(target: "backend", tracing::Level::ERROR, "{e}");
                    HttpResponse::BadRequest().json(types::ErrorResponse {error: "It appears that your password request token has expired or previously used".to_string()})
                }
            }
        }
        Err(e) => {
            tracing::event!(target: "backend", tracing::Level::ERROR, "{e}");
            HttpResponse::InternalServerError().json(types::ErrorResponse {
                error: "Something unexpected happened. Kindly try again".to_string(),
            })
        }
    }
}
