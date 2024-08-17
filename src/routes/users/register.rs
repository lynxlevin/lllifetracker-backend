use crate::entities::user;
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use sea_orm::*;

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct NewUser {
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
#[post("/register/")]
pub async fn register_user(
    data: Data<crate::startup::AppState>,
    new_user: Json<NewUser>,
) -> HttpResponse {
    let hashed_password = crate::utils::auth::password::hash(new_user.0.password.as_bytes()).await;

    let create_new_user = NewUser {
        password: hashed_password,
        email: new_user.0.email,
        first_name: new_user.0.first_name,
        last_name: new_user.0.last_name,
    };

    let user_id = user::ActiveModel {
        password: Set(create_new_user.password.clone()),
        email: Set(create_new_user.email.clone()),
        first_name: Set(create_new_user.first_name.clone()),
        last_name: Set(create_new_user.last_name.clone()),
        ..Default::default()
    }
    .save(&data.conn)
    .await
    .unwrap()
    .id
    .unwrap();

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

    crate::utils::emails::send_multipart_email(
        "Lynx Levin's WineCellar - Let's get you verified".to_string(),
        user_id,
        create_new_user.email,
        create_new_user.first_name,
        create_new_user.last_name,
        "verification_email.html",
        redis_con,
    )
    .await
    .unwrap();

    tracing::event!(target: "backend", tracing::Level::INFO, "User created successfully.");

    HttpResponse::Ok().json(crate::types::SuccessResponse {
        message: "Your account was created successfully. Check your email address to activate your account as we just sent you an activation link. Ensure you activate your account before the link expires".to_string(),
    })
}
