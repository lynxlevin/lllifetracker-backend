use std::future::Future;

use sea_orm::{ColumnTrait, DbConn, DbErr, EntityTrait, QueryFilter, Select};

use entities::{
    notification_rule::{Column, Entity, Model},
    user,
};

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
}

impl NotificationRuleFilter for NotificationRuleAdapter<'_> {
    fn filter_eq_user(mut self, user: &user::Model) -> Self {
        self.query = self.query.filter(Column::UserId.eq(user.id));
        self
    }
}

pub trait NotificationRuleQuery {
    fn get_all(self) -> impl Future<Output = Result<Vec<Model>, DbErr>>;
}

impl NotificationRuleQuery for NotificationRuleAdapter<'_> {
    async fn get_all(self) -> Result<Vec<Model>, DbErr> {
        self.query.all(self.db).await
    }
}

