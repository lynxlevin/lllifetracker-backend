use chrono::{NaiveDate, Utc};
use entities::memo;
use sea_orm::Set;
use uuid::Uuid;

pub fn memo(user_id: Uuid) -> memo::ActiveModel {
    let now = Utc::now();
    memo::ActiveModel {
        id: Set(uuid::Uuid::now_v7()),
        title: Set("memo".to_string()),
        text: Set("text".to_string()),
        date: Set(now.date_naive()),
        favorite: Set(false),
        archived: Set(false),
        user_id: Set(user_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
}

pub trait MemoFactory {
    fn title(self, title: String) -> memo::ActiveModel;
    fn date(self, date: NaiveDate) -> memo::ActiveModel;
    fn favorite(self, favorite: bool) -> memo::ActiveModel;
}

impl MemoFactory for memo::ActiveModel {
    fn title(mut self, title: String) -> memo::ActiveModel {
        self.title = Set(title);
        self
    }

    fn date(mut self, date: NaiveDate) -> memo::ActiveModel {
        self.date = Set(date);
        self
    }

    fn favorite(mut self, favorite: bool) -> memo::ActiveModel {
        self.favorite = Set(favorite);
        self
    }
}
