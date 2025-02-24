use entities::book_excerpt;
use chrono::Utc;
use sea_orm::Set;
use uuid::Uuid;

pub fn book_excerpt(user_id: Uuid) -> book_excerpt::ActiveModel {
    let now = Utc::now();
    book_excerpt::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        title: Set("book_excerpt".to_string()),
        page_number: Set(1),
        text: Set("book content".to_string()),
        date: Set(now.date_naive()),
        user_id: Set(user_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
}

pub trait BookExcerptFactory {
    fn title(self, title: String) -> book_excerpt::ActiveModel;
}

impl BookExcerptFactory for book_excerpt::ActiveModel {
    fn title(mut self, title: String) -> book_excerpt::ActiveModel {
        self.title = Set(title);
        self
    }
}
