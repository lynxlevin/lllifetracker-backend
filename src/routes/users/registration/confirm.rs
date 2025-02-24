use crate::services::user::Mutation as UserMutation;
use actix_web::{
    get,
    http::header,
    web::{Data, Query},
    HttpResponse,
};
use deadpool_redis::Pool;
use sea_orm::DbConn;

#[derive(serde::Deserialize)]
pub struct Parameters {
    token: String,
}

#[tracing::instrument(name = "Activating a new user", skip(db, redis_pool, parameters))]
#[get("/confirm")]
pub async fn confirm(
    parameters: Query<Parameters>,
    db: Data<DbConn>,
    redis_pool: Data<Pool>,
) -> HttpResponse {
    let settings = crate::settings::get_settings();

    match redis_pool.get().await {
        Ok(ref mut redis_con) => {
            match crate::utils::auth::tokens::verify_confirmation_token_pasetor(
                parameters.token.clone(),
                redis_con,
                None,
            )
            .await
            {
                Ok(confirmation_token) => {
                    match UserMutation::activate_user_by_id(&db, confirmation_token.user_id).await {
                        Ok(_) => {
                            tracing::event!(target: "backend", tracing::Level::INFO, "New user was activated successfully.");
                            HttpResponse::SeeOther()
                                .insert_header((
                                    header::LOCATION,
                                    format!("{}/auth/confirmed", settings.frontend_url),
                                ))
                                .json(::types::SuccessResponse {
                                    message: "Your account has been activated successfully! You can now log in."
                                        .to_string(),
                                })
                        }
                        Err(e) => {
                            tracing::event!(target: "backend", tracing::Level::ERROR, "Cannot activate account: {}", e);
                            HttpResponse::SeeOther()
                                .insert_header((
                                    header::LOCATION,
                                    format!("{}/auth/error?reason={e}", settings.frontend_url),
                                ))
                                .json(::types::ErrorResponse {
                                    error: "We cannot activate your account at the moment"
                                        .to_string(),
                                })
                        }
                    }
                }
                Err(e) => {
                    tracing::event!(target: "backend", tracing::Level::ERROR, "{:#?}", e);
                    HttpResponse::SeeOther()
                        .insert_header((
                            header::LOCATION,
                            format!("{}/auth/regenerate-token", settings.frontend_url),
                        )).json(::types::ErrorResponse {error: "It appears that your confirmation token has expired or previously used. Kindly generate a new token".to_string()})
                }
            }
        }
        Err(e) => {
            tracing::event!(target: "backend", tracing::Level::ERROR, "{}", e);
            HttpResponse::SeeOther()
                .insert_header((
                    header::LOCATION,
                    format!("{}/auth/error", settings.frontend_url),
                ))
                .json(::types::ErrorResponse {
                    error: "We cannot activate your account at the moment".to_string(),
                })
        }
    }
}

#[cfg(test)]
mod tests {
    #[actix_web::test]
    #[ignore]
    async fn confirm() -> Result<(), String> {
        todo!();
    }
}
