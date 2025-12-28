use chrono::Weekday;

use crate::notification_rule;

pub trait NotificationRuleTrait {
    fn get_utc_weekday(&self) -> Weekday;
}

impl NotificationRuleTrait for notification_rule::Model {
    fn get_utc_weekday(&self) -> Weekday {
        match self.weekday {
            0 | 1 | 2 | 3 | 4 | 5 | 6 => {
                let weekday: u8 = self.weekday.try_into().unwrap();
                Weekday::try_from(weekday).unwrap()
            }
            _ => unreachable!(
                "This should not happen, weekday has DB check to make sure it's between 0 to 6."
            ),
        }
    }
}
