use services::user;
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use deadpool_redis::Pool;
use sea_orm::DbConn;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    email: String,
    password: String,
    first_name: String,
    last_name: String,
}
#[tracing::instrument(name = "Adding a new user",
skip(db, redis_pool, new_user),
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
) -> HttpResponse {
    let settings = crate::settings::get_settings();

    let hashed_password = crate::utils::auth::password::hash(new_user.0.password.as_bytes()).await;

    let new_user = user::NewUser {
        password: hashed_password,
        email: new_user.0.email,
        first_name: new_user.0.first_name,
        last_name: new_user.0.last_name,
        is_active: settings.email.no_verify,
    };

    match user::Mutation::create_user(&db, new_user).await {
        Ok(user) => match redis_pool.get().await {
            Ok(ref mut redis_con) => {
                let message: String;
                if !settings.email.no_verify {
                    crate::utils::emails::send_multipart_email(
                        "Let's get you verified".to_string(),
                        user.id,
                        user.email,
                        user.first_name,
                        user.last_name,
                        "verification_email.html",
                        redis_con,
                    )
                    .await
                    .unwrap();

                    message = "Your account was created successfully. Check your email address to activate your account as we just sent you an activation link. Ensure you activate your account before the link expires".to_string();
                } else {
                    message = "Your account was created successfully.".to_string();
                }

                tracing::event!(target: "backend", tracing::Level::INFO, "User created successfully.");
                HttpResponse::Ok().json(::types::SuccessResponse { message: message })
            }
            Err(e) => {
                tracing::event!(target: "backend", tracing::Level::ERROR, "{}", e);
                HttpResponse::InternalServerError().json(::types::ErrorResponse {
                    error: "We cannot activate your account at the moment".to_string(),
                })
            }
        },
        Err(e) => {
            tracing::event!(target: "backend", tracing::Level::ERROR, "Failed to create user: {:#?}", e);
            HttpResponse::InternalServerError().json(::types::ErrorResponse {
                error: "Some error on user registration.".to_string(),
            })
        }
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
