use crate::entities::mission_memo;
use chrono::Utc;
use sea_orm::{prelude::DateTimeWithTimeZone, Set};
use uuid::Uuid;

#[cfg(test)]
pub fn mission_memo(user_id: Uuid) -> mission_memo::ActiveModel {
    let now = Utc::now();
    mission_memo::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        title: Set("mission_memo".to_string()),
        text: Set("text".to_string()),
        date: Set(now.date_naive()),
        archived: Set(false),
        accomplished_at: Set(None),
        user_id: Set(user_id),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
}

#[cfg(test)]
pub trait MissionMemoFactory {
    fn title(self, title: String) -> mission_memo::ActiveModel;
    fn archived(self, archived: bool) -> mission_memo::ActiveModel;
    fn accomplished_at(self, accomplished_at: Option<DateTimeWithTimeZone>) -> mission_memo::ActiveModel;
}

#[cfg(test)]
impl MissionMemoFactory for mission_memo::ActiveModel {
    fn title(mut self, title: String) -> mission_memo::ActiveModel {
        self.title = Set(title);
        self
    }

    fn archived(mut self, archived: bool) -> mission_memo::ActiveModel {
        self.archived = Set(archived);
        self
    }

    fn accomplished_at(mut self, accomplished_at: Option<DateTimeWithTimeZone>) -> mission_memo::ActiveModel {
        self.accomplished_at = Set(accomplished_at);
        self
    }
}
