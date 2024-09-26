use crate::{
    entities::user,
    services::user as user_service,
    startup::AppState,
    types::{USER_EMAIL_KEY, USER_ID_KEY},
    utils::auth::password::verify_password,
};
use actix_web::{
    post,
    rt::task,
    web::{Data, Json},
    HttpResponse,
};
use deadpool_redis::redis::{AsyncCommands, SetExpiry, SetOptions};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct LoginUser {
    email: String,
    password: String,
}

#[tracing::instrument(name = "Logging a user in", skip(data, req_user, session), fields(user_email = &req_user.email))]
#[post("/login")]
async fn login_user(
    data: Data<AppState>,
    req_user: Json<LoginUser>,
    session: actix_session::Session,
) -> HttpResponse {
    let not_found_message = "A user with these details does not exist. If you registered with these details, ensure you activate your account by clicking on the link sent to your e-mail address.";

    let max_login_request_count = 5;
    let redis_connection = &mut data
        .redis_pool
        .get()
        .await
        .map_err(|e| {
            tracing::event!(target: "backend", tracing::Level::ERROR, "{}", e);
            return HttpResponse::InternalServerError().json(crate::types::ErrorResponse {
                error: "Some unknown error happened. Please try again later.".to_string(),
            });
        })
        .expect("Redis connection cannot be gotten.");
    let redis_key = format!("login_count_{}", req_user.email.clone()).to_string();
    let login_request_count = match redis_connection
        .get(redis_key.clone())
        .await
        .map_err(|e| format!("{}", e))
    {
        Ok(count) => count,
        Err(_) => 0,
    };

    if login_request_count >= max_login_request_count {
        return HttpResponse::InternalServerError().json(crate::types::ErrorResponse {
            error: "Your account is temporarily locked. Please wait for 1 hour.".to_string(),
        });
    };

    let user: user::ActiveModel = match user_service::Query::find_active_by_email(
        &data.conn,
        req_user.email.clone(),
    )
    .await
    {
        Ok(user) => user.unwrap().into(),
        Err(e) => {
            tracing::event!(target: "sea-orm", tracing::Level::ERROR, "User not found:{:#?}", e);
            return HttpResponse::NotFound().json(crate::types::ErrorResponse {
                error: not_found_message.to_string(),
            });
        }
    };
    match task::spawn_blocking(move || {
        verify_password(
            user.password.unwrap().as_ref(),
            req_user.password.clone().as_bytes(),
        )
    })
    .await
    .expect("Unable to unwrap JoinError.")
    {
        Ok(_) => {
            tracing::event!(target: "backend", tracing::Level::INFO, "User logged in successfully.");
            let _ = redis_connection
                .del::<String, String>(redis_key)
                .await
                .map_err(|e| format!("{}", e));
            session.renew();
            session
                .insert(USER_ID_KEY, user.id.clone().unwrap())
                .expect(format!("`{}` cannot be inserted into session", USER_ID_KEY).as_str());
            session
                .insert(USER_EMAIL_KEY, &user.email.clone().unwrap())
                .expect(format!("`{}` cannot be inserted into session", USER_EMAIL_KEY).as_str());
            HttpResponse::Ok().json(crate::types::UserVisible {
                id: user.id.unwrap(),
                email: user.email.unwrap(),
                first_name: user.first_name.unwrap(),
                last_name: user.last_name.unwrap(),
                is_active: user.is_active.unwrap(),
            })
        }
        Err(e) => {
            tracing::event!(target: "argon2", tracing::Level::ERROR, "Failed to authenticate user: {:#?}", e);
            let opts = SetOptions::default().with_expiration(SetExpiry::EX(3600));
            let _ = redis_connection
                .set_options::<String, i32, String>(redis_key, login_request_count + 1, opts)
                .await
                .map_err(|e| format!("{}", e));
            HttpResponse::NotFound().json(crate::types::ErrorResponse {
                error: not_found_message.to_string(),
            })
        }
    }
}
