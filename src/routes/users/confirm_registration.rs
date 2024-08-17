use crate::entities::{user, user::Entity as User};
use actix_web::{
    get,
    http::header,
    web::{Data, Query},
    HttpResponse,
};
use sea_orm::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Parameters {
    token: String,
}

#[tracing::instrument(name = "Activating a new user", skip(data, parameters))]
#[get("register/confirm")]
pub async fn confirm(
    parameters: Query<Parameters>,
    data: Data<crate::startup::AppState>,
) -> HttpResponse {
    let settings = crate::settings::get_settings().expect("Failed to read settings.");

    let redis_con = &mut data
        .redis_pool
        .get()
        .await
        .map_err(|e| {
            tracing::event!(target: "backend", tracing::Level::ERROR, "{}", e);
            HttpResponse::SeeOther()
                .insert_header((
                    header::LOCATION,
                    format!("{}/auth/error", settings.frontend_url),
                ))
                .json(crate::types::ErrorResponse {
                    error: "We cannot activate your account at the moment".to_string(),
                })
        })
        .expect("Redis connection cannot be gotten.");

    let confirmation_token = match crate::utils::auth::tokens::verify_confirmation_token_pasetor(
        parameters.token.clone(),
        redis_con,
        None,
    )
    .await
    {
        Ok(token) => token,
        Err(e) => {
            tracing::event!(target: "backend", tracing::Level::ERROR, "{:#?}", e);

            return HttpResponse::SeeOther()
                .insert_header((
                    header::LOCATION,
                    format!("{}/auth/regenerate-token", settings.frontend_url),
                )).json(crate::types::ErrorResponse {error: "It appears that your confirmation token has expired or previously used. Kindly generate a new token".to_string()});
        }
    };

    let mut user: user::ActiveModel = match User::find_by_id(confirmation_token.user_id)
        .one(&data.conn)
        .await
    {
        Ok(user) => user.unwrap().into(),
        Err(e) => {
            tracing::event!(target: "backend", tracing::Level::ERROR, "Cannot activate account: {}", e);
            return HttpResponse::SeeOther()
                .insert_header((
                    header::LOCATION,
                    format!("{}/auth/error?reason={e}", settings.frontend_url),
                ))
                .json(crate::types::ErrorResponse {
                    error: "We cannot activate your account at the moment".to_string(),
                });
        }
    };
    user.is_active = Set(true);
    match user.update(&data.conn).await {
        Ok(_) => {
            tracing::event!(target: "backend", tracing::Level::INFO, "New user was activated successfully.");
            HttpResponse::SeeOther()
                .insert_header((
                    header::LOCATION,
                    format!("{}/auth/confirmed", settings.frontend_url),
                ))
                .json(crate::types::SuccessResponse {
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
                .json(crate::types::ErrorResponse {
                    error: "We cannot activate your account at the moment".to_string(),
                })
        }
    }
}
