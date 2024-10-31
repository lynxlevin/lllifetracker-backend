use actix_web::{
    get,
    http::header,
    web::{Data, Query},
    HttpResponse,
};
use deadpool_redis::Pool;

use crate::{
    settings,
    types::INTERNAL_SERVER_ERROR_MESSAGE,
    utils::auth::tokens::{issue_confirmation_token_pasetors, verify_confirmation_token_pasetor},
};

#[derive(serde::Deserialize)]
struct Parameters {
    token: String,
}

#[tracing::instrument(name = "Confirming change password token", skip(query, redis_pool))]
#[get("/email-verification")]
pub async fn verify_password_change_token(
    query: Query<Parameters>,
    redis_pool: Data<Pool>,
) -> HttpResponse {
    let frontend_url = settings::get_settings().frontend_url;
    match redis_pool.get().await {
        Ok(ref mut redis_con) => {
            match verify_confirmation_token_pasetor(query.token.clone(), redis_con, None).await {
                Ok(confirmation_token) => {
                    match issue_confirmation_token_pasetors(
                        confirmation_token.user_id,
                        redis_con,
                        Some(true),
                    )
                    .await
                    {
                        Ok(issued_token) => {
                            HttpResponse::SeeOther()
                            .insert_header((
                                header::LOCATION,
                                format!(
                                    // MYMEMO: Change url later.
                                    "{frontend_url}/auth/password/change-password?token={issued_token}"
                                ),
                            ))
                            .finish()
                        }
                        Err(e) => {
                            tracing::event!(target: "backend", tracing::Level::ERROR, "{e}");
                            // MYMEMO: Change url later.
                            HttpResponse::SeeOther()
                                .insert_header((
                                    header::LOCATION,
                                    format!("{frontend_url}/auth/error?reason={e}"),
                                ))
                                .finish()
                        }
                    }
                }
                Err(e) => {
                    tracing::event!(target: "backend", tracing::Level::ERROR, "{e}");
                    // MYMEMO: Change url later.
                    HttpResponse::SeeOther().insert_header((header::LOCATION, format!("{frontend_url}/auth/error?reason=It appears that your password request token has expired or previously used"))).finish()
                }
            }
        }
        Err(e) => {
            tracing::event!(target: "backend", tracing::Level::ERROR, "{e}");
            // MYMEMO: Change url later.
            HttpResponse::SeeOther()
                .insert_header((
                    header::LOCATION,
                    format!("{frontend_url}/auth/error?reason={INTERNAL_SERVER_ERROR_MESSAGE}"),
                ))
                .finish()
        }
    }
}

#[cfg(test)]
mod tests {
    #[actix_web::test]
    #[ignore]
    async fn verify_password_change_token() -> Result<(), String> {
        todo!();
    }
}
