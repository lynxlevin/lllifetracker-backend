use chrono::{NaiveTime, Weekday};
use entities::{notification_rule, sea_orm_active_enums::NotificationType};
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, DbConn, DbErr, Set};
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
) -> Result<NotificationRuleSet, DbErr> {
    let base = notification_rule(user_id).r#type(r#type).utc_time(utc_time);
    Ok(NotificationRuleSet {
        sunday: Some(base.clone().weekday(Weekday::Sun).insert(db).await?),
        monday: Some(base.clone().weekday(Weekday::Mon).insert(db).await?),
        tuesday: Some(base.clone().weekday(Weekday::Tue).insert(db).await?),
        wednesday: Some(base.clone().weekday(Weekday::Wed).insert(db).await?),
        thursday: Some(base.clone().weekday(Weekday::Thu).insert(db).await?),
        friday: Some(base.clone().weekday(Weekday::Fri).insert(db).await?),
        saturday: Some(base.clone().weekday(Weekday::Sat).insert(db).await?),
    })
}

pub async fn create_weekday_rules(
    user_id: Uuid,
    db: &DbConn,
    r#type: NotificationType,
    utc_time: NaiveTime,
    is_prev_day_in_utc: bool,
) -> Result<NotificationRuleSet, DbErr> {
    let base = notification_rule(user_id).r#type(r#type).utc_time(utc_time);
    if is_prev_day_in_utc {
        Ok(NotificationRuleSet {
            sunday: Some(base.clone().weekday(Weekday::Sun).insert(db).await?),
            monday: Some(base.clone().weekday(Weekday::Mon).insert(db).await?),
            tuesday: Some(base.clone().weekday(Weekday::Tue).insert(db).await?),
            wednesday: Some(base.clone().weekday(Weekday::Wed).insert(db).await?),
            thursday: Some(base.clone().weekday(Weekday::Thu).insert(db).await?),
            friday: None,
            saturday: None,
        })
    } else {
        Ok(NotificationRuleSet {
            sunday: None,
            monday: Some(base.clone().weekday(Weekday::Mon).insert(db).await?),
            tuesday: Some(base.clone().weekday(Weekday::Tue).insert(db).await?),
            wednesday: Some(base.clone().weekday(Weekday::Wed).insert(db).await?),
            thursday: Some(base.clone().weekday(Weekday::Thu).insert(db).await?),
            friday: Some(base.clone().weekday(Weekday::Fri).insert(db).await?),
            saturday: None,
        })
    }
}

pub async fn create_weekend_rules(
    user_id: Uuid,
    db: &DbConn,
    r#type: NotificationType,
    utc_time: NaiveTime,
    is_prev_day_in_utc: bool,
) -> Result<NotificationRuleSet, DbErr> {
    let base = notification_rule(user_id).r#type(r#type).utc_time(utc_time);
    if is_prev_day_in_utc {
        Ok(NotificationRuleSet {
            sunday: None,
            monday: None,
            tuesday: None,
            wednesday: None,
            thursday: None,
            friday: Some(base.clone().weekday(Weekday::Fri).insert(db).await?),
            saturday: Some(base.clone().weekday(Weekday::Sat).insert(db).await?),
        })
    } else {
        Ok(NotificationRuleSet {
            sunday: Some(base.clone().weekday(Weekday::Sun).insert(db).await?),
            monday: None,
            tuesday: None,
            wednesday: None,
            thursday: None,
            friday: None,
            saturday: Some(base.clone().weekday(Weekday::Sat).insert(db).await?),
        })
    }
}
