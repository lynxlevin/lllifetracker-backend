use chrono::NaiveTime;

use entities::sea_orm_active_enums::NotificationType;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, PartialEq, Clone)]
pub enum RecurrenceType {
    Everyday,
    Weekday,
    Weekend,
    Unknown,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct NotificationRuleVisible {
    pub r#type: NotificationType,
    pub recurrence_type: RecurrenceType,
    pub time: NaiveTime,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct NotificationRuleCreateRequest {
    pub r#type: NotificationType,
    pub recurrence_type: RecurrenceType,
    pub time: NaiveTime,
}
