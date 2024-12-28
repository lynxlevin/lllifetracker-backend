use crate::entities::book_excerpt;
use chrono::Utc;
use sea_orm::{prelude::*, DbConn, DbErr, Set};

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
