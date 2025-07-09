use chrono::{DateTime, FixedOffset, Utc};
use entities::user;
use sea_orm::Set;

pub fn user() -> user::ActiveModel {
    use entities::sea_orm_active_enums::TimezoneEnum;

    let now = Utc::now();
    user::ActiveModel {
        id: Set(uuid::Uuid::now_v7()),
        email: Set(format!("{}@test.com", uuid::Uuid::now_v7().to_string())),
        password: Set("password".to_string()),
        first_name: Set("Lynx".to_string()),
        last_name: Set("Levin".to_string()),
        timezone: Set(TimezoneEnum::AsiaTokyo),
        is_active: Set(true),
        first_track_at: Set(None),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
}

pub trait UserFactory {
    fn is_active(self, is_active: bool) -> user::ActiveModel;
    fn password(self, hashed_password: &str) -> user::ActiveModel;
    fn first_track_at(self, first_track_at: Option<DateTime<FixedOffset>>) -> user::ActiveModel;
}

impl UserFactory for user::ActiveModel {
    fn is_active(mut self, is_active: bool) -> user::ActiveModel {
        self.is_active = Set(is_active);
        self
    }

    fn password(mut self, hashed_password: &str) -> user::ActiveModel {
        self.password = Set(hashed_password.to_string());
        self
    }

    fn first_track_at(
        mut self,
        first_track_at: Option<DateTime<FixedOffset>>,
    ) -> user::ActiveModel {
        self.first_track_at = Set(first_track_at);
        self
    }
}
