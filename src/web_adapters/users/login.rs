use actix_session::SessionInsertError;
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use common::settings::types::Settings;
use db_adapters::user_adapter::{UserAdapter, UserFilter, UserQuery};
use deadpool_redis::{
    redis::{AsyncCommands, SetExpiry, SetOptions},
    Connection, Pool,
};
use sea_orm::DbConn;

use crate::{
    users::types::{UserVisible, USER_EMAIL_KEY, USER_ID_KEY},
    utils::{auth::password::verify_password, response_404, response_500},
};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct LoginUser {
    email: String,
    password: String,
}

#[tracing::instrument(name = "Logging a user in", skip(db, redis_pool, req_user, session, settings), fields(user_email = &req_user.email))]
#[post("/login")]
async fn login_user(
    db: Data<DbConn>,
    redis_pool: Data<Pool>,
    req_user: Json<LoginUser>,
    session: actix_session::Session,
    settings: Data<Settings>,
) -> HttpResponse {
    let not_found_message = "A user with these details does not exist. If you registered with these details, ensure you activate your account by clicking on the link sent to your e-mail address.";
    match redis_pool.get().await {
        Ok(ref mut redis_con) => {
            match validate_request_count(redis_con, &req_user.email, &settings).await {
                Ok((login_request_count_key, login_request_count)) => {
                    match UserAdapter::init(&db)
                        .filter_eq_is_active(true)
                        .get_by_email(req_user.email.clone())
                        .await
                    {
                        Ok(user) => match user {
                            Some(user) => {
                                match verify_password(
                                    &user.password,
                                    req_user.password.clone().as_bytes(),
                                ) {
                                    Ok(_) => {
                                        tracing::event!(target: "backend", tracing::Level::INFO, "User logged in successfully.");
                                        if let Err(e) = redis_con
                                            .del::<String, String>(login_request_count_key)
                                            .await
                                        {
                                            tracing::event!(target: "redis", tracing::Level::WARN, "Error deleting login_request_count_key from Redis: {:#?}", e)
                                        };
                                        match renew_session(session, user.id, user.email.clone()) {
                                            Ok(_) => HttpResponse::Ok().json(UserVisible {
                                                id: user.id,
                                                email: user.email,
                                                first_name: user.first_name,
                                                last_name: user.last_name,
                                                is_active: user.is_active,
                                            }),
                                            Err(e) => response_500(e),
                                        }
                                    }
                                    Err(_) => {
                                        increment_login_request_count(
                                            redis_con,
                                            login_request_count_key,
                                            login_request_count,
                                            &settings,
                                        )
                                        .await;
                                        response_404(not_found_message)
                                    }
                                }
                            }
                            None => {
                                increment_login_request_count(
                                    redis_con,
                                    login_request_count_key,
                                    login_request_count,
                                    &settings,
                                )
                                .await;
                                response_404(not_found_message)
                            }
                        },
                        Err(e) => response_500(e),
                    }
                }
                Err(_) => HttpResponse::Unauthorized()
                    .json("Your account is temporarily locked. Please wait for 1 hour."),
            }
        }
        Err(e) => response_500(e),
    }
}

async fn validate_request_count(
    redis_con: &mut Connection,
    email: &str,
    settings: &Settings,
) -> Result<(String, u64), String> {
    let login_request_count_key = format!("login_count_{}", email);
    let login_request_count = redis_con.get(login_request_count_key.clone()).await.map_err(|e| {
        tracing::event!(target: "backend", tracing::Level::WARN, "Error getting login_request_count, defaults to 0: {}", e);
    }).unwrap_or(0);
    if login_request_count >= settings.application.max_login_attempts {
        Err("Too many login requests".to_string())
    } else {
        Ok((login_request_count_key, login_request_count))
    }
}

fn renew_session(
    session: actix_session::Session,
    id: uuid::Uuid,
    email: String,
) -> Result<(), SessionInsertError> {
    session.renew();
    session.insert(USER_ID_KEY, id)?;
    session.insert(USER_EMAIL_KEY, email)?;
    Ok(())
}

async fn increment_login_request_count(
    redis_con: &mut Connection,
    login_request_count_key: String,
    login_request_count: u64,
    settings: &Settings,
) -> () {
    if let Err(e) = redis_con
        .set_options::<String, u64, String>(
            login_request_count_key,
            login_request_count + 1,
            SetOptions::default().with_expiration(SetExpiry::EX(
                settings.application.login_attempts_cool_time_seconds,
            )),
        )
        .await
    {
        tracing::event!(target: "redis", tracing::Level::WARN, "Error adding login_request_count_key to Redis: {:#?}", e)
    };
}

#[cfg(test)]
mod tests {
    #[actix_web::test]
    #[ignore]
    async fn login_user() -> Result<(), String> {
        todo!();
    }
    #[actix_web::test]
    #[ignore]
    async fn validate_request_count() -> Result<(), String> {
        todo!();
    }
    #[actix_web::test]
    #[ignore]
    async fn renew_session() -> Result<(), String> {
        todo!();
    }
    #[actix_web::test]
    #[ignore]
    async fn increment_login_request_count() -> Result<(), String> {
        todo!();
    }
}
