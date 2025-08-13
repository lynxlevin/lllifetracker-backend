use chrono::{DateTime, FixedOffset, Utc};

use crate::{sea_orm_active_enums::TimezoneEnum, user};

pub trait UserTimezoneTrait {
    fn to_user_timezone(&self, datetime: DateTime<Utc>) -> DateTime<FixedOffset>;
}

impl UserTimezoneTrait for user::Model {
    fn to_user_timezone(&self, datetime: DateTime<Utc>) -> DateTime<FixedOffset> {
        match self.timezone {
            TimezoneEnum::AsiaTokyo => {
                datetime.with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap())
            }
            TimezoneEnum::Utc => datetime.fixed_offset(),
        }
    }
}
