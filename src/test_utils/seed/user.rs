use crate::entities::user;
use chrono::Utc;
use sea_orm::{prelude::*, DbConn, DbErr, Set};

#[cfg(test)]
pub async fn create_user(db: &DbConn, is_active: bool) -> Result<user::Model, DbErr> {
    use crate::entities::sea_orm_active_enums::TimezoneEnum;

    let now = Utc::now();
    user::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        email: Set(format!("{}@test.com", uuid::Uuid::new_v4().to_string())),
        password: Set("password".to_string()),
        first_name: Set("Lynx".to_string()),
        last_name: Set("Levin".to_string()),
        timezone: Set(TimezoneEnum::AsiaTokyo),
        is_active: Set(is_active),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
    .insert(db)
    .await
}

#[cfg(test)]
pub async fn create_active_user(db: &DbConn) -> Result<user::Model, DbErr> {
    create_user(db, true).await
}

#[cfg(test)]
pub async fn create_inactive_user(db: &DbConn) -> Result<user::Model, DbErr> {
    create_user(db, false).await
}