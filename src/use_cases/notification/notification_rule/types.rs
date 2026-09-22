use std::{fmt::Display, str::FromStr};

use chrono::NaiveTime;

use entities::notification_rule::NotificationType;
use sea_orm::sea_query::ValueTypeErr;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, PartialEq, Clone)]
pub enum RecurrenceType {
    Everyday,
    Weekday,
    Weekend,
    Unknown,
}

impl FromStr for RecurrenceType {
    type Err = ValueTypeErr;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Everyday" => Ok(Self::Everyday),
            "Weekday" => Ok(Self::Weekday),
            "Weekend" => Ok(Self::Weekend),
            "Unknown" => Ok(Self::Unknown),
            _ => Err(ValueTypeErr),
        }
    }
}

impl Display for RecurrenceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Everyday => "Everyday",
                Self::Weekday => "Weekday",
                Self::Weekend => "Weekend",
                Self::Unknown => "Unknown",
            }
        )
    }
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

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct NotificationRuleDeleteQuery {
    pub r#type: NotificationType,
}
