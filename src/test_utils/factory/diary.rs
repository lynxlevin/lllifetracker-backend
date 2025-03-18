use chrono::{NaiveDate, Utc};
use entities::diary;
use sea_orm::Set;
use uuid::Uuid;

pub fn diary(user_id: Uuid) -> diary::ActiveModel {
    diary::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        text: Set(Some("diary".to_string())),
        date: Set(Utc::now().date_naive()),
        score: Set(Some(4)),
        user_id: Set(user_id),
    }
}

pub trait DiaryFactory {
    fn text(self, text: Option<String>) -> diary::ActiveModel;
    fn date(self, date: NaiveDate) -> diary::ActiveModel;
}

impl DiaryFactory for diary::ActiveModel {
    fn text(mut self, text: Option<String>) -> diary::ActiveModel {
        self.text = Set(text);
        self
    }

    fn date(mut self, date: NaiveDate) -> diary::ActiveModel {
        self.date = Set(date);
        self
    }
}
