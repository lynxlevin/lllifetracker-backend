use std::collections::HashMap;

use chrono::{
    TimeDelta,
    Weekday::{Fri, Mon, Sat, Sun, Thu, Tue, Wed},
};
use db_adapters::notification_rule_adapter::{
    NotificationRuleAdapter, NotificationRuleFilter, NotificationRuleQuery,
};
use entities::{
    custom_methods::{notification_rule::NotificationRuleTrait, user::UserTimezoneTrait},
    sea_orm_active_enums::NotificationType,
    user as user_entity,
};
use sea_orm::ActiveEnum;

use crate::{
    notification::notification_rule::types::{NotificationRuleVisible, RecurrenceType},
    UseCaseError,
};

pub async fn list_notification_rules<'a>(
    user: user_entity::Model,
    notification_rule_adapter: NotificationRuleAdapter<'a>,
) -> Result<Vec<NotificationRuleVisible>, UseCaseError> {
    let notification_rules = notification_rule_adapter
        .filter_eq_user(&user)
        .get_all()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?;

    let user_timezone_offset = user.get_user_timezone_offset();
    let mut res = notification_rules
        .iter()
        .fold(HashMap::new(), |mut acc, rule| {
            let (time, overflow) = rule
                .utc_time
                .overflowing_add_signed(TimeDelta::try_hours(user_timezone_offset.into()).unwrap());
            let mut weekday = rule.get_utc_weekday();
            if overflow > 0 {
                weekday = weekday.succ()
            } else if overflow < 0 {
                weekday = weekday.pred()
            }

            let key = (rule.r#type.to_value(), rule.action_id, time);
            acc.entry(key).or_insert(vec![]).push(weekday);
            acc
        })
        .into_iter()
        .map(|rule_sets| {
            let mut weekdays = rule_sets.1;
            weekdays.sort_by_key(|weekday| weekday.num_days_from_monday());
            let recurrence_type = match weekdays.as_slice() {
                [Mon, Tue, Wed, Thu, Fri, Sat, Sun] => RecurrenceType::Everyday,
                [Mon, Tue, Wed, Thu, Fri] => RecurrenceType::Weekday,
                [Sat, Sun] => RecurrenceType::Weekend,
                _ => RecurrenceType::Unknown,
            };
            NotificationRuleVisible {
                r#type: NotificationType::try_from_value(&rule_sets.0 .0).unwrap(),
                recurrence_type,
                time: rule_sets.0 .2,
            }
        })
        .collect::<Vec<_>>();

    res.sort_by_key(|rule| rule.r#type.clone().into_value());
    Ok(res)
}
