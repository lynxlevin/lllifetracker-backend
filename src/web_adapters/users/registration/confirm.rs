use actix_web::{
    get,
    http::header,
    web::{Data, Query},
    HttpResponse,
};
use common::settings::types::Settings;
use db_adapters::user_adapter::{UserAdapter, UserMutation, UserQuery};
use deadpool_redis::Pool;
use sea_orm::DbConn;

use crate::{users::utils::auth::tokens::verify_confirmation_token_pasetor, utils::ErrorResponse};

#[derive(serde::Deserialize)]
pub struct Parameters {
    token: String,
}

#[tracing::instrument(
    name = "Activating a new user",
    skip(db, redis_pool, parameters, settings)
)]
#[get("/confirm")]
pub async fn confirm(
    parameters: Query<Parameters>,
    db: Data<DbConn>,
    redis_pool: Data<Pool>,
    settings: Data<Settings>,
) -> HttpResponse {
    match redis_pool.get().await {
        Ok(ref mut redis_con) => {
            match verify_confirmation_token_pasetor(
                parameters.token.clone(),
                redis_con,
                None,
                &settings,
            )
            .await
            {
                Ok(confirmation_token) => {
                    match UserAdapter::init(&db)
                        .get_by_id(confirmation_token.user_id)
                        .await
                    {
                        Ok(user) => match user {
                            Some(user) => match UserAdapter::init(&db).activate(user).await {
                                Ok(_) => {
                                    tracing::event!(target: "backend", tracing::Level::INFO, "New user was activated successfully.");
                                    HttpResponse::SeeOther()
                                            .insert_header((
                                                header::LOCATION,
                                                format!("{}/auth/confirmed", settings.application.frontend_url),
                                            ))
                                            .json("Your account has been activated successfully! You can now log in.")
                                }
                                Err(e) => {
                                    tracing::event!(target: "backend", tracing::Level::ERROR, "Cannot activate account: {}", e);
                                    HttpResponse::SeeOther()
                                        .insert_header((
                                            header::LOCATION,
                                            format!(
                                                "{}/auth/error?reason=internal_server_error",
                                                settings.application.frontend_url
                                            ),
                                        ))
                                        .json(ErrorResponse {
                                            error: "We cannot activate your account at the moment"
                                                .to_string(),
                                        })
                                }
                            },
                            None => HttpResponse::SeeOther()
                                .insert_header((
                                    header::LOCATION,
                                    format!(
                                        "{}/auth/error?reason=user_not_found",
                                        settings.application.frontend_url
                                    ),
                                ))
                                .json(ErrorResponse {
                                    error: "We cannot activate your account at the moment"
                                        .to_string(),
                                }),
                        },
                        Err(e) => {
                            tracing::event!(target: "backend", tracing::Level::ERROR, "Cannot activate account: {}", e);
                            HttpResponse::SeeOther()
                                .insert_header((
                                    header::LOCATION,
                                    format!(
                                        "{}/auth/error?reason=internal_server_error",
                                        settings.application.frontend_url
                                    ),
                                ))
                                .json(ErrorResponse {
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
                            format!("{}/auth/regenerate-token", settings.application.frontend_url),
                        )).json(ErrorResponse {error: "It appears that your confirmation token has expired or previously used. Kindly generate a new token".to_string()})
                }
            }
        }
        Err(e) => {
            tracing::event!(target: "backend", tracing::Level::ERROR, "{}", e);
            HttpResponse::SeeOther()
                .insert_header((
                    header::LOCATION,
                    format!("{}/auth/error", settings.application.frontend_url),
                ))
                .json(ErrorResponse {
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
