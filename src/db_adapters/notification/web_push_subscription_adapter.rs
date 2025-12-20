use std::future::Future;

use sea_orm::{
    sea_query::OnConflict, ColumnTrait, DbConn, DbErr, EntityTrait, ModelTrait, QueryFilter,
    Select, Set,
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

pub trait WebPushSubscriptionFilter {
    fn filter_in_user_ids(self, user_ids: Vec<Uuid>) -> Self;
}

impl WebPushSubscriptionFilter for WebPushSubscriptionAdapter<'_> {
    fn filter_in_user_ids(mut self, user_ids: Vec<Uuid>) -> Self {
        self.query = self.query.filter(Column::UserId.is_in(user_ids));
        self
    }
}

pub trait WebPushSubscriptionQuery {
    fn get_by_user(self, user: &user::Model) -> impl Future<Output = Result<Option<Model>, DbErr>>;
    fn get_all(self) -> impl Future<Output = Result<Vec<Model>, DbErr>>;
}

impl WebPushSubscriptionQuery for WebPushSubscriptionAdapter<'_> {
    async fn get_by_user(self, user: &user::Model) -> Result<Option<Model>, DbErr> {
        self.query
            .filter(Column::UserId.eq(user.id))
            .one(self.db)
            .await
    }

    async fn get_all(self) -> Result<Vec<Model>, DbErr> {
        self.query.all(self.db).await
    }
}

#[derive(Debug, Clone)]
pub struct CreateWebPushSubscriptionParams {
    pub user_id: Uuid,
    pub device_name: String,
    pub endpoint: String,
    pub expiration_epoch_time: Option<i64>,
    pub p256dh_key: String,
    pub auth_key: String,
}

pub trait WebPushSubscriptionMutation {
    fn upsert(
        self,
        params: CreateWebPushSubscriptionParams,
    ) -> impl Future<Output = Result<Model, DbErr>>;
    fn delete(self, web_push_subscription: Model) -> impl Future<Output = Result<(), DbErr>>;
}

impl WebPushSubscriptionMutation for WebPushSubscriptionAdapter<'_> {
    async fn upsert(self, params: CreateWebPushSubscriptionParams) -> Result<Model, DbErr> {
        let subscription = ActiveModel {
            id: Set(Uuid::now_v7()),
            user_id: Set(params.user_id),
            device_name: Set(params.device_name.clone()),
            endpoint: Set(params.endpoint.clone()),
            expiration_epoch_time: Set(params.expiration_epoch_time),
            p256dh_key: Set(params.p256dh_key.clone()),
            auth_key: Set(params.auth_key.clone()),
        };
        Entity::insert(subscription)
            .on_conflict(
                OnConflict::column(Column::UserId)
                    .update_columns([
                        Column::DeviceName,
                        Column::Endpoint,
                        Column::ExpirationEpochTime,
                        Column::P256dhKey,
                        Column::AuthKey,
                    ])
                    .to_owned(),
            )
            .exec(self.db)
            .await
            .map(|res| Model {
                id: res.last_insert_id,
                user_id: params.user_id,
                device_name: params.device_name,
                endpoint: params.endpoint,
                expiration_epoch_time: params.expiration_epoch_time,
                p256dh_key: params.p256dh_key,
                auth_key: params.auth_key,
            })
    }

    async fn delete(self, web_push_subscription: Model) -> Result<(), DbErr> {
        web_push_subscription.delete(self.db).await.map(|_| ())
    }
}
