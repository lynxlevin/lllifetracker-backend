use crate::entities::memo;
use chrono::Utc;
use sea_orm::Set;
use uuid::Uuid;

#[cfg(test)]
pub fn memo(user_id: Uuid) -> memo::ActiveModel {
    let now = Utc::now();
    memo::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        title: Set("memo".to_string()),
        text: Set("text".to_string()),
        date: Set(now.date_naive()),
        archived: Set(false),
        user_id: Set(user_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
}

#[cfg(test)]
pub trait MemoFactory {
    fn title(self, title: String) -> memo::ActiveModel;
}

#[cfg(test)]
impl MemoFactory for memo::ActiveModel {
    fn title(mut self, title: String) -> memo::ActiveModel {
        self.title = Set(title);
        self
    }
}
