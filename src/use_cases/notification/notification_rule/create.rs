use chrono::{
    TimeDelta,
    Weekday::{Fri, Mon, Sat, Sun, Thu, Tue, Wed},
};
use db_adapters::notification_rule_adapter::{
    CreateNotificationRuleParams, NotificationRuleAdapter, NotificationRuleMutation,
};
use entities::{custom_methods::user::UserTimezoneTrait, user as user_entity};

use crate::{
    notification::notification_rule::types::{NotificationRuleCreateRequest, RecurrenceType},
    UseCaseError,
};

pub async fn create_notification_rules<'a>(
    user: user_entity::Model,
    notification_rule_adapter: NotificationRuleAdapter<'a>,
    params: NotificationRuleCreateRequest,
) -> Result<(), UseCaseError> {
    let user_timezone_offset = user.get_user_timezone_offset();
    let (utc_time, overflow) = params
        .time
        .overflowing_sub_signed(TimeDelta::hours(user_timezone_offset.into()));
    let weekdays = match params.recurrence_type {
        RecurrenceType::Everyday => vec![Mon, Tue, Wed, Thu, Fri, Sat, Sun],
        RecurrenceType::Weekday => {
            if overflow == 0 {
                vec![Mon, Tue, Wed, Thu, Fri]
            } else if overflow > 0 {
                vec![Mon, Tue, Wed, Thu, Sun]
            } else {
                vec![Tue, Wed, Thu, Fri, Sat]
            }
        }
        RecurrenceType::Weekend => {
            if overflow == 0 {
                vec![Sat, Sun]
            } else if overflow > 0 {
                vec![Fri, Sat]
            } else {
                vec![Mon, Sun]
            }
        }
        RecurrenceType::Unknown => {
            return Err(UseCaseError::BadRequest(
                "Unknown recurrence_type".to_string(),
            ))
        }
    };

    let notification_rule_params = weekdays
        .into_iter()
        .map(|weekday| CreateNotificationRuleParams {
            user_id: user.id,
            r#type: params.r#type.clone(),
            weekday,
            utc_time,
            action_id: None,
        })
        .collect::<Vec<_>>();

    notification_rule_adapter
        .create_many(notification_rule_params)
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}
