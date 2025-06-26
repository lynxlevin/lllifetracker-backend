use actix_session::Session;

use crate::users::types::USER_ID_KEY;

// MYMEMO: use log
pub async fn get_user_id(session: &Session) -> Result<uuid::Uuid, String> {
    match session.get(USER_ID_KEY) {
        Ok(user_id) => match user_id {
            None => Err("You are not authenticated".to_string()),
            Some(id) => Ok(id),
        },
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    #[actix_web::test]
    #[ignore]
    async fn get_user_id() -> Result<(), String> {
        todo!();
    }
}
