use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use common::settings::types::Settings;
use db_adapters::user_adapter::{CreateUserParams, UserAdapter, UserMutation};
use deadpool_redis::Pool;
use sea_orm::DbConn;

use crate::utils::response_500;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    email: String,
    password: String,
    first_name: String,
    last_name: String,
}
#[tracing::instrument(name = "Adding a new user",
skip(db, redis_pool, new_user, settings),
fields(
    new_user_mail = %new_user.email,
    new_user_first_name = %new_user.first_name,
    new_user_last_name = %new_user.last_name
))]
#[post("")]
pub async fn register(
    db: Data<DbConn>,
    redis_pool: Data<Pool>,
    new_user: Json<RequestBody>,
    settings: Data<Settings>,
) -> HttpResponse {
    let hashed_password = utils::auth::password::hash(new_user.0.password.as_bytes()).await;

    match UserAdapter::init(&db)
        .create(CreateUserParams {
            email: new_user.0.email,
            password: hashed_password,
            first_name: new_user.0.first_name,
            last_name: new_user.0.last_name,
            is_active: settings.email.no_verify,
        })
        .await
    {
        Ok(user) => match redis_pool.get().await {
            Ok(ref mut redis_con) => {
                let message: String;
                if !settings.email.no_verify {
                    utils::emails::send_multipart_email(
                        "Let's get you verified".to_string(),
                        user.id,
                        user.email,
                        user.first_name,
                        user.last_name,
                        "verification_email.html",
                        redis_con,
                        &settings,
                    )
                    .await
                    .unwrap();

                    message = "Your account was created successfully. Check your email address to activate your account as we just sent you an activation link. Ensure you activate your account before the link expires".to_string();
                } else {
                    message = "Your account was created successfully.".to_string();
                }

                tracing::event!(target: "backend", tracing::Level::INFO, "User created successfully.");
                HttpResponse::Ok().json(::types::SuccessResponse { message })
            }
            Err(e) => response_500(e),
        },
        Err(e) => response_500(e),
    }
}

#[cfg(test)]
mod tests {
    #[actix_web::test]
    #[ignore]
    async fn register() -> Result<(), String> {
        todo!();
    }
}
