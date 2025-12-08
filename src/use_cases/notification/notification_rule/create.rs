use chrono::{
    NaiveTime, TimeDelta, Timelike,
    Weekday::{self, Fri, Mon, Sat, Sun, Thu, Tue, Wed},
};
use db_adapters::notification_rule_adapter::{
    CreateNotificationRuleParams, NotificationRuleAdapter, NotificationRuleFilter,
    NotificationRuleMutation, NotificationRuleQuery,
};
use entities::{
    custom_methods::user::UserTimezoneTrait, sea_orm_active_enums::NotificationType,
    user as user_entity,
};

use crate::{
    notification::notification_rule::types::{NotificationRuleCreateRequest, RecurrenceType},
    UseCaseError,
};

pub async fn create_notification_rules<'a>(
    user: user_entity::Model,
    notification_rule_adapter: NotificationRuleAdapter<'a>,
    params: NotificationRuleCreateRequest,
) -> Result<(), UseCaseError> {
    let params = parse_params(params, notification_rule_adapter.clone(), &user).await?;

    let notification_rule_params = params
        .weekdays
        .into_iter()
        .map(|weekday| CreateNotificationRuleParams {
            user_id: user.id,
            r#type: params.r#type.clone(),
            weekday,
            utc_time: params.utc_time,
            action_id: None,
        })
        .collect::<Vec<_>>();

    notification_rule_adapter
        .create_many(notification_rule_params)
        .await
        .map(|_| ())
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))
}

struct ParsedParam {
    utc_time: NaiveTime,
    weekdays: Vec<Weekday>,
    r#type: NotificationType,
}

async fn parse_params<'a>(
    params: NotificationRuleCreateRequest,
    adapter: NotificationRuleAdapter<'a>,
    user: &user_entity::Model,
) -> Result<ParsedParam, UseCaseError> {
    let exists_same_type_rules = adapter
        .filter_eq_user(user)
        .filter_eq_type(params.r#type.clone())
        .get_count()
        .await
        .map_err(|e| UseCaseError::InternalServerError(format!("{:?}", e)))?
        > 0;
    if exists_same_type_rules {
        return Err(UseCaseError::Conflict(
            "Notification rules for the same type already exists.".to_string(),
        ));
    }

    let user_timezone_offset = user.get_user_timezone_offset();
    let (utc_time, overflow) = params
        .time
        .overflowing_sub_signed(TimeDelta::hours(user_timezone_offset.into()));

    if utc_time.second() != 0 {
        return Err(UseCaseError::BadRequest(
            "Seconds in time fields must be zero.".to_string(),
        ));
    }
    if utc_time.minute() % 10 != 0 {
        return Err(UseCaseError::BadRequest(
            "Minutes in time fields must be multiples of ten.".to_string(),
        ));
    }

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
    Ok(ParsedParam {
        utc_time,
        weekdays,
        r#type: params.r#type,
    })
}
