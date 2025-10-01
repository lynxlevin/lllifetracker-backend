use std::future::Future;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, ModelTrait, QueryFilter, Select, Set,
};
use uuid::Uuid;

use entities::{
    user,
    web_push_subscription::{ActiveModel, Column, Entity, Model},
};

#[derive(Clone)]
pub struct WebPushSubscriptionAdapter<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> WebPushSubscriptionAdapter<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self {
            db,
            query: Entity::find(),
        }
    }
}

pub trait WebPushSubscriptionQuery {
    fn get_by_user(self, user: &user::Model) -> impl Future<Output = Result<Option<Model>, DbErr>>;
}

impl WebPushSubscriptionQuery for WebPushSubscriptionAdapter<'_> {
    async fn get_by_user(self, user: &user::Model) -> Result<Option<Model>, DbErr> {
        self.query
            .filter(Column::UserId.eq(user.id))
            .one(self.db)
            .await
    }
}

#[derive(Debug, Clone)]
pub struct CreateWebPushSubscriptionParams {
    pub user_id: Uuid,
    pub device_name: String,
    pub endpoint: String,
    pub expiration_epoch_time: i64,
    pub p256dh_key: String,
    pub auth_key: String,
}

pub trait WebPushSubscriptionMutation {
    fn create(
        self,
        params: CreateWebPushSubscriptionParams,
    ) -> impl Future<Output = Result<Model, DbErr>>;
    fn delete(self, web_push_subscription: Model) -> impl Future<Output = Result<(), DbErr>>;
}

impl WebPushSubscriptionMutation for WebPushSubscriptionAdapter<'_> {
    async fn create(self, params: CreateWebPushSubscriptionParams) -> Result<Model, DbErr> {
        ActiveModel {
            id: Set(Uuid::now_v7()),
            user_id: Set(params.user_id),
            device_name: Set(params.device_name),
            endpoint: Set(params.endpoint),
            expiration_epoch_time: Set(params.expiration_epoch_time),
            p256dh_key: Set(params.p256dh_key),
            auth_key: Set(params.auth_key),
        }
        .insert(self.db)
        .await
    }

    async fn delete(self, web_push_subscription: Model) -> Result<(), DbErr> {
        web_push_subscription.delete(self.db).await.map(|_| ())
    }
}
