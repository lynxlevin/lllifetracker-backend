use crate::services::user;
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
struct RequestBody {
    email: String,
    password: String,
    first_name: String,
    last_name: String,
}
#[tracing::instrument(name = "Adding a new user",
skip(data, new_user),
fields(
    new_user_mail = %new_user.email,
    new_user_first_name = %new_user.first_name,
    new_user_last_name = %new_user.last_name
))]
#[post("/register")]
pub async fn register_user(
    data: Data<crate::startup::AppState>,
    new_user: Json<RequestBody>,
) -> HttpResponse {
    let settings = crate::settings::get_settings().expect("Failed to read settings.");

    let hashed_password = crate::utils::auth::password::hash(new_user.0.password.as_bytes()).await;

    let create_new_user = user::NewUser {
        password: hashed_password,
        email: new_user.0.email,
        first_name: new_user.0.first_name,
        last_name: new_user.0.last_name,
        is_active: settings.email.no_verify,
    };

    let user_id = match user::Mutation::create_user(&data.conn, create_new_user.clone()).await {
        Ok(user) => user.id,
        Err(e) => {
            tracing::event!(target: "backend", tracing::Level::ERROR, "Failed to create user: {:#?}", e);
            return HttpResponse::InternalServerError().json(crate::types::ErrorResponse {
                error: "Some error on user registration.".to_string(),
            });
        }
    };

    let redis_con = &mut data
        .redis_pool
        .get()
        .await
        .map_err(|e| {
            tracing::event!(target: "backend", tracing::Level::ERROR, "{}", e);
            HttpResponse::InternalServerError().json(crate::types::ErrorResponse {
                error: "We cannot activate your account at the moment".to_string(),
            })
        })
        .expect("Redis connection cannot be gotten.");

    let message: String;
    if !settings.email.no_verify {
        crate::utils::emails::send_multipart_email(
            "Lynx Levin's LifeTracker - Let's get you verified".to_string(),
            user_id,
            create_new_user.email,
            create_new_user.first_name,
            create_new_user.last_name,
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

    HttpResponse::Ok().json(crate::types::SuccessResponse { message: message })
}
