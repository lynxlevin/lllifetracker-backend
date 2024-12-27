use crate::entities::{book_excerpt, memo, mission_memo};
use chrono::Utc;
use sea_orm::{prelude::*, DbConn, DbErr, Set};

#[cfg(test)]
pub async fn create_memo(
    db: &DbConn,
    title: String,
    user_id: uuid::Uuid,
) -> Result<memo::Model, DbErr> {
    let now = Utc::now();
    memo::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        title: Set(title),
        text: Set("text".to_string()),
        date: Set(now.date_naive()),
        archived: Set(false),
        user_id: Set(user_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
    .insert(db)
    .await
}

#[cfg(test)]
pub async fn create_mission_memo(
    db: &DbConn,
    title: String,
    user_id: uuid::Uuid,
) -> Result<mission_memo::Model, DbErr> {
    let now = Utc::now();
    mission_memo::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        title: Set(title),
        text: Set("text".to_string()),
        date: Set(now.date_naive()),
        archived: Set(false),
        accomplished_at: Set(None),
        user_id: Set(user_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
    .insert(db)
    .await
}

#[cfg(test)]
pub async fn create_book_excerpt(
    db: &DbConn,
    title: String,
    user_id: uuid::Uuid,
) -> Result<book_excerpt::Model, DbErr> {
    let now = Utc::now();
    book_excerpt::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        title: Set(title),
        page_number: Set(1),
        text: Set("text".to_string()),
        date: Set(now.date_naive()),
        user_id: Set(user_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
    .insert(db)
    .await
}
