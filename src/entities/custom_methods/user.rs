use chrono::{DateTime, FixedOffset, Utc};

use crate::{sea_orm_active_enums::TimezoneEnum, user};

pub trait UserTimezoneTrait {
    fn to_user_timezone(&self, datetime: DateTime<Utc>) -> DateTime<FixedOffset>;
    fn get_user_timezone_offset(&self) -> i32;
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

    fn get_user_timezone_offset(&self) -> i32 {
        match self.timezone {
            TimezoneEnum::AsiaTokyo => 9,
            TimezoneEnum::Utc => 0,
        }
    }
}
