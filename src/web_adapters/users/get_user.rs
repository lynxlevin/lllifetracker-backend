use actix_web::{get, web::ReqData, HttpResponse};

use entities::user as user_entity;

use crate::{users::types::UserVisible, utils::response_401};

#[get("/me")]
pub async fn get_user(user: Option<ReqData<user_entity::Model>>) -> HttpResponse {
    match user {
        Some(user) => {
            let user = user.into_inner();
            HttpResponse::Ok().json(UserVisible {
                id: user.id,
                email: user.email,
                first_name: user.first_name,
                last_name: user.last_name,
                is_active: user.is_active,
            })
        }
        None => response_401(),
    }
}

#[cfg(test)]
mod tests {
    #[actix_web::test]
    #[ignore]
    async fn get_user() -> Result<(), String> {
        todo!();
    }
}
