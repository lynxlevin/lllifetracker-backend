use actix_web::{get, web::ReqData, HttpResponse};

use crate::entities::user as user_entity;

#[get("/me")]
pub async fn get_user(user: Option<ReqData<user_entity::ActiveModel>>) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            HttpResponse::Ok().json(crate::types::UserVisible {
                id: user.id.unwrap(),
                email: user.email.unwrap(),
                first_name: user.first_name.unwrap(),
                last_name: user.last_name.unwrap(),
                is_active: user.is_active.unwrap(),
            })
        }
        None => HttpResponse::Ok().json("You are not logged in."),
    }
}
