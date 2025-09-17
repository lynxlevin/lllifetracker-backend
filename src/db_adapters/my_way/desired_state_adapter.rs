use std::future::Future;

use chrono::Utc;
use sea_orm::{
    sea_query::NullOrdering::Last, ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait,
    IntoActiveModel, ModelTrait, Order, QueryFilter, QueryOrder, Select, Set, TransactionError,
    TransactionTrait,
};
use uuid::Uuid;

use entities::{
    desired_state::{ActiveModel, Column, Entity, Model},
    sea_orm_active_enums::TagType,
    tag, user,
};

#[derive(Clone)]
pub struct DesiredStateAdapter<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> DesiredStateAdapter<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self {
            db,
            query: Entity::find(),
        }
    }
}

pub trait DesiredStateFilter {
    fn filter_eq_user(self, user: &user::Model) -> Self;
    fn filter_eq_archived(self, archived: bool) -> Self;
    fn filter_in_ids(self, ids: Vec<Uuid>) -> Self;
}

impl DesiredStateFilter for DesiredStateAdapter<'_> {
    fn filter_eq_user(mut self, user: &user::Model) -> Self {
        self.query = self.query.filter(Column::UserId.eq(user.id));
        self
    }

    fn filter_eq_archived(mut self, archived: bool) -> Self {
        self.query = self.query.filter(Column::Archived.eq(archived));
        self
    }

    fn filter_in_ids(mut self, ids: Vec<Uuid>) -> Self {
        self.query = self.query.filter(Column::Id.is_in(ids));
        self
    }
}

pub trait DesiredStateOrder {
    fn order_by_ordering_nulls_last(self, order: Order) -> Self;
    fn order_by_created_at(self, order: Order) -> Self;
}

impl DesiredStateOrder for DesiredStateAdapter<'_> {
    fn order_by_ordering_nulls_last(mut self, order: Order) -> Self {
        self.query = self
            .query
            .order_by_with_nulls(Column::Ordering, order, Last);
        self
    }

    fn order_by_created_at(mut self, order: Order) -> Self {
        self.query = self.query.order_by(Column::CreatedAt, order);
        self
    }
}

pub trait DesiredStateQuery {
    fn get_all(self) -> impl Future<Output = Result<Vec<Model>, DbErr>>;
    fn get_by_id(self, id: Uuid) -> impl Future<Output = Result<Option<Model>, DbErr>>;
}

impl DesiredStateQuery for DesiredStateAdapter<'_> {
    async fn get_all(self) -> Result<Vec<Model>, DbErr> {
        self.query.all(self.db).await
    }

    async fn get_by_id(self, id: Uuid) -> Result<Option<Model>, DbErr> {
        self.query.filter(Column::Id.eq(id)).one(self.db).await
    }
}

#[derive(Debug, Clone)]
pub struct CreateDesiredStateParams {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub is_focused: bool,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct UpdateDesiredStateParams {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub is_focused: bool,
}

pub trait DesiredStateMutation {
    fn create_with_tag(
        self,
        params: CreateDesiredStateParams,
    ) -> impl Future<Output = Result<Model, TransactionError<DbErr>>>;
    fn update(
        self,
        desired_state: Model,
        params: UpdateDesiredStateParams,
    ) -> impl Future<Output = Result<Model, DbErr>>;
    fn archive(self, desired_state: Model) -> impl Future<Output = Result<Model, DbErr>>;
    fn unarchive(self, desired_state: Model) -> impl Future<Output = Result<Model, DbErr>>;
    fn bulk_update_ordering(
        self,
        desired_states: Vec<Model>,
        ordering: Vec<Uuid>,
    ) -> impl Future<Output = Result<(), DbErr>>;
    fn delete(self, desired_state: Model) -> impl Future<Output = Result<(), DbErr>>;
}

impl DesiredStateMutation for DesiredStateAdapter<'_> {
    async fn create_with_tag(
        self,
        params: CreateDesiredStateParams,
    ) -> Result<Model, TransactionError<DbErr>> {
        self.db
            .transaction::<_, Model, DbErr>(|txn| {
                Box::pin(async move {
                    let desired_state_id = uuid::Uuid::now_v7();
                    let created_desired_state = ActiveModel {
                        id: Set(desired_state_id),
                        user_id: Set(params.user_id),
                        name: Set(params.name.to_owned()),
                        description: Set(params.description.to_owned()),
                        category_id: Set(params.category_id),
                        is_focused: Set(params.is_focused),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await?;
                    tag::ActiveModel {
                        id: Set(uuid::Uuid::now_v7()),
                        user_id: Set(params.user_id),
                        desired_state_id: Set(Some(desired_state_id)),
                        r#type: Set(TagType::DesiredState),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await?;

                    Ok(created_desired_state)
                })
            })
            .await
    }

    async fn update(
        self,
        desired_state: Model,
        params: UpdateDesiredStateParams,
    ) -> Result<Model, DbErr> {
        let mut desired_state = desired_state.into_active_model();
        desired_state.name = Set(params.name);
        desired_state.description = Set(params.description);
        desired_state.category_id = Set(params.category_id);
        desired_state.is_focused = Set(params.is_focused);
        desired_state.updated_at = Set(Utc::now().into());
        desired_state.update(self.db).await
    }

    async fn archive(self, desired_state: Model) -> Result<Model, DbErr> {
        let mut desired_state = desired_state.into_active_model();
        desired_state.archived = Set(true);
        desired_state.updated_at = Set(Utc::now().into());
        desired_state.update(self.db).await
    }

    async fn unarchive(self, desired_state: Model) -> Result<Model, DbErr> {
        let mut desired_state = desired_state.into_active_model();
        desired_state.archived = Set(false);
        desired_state.updated_at = Set(Utc::now().into());
        desired_state.update(self.db).await
    }

    async fn bulk_update_ordering(
        self,
        desired_states: Vec<Model>,
        ordering: Vec<Uuid>,
    ) -> Result<(), DbErr> {
        for desired_state in desired_states {
            let order = &ordering.iter().position(|id| id == &desired_state.id);
            if let Some(order) = order {
                let mut desired_state = desired_state.into_active_model();
                desired_state.ordering = Set(Some((order + 1) as i32));
                desired_state.update(self.db).await?;
            }
        }
        Ok(())
    }

    async fn delete(self, desired_state: Model) -> Result<(), DbErr> {
        desired_state.delete(self.db).await.map(|_| ())
    }
}
