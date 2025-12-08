use std::future::Future;

use chrono::{NaiveTime, Weekday};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, DbConn, DbErr, EntityTrait, ModelTrait, PaginatorTrait,
    QueryFilter, Select,
};

use entities::{
    notification_rule::{ActiveModel, Column, Entity, Model},
    sea_orm_active_enums::NotificationType,
    user,
};
use uuid::Uuid;

#[derive(Clone)]
pub struct NotificationRuleAdapter<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> NotificationRuleAdapter<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self {
            db,
            query: Entity::find(),
        }
    }
}

pub trait NotificationRuleFilter {
    fn filter_eq_user(self, user: &user::Model) -> Self;
    fn filter_eq_type(self, r#type: NotificationType) -> Self;
}

impl NotificationRuleFilter for NotificationRuleAdapter<'_> {
    fn filter_eq_user(mut self, user: &user::Model) -> Self {
        self.query = self.query.filter(Column::UserId.eq(user.id));
        self
    }

    fn filter_eq_type(mut self, r#type: NotificationType) -> Self {
        self.query = self.query.filter(Column::Type.eq(r#type));
        self
    }
}

pub trait NotificationRuleQuery {
    fn get_all(self) -> impl Future<Output = Result<Vec<Model>, DbErr>>;
    fn get_count(self) -> impl Future<Output = Result<u64, DbErr>>;
}

impl NotificationRuleQuery for NotificationRuleAdapter<'_> {
    async fn get_all(self) -> Result<Vec<Model>, DbErr> {
        self.query.all(self.db).await
    }

    async fn get_count(self) -> Result<u64, DbErr> {
        self.query.count(self.db).await
    }
}

#[derive(Debug, Clone)]
pub struct CreateNotificationRuleParams {
    pub user_id: Uuid,
    pub r#type: NotificationType,
    pub weekday: Weekday,
    pub utc_time: NaiveTime,
    pub action_id: Option<Uuid>,
}

pub trait NotificationRuleMutation {
    fn create_many(
        self,
        params: Vec<CreateNotificationRuleParams>,
    ) -> impl Future<Output = Result<(), DbErr>>;
    fn delete_many(self, notification_rules: Vec<Model>)
        -> impl Future<Output = Result<(), DbErr>>;
}

impl NotificationRuleMutation for NotificationRuleAdapter<'_> {
    async fn create_many(self, params: Vec<CreateNotificationRuleParams>) -> Result<(), DbErr> {
        let notification_rules = params
            .iter()
            .map(|param| ActiveModel {
                id: Set(Uuid::now_v7()),
                user_id: Set(param.user_id),
                r#type: Set(param.r#type.clone()),
                weekday: Set(param.weekday.num_days_from_monday() as i16),
                utc_time: Set(param.utc_time),
                action_id: Set(param.action_id),
            })
            .collect::<Vec<_>>();
        Entity::insert_many(notification_rules)
            .on_empty_do_nothing()
            .exec(self.db)
            .await?;
        Ok(())
    }

    async fn delete_many(self, notification_rules: Vec<Model>) -> Result<(), DbErr> {
        // FIXME: SeaOrm delete_many methods needs filters to be chained after,
        //        find a way to incorporate into this adapter.
        for rule in notification_rules {
            rule.delete(self.db).await.map(|_| ())?
        }
        Ok(())
    }
}
