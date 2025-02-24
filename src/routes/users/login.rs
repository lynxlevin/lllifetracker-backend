use crate::utils::auth::password::verify_password;
use ::types::{INTERNAL_SERVER_ERROR_MESSAGE, USER_EMAIL_KEY, USER_ID_KEY};
use actix_session::SessionInsertError;
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use deadpool_redis::{
    redis::{AsyncCommands, SetExpiry, SetOptions},
    Connection, Pool,
};
use sea_orm::DbConn;
use services::user::Query as UserQuery;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct LoginUser {
    email: String,
    password: String,
}

#[tracing::instrument(name = "Logging a user in", skip(db, redis_pool, req_user, session), fields(user_email = &req_user.email))]
#[post("/login")]
async fn login_user(
    db: Data<DbConn>,
    redis_pool: Data<Pool>,
    req_user: Json<LoginUser>,
    session: actix_session::Session,
) -> HttpResponse {
    let not_found_message = "A user with these details does not exist. If you registered with these details, ensure you activate your account by clicking on the link sent to your e-mail address.";
    match redis_pool.get().await {
        Ok(ref mut redis_con) => match validate_request_count(redis_con, &req_user.email).await {
            Ok((login_request_count_key, login_request_count)) => {
                match UserQuery::find_active_by_email(&db, req_user.email.clone()).await {
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
                                        Ok(_) => HttpResponse::Ok().json(::types::UserVisible {
                                            id: user.id,
                                            email: user.email,
                                            first_name: user.first_name,
                                            last_name: user.last_name,
                                            is_active: user.is_active,
                                        }),
                                        Err(e) => {
                                            tracing::event!(target: "redis", tracing::Level::WARN, "Failed to renew session: {:#?}", e);
                                            HttpResponse::InternalServerError().json(
                                                ::types::ErrorResponse {
                                                    error: INTERNAL_SERVER_ERROR_MESSAGE
                                                        .to_string(),
                                                },
                                            )
                                        }
                                    }
                                }
                                Err(_) => {
                                    increment_login_request_count(
                                        redis_con,
                                        login_request_count_key,
                                        login_request_count,
                                    )
                                    .await;
                                    HttpResponse::NotFound().json(::types::ErrorResponse {
                                        error: not_found_message.to_string(),
                                    })
                                }
                            }
                        }
                        None => {
                            increment_login_request_count(
                                redis_con,
                                login_request_count_key,
                                login_request_count,
                            )
                            .await;
                            HttpResponse::NotFound().json(::types::ErrorResponse {
                                error: not_found_message.to_string(),
                            })
                        }
                    },
                    Err(e) => {
                        tracing::event!(target: "sea-orm", tracing::Level::ERROR, "Some DB error on retrieving a user:{:#?}", e);
                        HttpResponse::InternalServerError().json(::types::ErrorResponse {
                            error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
                        })
                    }
                }
            }
            Err(_) => HttpResponse::Unauthorized().json(::types::ErrorResponse {
                error: "Your account is temporarily locked. Please wait for 1 hour.".to_string(),
            }),
        },
        Err(e) => {
            tracing::event!(target: "backend", tracing::Level::ERROR, "{}", e);
            HttpResponse::InternalServerError().json(::types::ErrorResponse {
                error: INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
            })
        }
    }
}

async fn validate_request_count(
    redis_con: &mut Connection,
    email: &str,
) -> Result<(String, i32), String> {
    let max_login_request_count = 5;
    let login_request_count_key = format!("login_count_{}", email);
    let login_request_count = redis_con.get(login_request_count_key.clone()).await.map_err(|e| {
        tracing::event!(target: "backend", tracing::Level::WARN, "Error getting login_request_count, defaults to 0: {}", e);
    }).unwrap_or(0);
    if login_request_count >= max_login_request_count {
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
    // MYMEMO: maybe ? does the same as this if let Err(e)
    if let Err(e) = session.insert(USER_ID_KEY, id) {
        return Err(e);
    }
    session.insert(USER_EMAIL_KEY, email)
}

async fn increment_login_request_count(
    redis_con: &mut Connection,
    login_request_count_key: String,
    login_request_count: i32,
) -> () {
    if let Err(e) = redis_con
        .set_options::<String, i32, String>(
            login_request_count_key,
            login_request_count + 1,
            SetOptions::default().with_expiration(SetExpiry::EX(3600)),
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
