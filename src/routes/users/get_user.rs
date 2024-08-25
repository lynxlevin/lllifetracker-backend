use actix_web::{get, web::Data, HttpResponse};

use crate::{entities::user, services::user as user_service, startup::AppState};

#[get("/me")]
pub async fn get_user(data: Data<AppState>, session: actix_session::Session) -> HttpResponse {
    let user_id =
        match session_user_id(&session).await {
            Ok(user_id) => user_id,
            Err(e) => return HttpResponse::BadRequest().json(crate::types::ErrorResponse {
                error:
                    "We currently have some issues. Kindly try again and ensure you are logged in."
                        .to_string(),
            }),
        };
    match user_service::Query::find_by_id(&data.conn, user_id).await {
        Ok(user) => {
            let _user: user::ActiveModel = user.unwrap().into();
            HttpResponse::Ok().json(crate::types::UserVisible {
                id: _user.id.unwrap(),
                email: _user.email.unwrap(),
                first_name: _user.first_name.unwrap(),
                last_name: _user.last_name.unwrap(),
                is_active: _user.is_active.unwrap(),
            })
        }
        Err(e) => HttpResponse::NotFound().json(crate::types::ErrorResponse {
            error: "We could not find the user.".to_string(),
        }),
    }
}

async fn session_user_id(session: &actix_session::Session) -> Result<uuid::Uuid, String> {
    match session.get(crate::types::USER_ID_KEY) {
        Ok(user_id) => match user_id {
            None => Err("You are not authenticated".to_string()),
            Some(id) => Ok(id),
        },
        Err(e) => Err(format!("{e}")),
    }
}
