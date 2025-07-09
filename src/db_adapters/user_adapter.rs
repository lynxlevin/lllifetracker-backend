use std::future::Future;

use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, IntoActiveModel, QueryFilter,
    Select, Set,
};
use uuid::Uuid;

use entities::{
    sea_orm_active_enums::TimezoneEnum,
    user::{ActiveModel, Column, Entity, Model},
};

#[derive(Clone)]
pub struct UserAdapter<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> UserAdapter<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self {
            db,
            query: Entity::find(),
        }
    }
}

pub trait UserFilter {
    fn filter_eq_is_active(self, is_active: bool) -> Self;
}

impl UserFilter for UserAdapter<'_> {
    fn filter_eq_is_active(mut self, is_active: bool) -> Self {
        self.query = self.query.filter(Column::IsActive.eq(is_active));
        self
    }
}

pub trait UserQuery {
    fn get_by_id(self, id: Uuid) -> impl Future<Output = Result<Option<Model>, DbErr>>;
    fn get_by_email(self, email: String) -> impl Future<Output = Result<Option<Model>, DbErr>>;
}

impl UserQuery for UserAdapter<'_> {
    async fn get_by_id(self, id: Uuid) -> Result<Option<Model>, DbErr> {
        self.query.filter(Column::Id.eq(id)).one(self.db).await
    }

    async fn get_by_email(self, email: String) -> Result<Option<Model>, DbErr> {
        self.query
            .filter(Column::Email.eq(email))
            .one(self.db)
            .await
    }
}

#[derive(Debug, Clone)]
pub struct CreateUserParams {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub is_active: bool,
}

pub trait UserMutation {
    fn create(self, params: CreateUserParams) -> impl Future<Output = Result<Model, DbErr>>;
    fn activate(self, user: Model) -> impl Future<Output = Result<Model, DbErr>>;
    fn update_password(
        self,
        user: Model,
        password: String,
    ) -> impl Future<Output = Result<Model, DbErr>>;
    fn update_first_track_at(
        self,
        user: Model,
        first_track_at: Option<DateTime<FixedOffset>>,
    ) -> impl Future<Output = Result<Model, DbErr>>;
}

impl UserMutation for UserAdapter<'_> {
    async fn create(self, params: CreateUserParams) -> Result<Model, DbErr> {
        let now = Utc::now();
        ActiveModel {
            id: Set(uuid::Uuid::now_v7()),
            email: Set(params.email),
            password: Set(params.password),
            first_name: Set(params.first_name),
            last_name: Set(params.last_name),
            is_active: Set(params.is_active),
            timezone: Set(TimezoneEnum::AsiaTokyo),
            first_track_at: Set(None),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        }
        .insert(self.db)
        .await
    }

    async fn activate(self, user: Model) -> Result<Model, DbErr> {
        let mut user = user.into_active_model();
        user.is_active = Set(true);
        user.updated_at = Set(Utc::now().into());
        user.update(self.db).await
    }

    async fn update_password(self, user: Model, password: String) -> Result<Model, DbErr> {
        let mut user = user.into_active_model();
        user.password = Set(password);
        user.updated_at = Set(Utc::now().into());
        user.update(self.db).await
    }

    async fn update_first_track_at(
        self,
        user: Model,
        first_track_at: Option<DateTime<FixedOffset>>,
    ) -> Result<Model, DbErr> {
        let mut user = user.into_active_model();
        user.first_track_at = Set(first_track_at);
        user.updated_at = Set(Utc::now().into());
        user.update(self.db).await
    }
}
