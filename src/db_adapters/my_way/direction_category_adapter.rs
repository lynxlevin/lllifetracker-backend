use std::future::Future;

use sea_orm::{
    sea_query::NullOrdering::Last, ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait,
    IntoActiveModel, ModelTrait, Order, PaginatorTrait, QueryFilter, QueryOrder, Select, Set,
};
use uuid::Uuid;

use entities::{
    direction_category::{ActiveModel, Column, Entity, Model},
    user,
};

#[derive(Clone)]
pub struct DirectionCategoryAdapter<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> DirectionCategoryAdapter<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self {
            db,
            query: Entity::find(),
        }
    }
}

pub trait DirectionCategoryFilter {
    fn filter_eq_user(self, user: &user::Model) -> Self;
    fn filter_in_ids(self, ids: Vec<Uuid>) -> Self;
}

impl DirectionCategoryFilter for DirectionCategoryAdapter<'_> {
    fn filter_eq_user(mut self, user: &user::Model) -> Self {
        self.query = self.query.filter(Column::UserId.eq(user.id));
        self
    }

    fn filter_in_ids(mut self, ids: Vec<Uuid>) -> Self {
        self.query = self.query.filter(Column::Id.is_in(ids));
        self
    }
}

pub trait DirectionCategoryOrder {
    fn order_by_ordering_nulls_last(self, order: Order) -> Self;
    fn order_by_id(self, order: Order) -> Self;
}

impl DirectionCategoryOrder for DirectionCategoryAdapter<'_> {
    fn order_by_ordering_nulls_last(mut self, order: Order) -> Self {
        self.query = self
            .query
            .order_by_with_nulls(Column::Ordering, order, Last);
        self
    }

    fn order_by_id(mut self, order: Order) -> Self {
        self.query = self.query.order_by(Column::Id, order);
        self
    }
}

pub trait DirectionCategoryQuery {
    fn get_all(self) -> impl Future<Output = Result<Vec<Model>, DbErr>>;
    fn get_by_id(self, id: Uuid) -> impl Future<Output = Result<Option<Model>, DbErr>>;
    fn get_count(self) -> impl Future<Output = Result<u64, DbErr>>;
}

impl DirectionCategoryQuery for DirectionCategoryAdapter<'_> {
    async fn get_all(self) -> Result<Vec<Model>, DbErr> {
        self.query.all(self.db).await
    }

    async fn get_by_id(self, id: Uuid) -> Result<Option<Model>, DbErr> {
        self.query.filter(Column::Id.eq(id)).one(self.db).await
    }

    async fn get_count(self) -> Result<u64, DbErr> {
        self.query.count(self.db).await
    }
}

#[derive(Debug, Clone)]
pub struct CreateDirectionCategoryParams {
    pub name: String,
    pub ordering: Option<i32>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct UpdateDirectionCategoryParams {
    pub name: String,
}

pub trait DirectionCategoryMutation {
    fn create(
        self,
        params: CreateDirectionCategoryParams,
    ) -> impl Future<Output = Result<Model, DbErr>>;
    fn update(
        self,
        category: Model,
        params: UpdateDirectionCategoryParams,
    ) -> impl Future<Output = Result<Model, DbErr>>;
    fn bulk_update_ordering(
        self,
        params: Vec<(Model, Option<i32>)>,
    ) -> impl Future<Output = Result<(), DbErr>>;
    fn delete(self, category: Model) -> impl Future<Output = Result<(), DbErr>>;
}

impl DirectionCategoryMutation for DirectionCategoryAdapter<'_> {
    async fn create(self, params: CreateDirectionCategoryParams) -> Result<Model, DbErr> {
        ActiveModel {
            id: Set(uuid::Uuid::now_v7()),
            user_id: Set(params.user_id),
            name: Set(params.name),
            ordering: Set(params.ordering),
        }
        .insert(self.db)
        .await
    }

    async fn update(
        self,
        category: Model,
        params: UpdateDirectionCategoryParams,
    ) -> Result<Model, DbErr> {
        let mut category = category.into_active_model();
        category.name = Set(params.name);
        category.update(self.db).await
    }

    async fn bulk_update_ordering(self, params: Vec<(Model, Option<i32>)>) -> Result<(), DbErr> {
        for (category, ordering) in params {
            let mut category = category.into_active_model();
            category.ordering = Set(ordering);
            category.update(self.db).await?;
        }
        Ok(())
    }

    async fn delete(self, category: Model) -> Result<(), DbErr> {
        category.delete(self.db).await.map(|_| ())
    }
}
