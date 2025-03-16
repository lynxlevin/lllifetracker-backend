use chrono::{NaiveDate, Utc};
use entities::diary;
use sea_orm::Set;
use uuid::Uuid;

pub fn diary(user_id: Uuid) -> diary::ActiveModel {
    diary::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        positive_text: Set(Some("diary".to_string())),
        negative_text: Set(Some("text".to_string())),
        date: Set(Utc::now().date_naive()),
        score: Set(8),
        user_id: Set(user_id),
    }
}

pub trait DiaryFactory {
    fn date(self, date: NaiveDate) -> diary::ActiveModel;
}

impl DiaryFactory for diary::ActiveModel {
    fn date(mut self, date: NaiveDate) -> diary::ActiveModel {
        self.date = Set(date);
        self
    }
}
