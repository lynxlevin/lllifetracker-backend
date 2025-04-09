use entities::challenge;
use chrono::Utc;
use sea_orm::{prelude::DateTimeWithTimeZone, Set};
use uuid::Uuid;

pub fn challenge(user_id: Uuid) -> challenge::ActiveModel {
    let now = Utc::now();
    challenge::ActiveModel {
        id: Set(uuid::Uuid::now_v7()),
        title: Set("challenge".to_string()),
        text: Set("text".to_string()),
        date: Set(now.date_naive()),
        archived: Set(false),
        accomplished_at: Set(None),
        user_id: Set(user_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
}

pub trait ChallengeFactory {
    fn title(self, title: String) -> challenge::ActiveModel;
    fn archived(self, archived: bool) -> challenge::ActiveModel;
    fn accomplished_at(self, accomplished_at: Option<DateTimeWithTimeZone>) -> challenge::ActiveModel;
}

impl ChallengeFactory for challenge::ActiveModel {
    fn title(mut self, title: String) -> challenge::ActiveModel {
        self.title = Set(title);
        self
    }

    fn archived(mut self, archived: bool) -> challenge::ActiveModel {
        self.archived = Set(archived);
        self
    }

    fn accomplished_at(mut self, accomplished_at: Option<DateTimeWithTimeZone>) -> challenge::ActiveModel {
        self.accomplished_at = Set(accomplished_at);
        self
    }
}
