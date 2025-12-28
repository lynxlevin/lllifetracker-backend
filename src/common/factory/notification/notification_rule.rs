use chrono::{
    NaiveTime,
    Weekday::{self, Fri, Mon, Sat, Sun, Thu, Tue, Wed},
};
use entities::{notification_rule, sea_orm_active_enums::NotificationType};
use sea_orm::{ActiveValue::NotSet, DbConn, DbErr, EntityTrait, Set};
use uuid::Uuid;

pub fn notification_rule(user_id: Uuid) -> notification_rule::ActiveModel {
    notification_rule::ActiveModel {
        id: Set(Uuid::now_v7()),
        user_id: Set(user_id),
        r#type: NotSet,
        weekday: NotSet,
        utc_time: NotSet,
        action_id: NotSet,
    }
}

pub trait NotificationRuleFactory {
    fn r#type(self, r#type: NotificationType) -> notification_rule::ActiveModel;
    fn weekday(self, weekday: Weekday) -> notification_rule::ActiveModel;
    fn utc_time(self, utc_time: NaiveTime) -> notification_rule::ActiveModel;
}

impl NotificationRuleFactory for notification_rule::ActiveModel {
    fn r#type(mut self, r#type: NotificationType) -> notification_rule::ActiveModel {
        self.r#type = Set(r#type);
        self
    }

    fn weekday(mut self, weekday: Weekday) -> notification_rule::ActiveModel {
        self.weekday = Set(weekday as i16);
        self
    }

    fn utc_time(mut self, utc_time: NaiveTime) -> notification_rule::ActiveModel {
        self.utc_time = Set(utc_time);
        self
    }
}

pub struct NotificationRuleSet {
    pub sunday: Option<notification_rule::Model>,
    pub monday: Option<notification_rule::Model>,
    pub tuesday: Option<notification_rule::Model>,
    pub wednesday: Option<notification_rule::Model>,
    pub thursday: Option<notification_rule::Model>,
    pub friday: Option<notification_rule::Model>,
    pub saturday: Option<notification_rule::Model>,
}

pub async fn create_everyday_rules(
    user_id: Uuid,
    db: &DbConn,
    r#type: NotificationType,
    utc_time: NaiveTime,
) -> Result<(), DbErr> {
    let mut rules = vec![];
    for weekday in [Mon, Tue, Wed, Thu, Fri, Sat, Sun] {
        rules.push(
            notification_rule(user_id)
                .r#type(r#type.clone())
                .utc_time(utc_time)
                .weekday(weekday),
        );
    }
    notification_rule::Entity::insert_many(rules)
        .exec(db)
        .await?;
    Ok(())
}

pub async fn create_weekday_rules(
    user_id: Uuid,
    db: &DbConn,
    r#type: NotificationType,
    utc_time: NaiveTime,
    is_prev_day_in_utc: bool,
) -> Result<(), DbErr> {
    let mut rules = vec![];
    let weekdays = if is_prev_day_in_utc {
        [Mon, Tue, Wed, Thu, Sun]
    } else {
        [Mon, Tue, Wed, Thu, Fri]
    };
    for weekday in weekdays {
        rules.push(
            notification_rule(user_id)
                .r#type(r#type.clone())
                .utc_time(utc_time)
                .weekday(weekday),
        );
    }
    notification_rule::Entity::insert_many(rules)
        .exec(db)
        .await?;
    Ok(())
}

pub async fn create_weekend_rules(
    user_id: Uuid,
    db: &DbConn,
    r#type: NotificationType,
    utc_time: NaiveTime,
    is_prev_day_in_utc: bool,
) -> Result<(), DbErr> {
    let mut rules = vec![];
    let weekdays = if is_prev_day_in_utc {
        [Fri, Sat]
    } else {
        [Sat, Sun]
    };
    for weekday in weekdays {
        rules.push(
            notification_rule(user_id)
                .r#type(r#type.clone())
                .utc_time(utc_time)
                .weekday(weekday),
        );
    }
    notification_rule::Entity::insert_many(rules)
        .exec(db)
        .await?;
    Ok(())
}
