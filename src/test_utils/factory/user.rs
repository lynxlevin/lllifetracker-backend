use entities::user;
use chrono::Utc;
use sea_orm::Set;

#[cfg(test)]
pub fn user() -> user::ActiveModel {
    use entities::sea_orm_active_enums::TimezoneEnum;

    let now = Utc::now();
    user::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        email: Set(format!("{}@test.com", uuid::Uuid::new_v4().to_string())),
        password: Set("password".to_string()),
        first_name: Set("Lynx".to_string()),
        last_name: Set("Levin".to_string()),
        timezone: Set(TimezoneEnum::AsiaTokyo),
        is_active: Set(true),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
    }
}

#[cfg(test)]
pub trait UserFactory {
    fn is_active(self, is_active: bool) -> user::ActiveModel;
}

#[cfg(test)]
impl UserFactory for user::ActiveModel {
    fn is_active(mut self, is_active: bool) -> user::ActiveModel {
        self.is_active = Set(is_active);
        self
    }
}
