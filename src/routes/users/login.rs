use actix_web::{
    post,
    rt::task,
    web::{Data, Json},
    HttpResponse,
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::{
    entities::user::{self, Entity as User},
    startup::AppState,
    types::{USER_EMAIL_KEY, USER_ID_KEY},
    utils::auth::password::verify_password,
};

#[derive(serde::Deserialize, Debug, serde::Serialize)]
pub struct LoginUser {
    email: String,
    password: String,
}

// MYMEMO: add countermeasures for Brute force attack.
#[tracing::instrument(name = "Logging a user in", skip(data, req_user, session), fields(user_email = &req_user.email))]
#[post("/login")]
async fn login_user(
    data: Data<AppState>,
    req_user: Json<LoginUser>,
    session: actix_session::Session,
) -> HttpResponse {
    let user: user::ActiveModel = match User::find()
        .filter(user::Column::Email.eq(req_user.email.clone()))
        .filter(user::Column::IsActive.eq(true))
        .one(&data.conn)
        .await
    {
        Ok(user) => user.unwrap().into(),
        Err(e) => {
            tracing::event!(target: "sea-orm", tracing::Level::ERROR, "User not found:{:#?}", e);
            return HttpResponse::NotFound().json(crate::types::ErrorResponse {error: "A user with these details does not exist. If you registered with these details, ensure you activate your account by clicking on the link sent to your e-mail address".to_string()});
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
            HttpResponse::NotFound().json(crate::types::ErrorResponse {error: "A user with these details does not exist. If you registered with these details, ensure you activate your account by clicking on the link sent to your e-mail address".to_string()})
        }
    }
}
