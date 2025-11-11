use chrono::{NaiveDate, Utc};
use entities::reading_note;
use sea_orm::Set;
use uuid::Uuid;

pub fn reading_note(user_id: Uuid) -> reading_note::ActiveModel {
    let now = Utc::now();
    reading_note::ActiveModel {
        id: Set(uuid::Uuid::now_v7()),
        title: Set("reading_note".to_string()),
        page_number: Set(1),
        text: Set("book content".to_string()),
        date: Set(now.date_naive()),
        user_id: Set(user_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
}

pub trait ReadingNoteFactory {
    fn title(self, title: String) -> reading_note::ActiveModel;
    fn date(self, date: NaiveDate) -> reading_note::ActiveModel;
}

impl ReadingNoteFactory for reading_note::ActiveModel {
    fn title(mut self, title: String) -> reading_note::ActiveModel {
        self.title = Set(title);
        self
    }

    fn date(mut self, date: NaiveDate) -> reading_note::ActiveModel {
        self.date = Set(date);
        self
    }
}
