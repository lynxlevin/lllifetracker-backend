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
        score: Set(Some(8)),
        user_id: Set(user_id),
    }
}

pub trait DiaryFactory {
    fn positive_text(self, text: Option<String>) -> diary::ActiveModel;
    fn date(self, date: NaiveDate) -> diary::ActiveModel;
}

impl DiaryFactory for diary::ActiveModel {
    fn positive_text(mut self, text: Option<String>) -> diary::ActiveModel {
        self.positive_text = Set(text);
        self
    }

    fn date(mut self, date: NaiveDate) -> diary::ActiveModel {
        self.date = Set(date);
        self
    }
}
