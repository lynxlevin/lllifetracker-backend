use std::future::Future;

use chrono::Utc;
use sea_orm::{
    sea_query::NullOrdering::Last, ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait,
    IntoActiveModel, Order, QueryFilter, QueryOrder, Select, Set, TransactionError,
    TransactionTrait,
};
use uuid::Uuid;

use entities::{
    ambition::{ActiveModel, Column, Entity, Model},
    tag, user,
};

#[derive(Clone)]
pub struct AmbitionAdapter<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> AmbitionAdapter<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self {
            db,
            query: Entity::find(),
        }
    }
}

pub trait AmbitionFilter {
    fn filter_eq_user(self, user: &user::Model) -> Self;
    fn filter_eq_archived(self, archived: bool) -> Self;
    fn filter_in_ids(self, ids: Vec<Uuid>) -> Self;
}

impl AmbitionFilter for AmbitionAdapter<'_> {
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

pub trait AmbitionOrder {
    fn order_by_ordering_nulls_last(self, order: Order) -> Self;
    fn order_by_created_at(self, order: Order) -> Self;
}

impl AmbitionOrder for AmbitionAdapter<'_> {
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

pub trait AmbitionQuery {
    fn get_all(self) -> impl Future<Output = Result<Vec<Model>, DbErr>>;
    fn get_by_id(self, id: Uuid) -> impl Future<Output = Result<Option<Model>, DbErr>>;
}

impl AmbitionQuery for AmbitionAdapter<'_> {
    async fn get_all(self) -> Result<Vec<Model>, DbErr> {
        self.query.all(self.db).await
    }

    async fn get_by_id(self, id: Uuid) -> Result<Option<Model>, DbErr> {
        self.query.filter(Column::Id.eq(id)).one(self.db).await
    }
}

#[derive(Debug, Clone)]
pub struct CreateAmbitionParams {
    pub name: String,
    pub description: Option<String>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct UpdateAmbitionParams {
    pub name: String,
    pub description: Option<String>,
}

pub trait AmbitionMutation {
    fn create_with_tag(
        self,
        params: CreateAmbitionParams,
    ) -> impl Future<Output = Result<Model, TransactionError<DbErr>>>;
    fn update(
        self,
        ambition: Model,
        params: UpdateAmbitionParams,
    ) -> impl Future<Output = Result<Model, DbErr>>;
    fn archive(self, ambition: Model) -> impl Future<Output = Result<Model, DbErr>>;
    fn unarchive(self, ambition: Model) -> impl Future<Output = Result<Model, DbErr>>;
    fn bulk_update_ordering(
        self,
        ambitions: Vec<Model>,
        ordering: Vec<Uuid>,
    ) -> impl Future<Output = Result<(), DbErr>>;
    fn delete(self, id: Uuid, user: &user::Model) -> impl Future<Output = Result<(), DbErr>>;
}

impl AmbitionMutation for AmbitionAdapter<'_> {
    async fn create_with_tag(
        self,
        params: CreateAmbitionParams,
    ) -> Result<Model, TransactionError<DbErr>> {
        self.db
            .transaction::<_, Model, DbErr>(|txn| {
                Box::pin(async move {
                    let ambition_id = uuid::Uuid::now_v7();
                    let created_ambition = ActiveModel {
                        id: Set(ambition_id),
                        user_id: Set(params.user_id),
                        name: Set(params.name.to_owned()),
                        description: Set(params.description),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await?;
                    tag::ActiveModel {
                        id: Set(uuid::Uuid::now_v7()),
                        user_id: Set(params.user_id),
                        ambition_id: Set(Some(ambition_id)),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await?;

                    Ok(created_ambition)
                })
            })
            .await
    }

    async fn update(self, ambition: Model, params: UpdateAmbitionParams) -> Result<Model, DbErr> {
        let mut ambition = ambition.into_active_model();
        ambition.name = Set(params.name);
        ambition.description = Set(params.description);
        ambition.updated_at = Set(Utc::now().into());
        ambition.update(self.db).await
    }

    async fn archive(self, ambition: Model) -> Result<Model, DbErr> {
        let mut ambition = ambition.into_active_model();
        ambition.archived = Set(true);
        ambition.updated_at = Set(Utc::now().into());
        ambition.update(self.db).await
    }

    async fn unarchive(self, ambition: Model) -> Result<Model, DbErr> {
        let mut ambition = ambition.into_active_model();
        ambition.archived = Set(false);
        ambition.updated_at = Set(Utc::now().into());
        ambition.update(self.db).await
    }

    async fn bulk_update_ordering(
        self,
        ambitions: Vec<Model>,
        ordering: Vec<Uuid>,
    ) -> Result<(), DbErr> {
        for ambition in ambitions {
            let order = &ordering.iter().position(|id| id == &ambition.id);
            if let Some(order) = order {
                let mut ambition = ambition.into_active_model();
                ambition.ordering = Set(Some((order + 1) as i32));
                ambition.update(self.db).await?;
            }
        }
        Ok(())
    }

    async fn delete(self, id: Uuid, user: &user::Model) -> Result<(), DbErr> {
        Entity::delete(ActiveModel {
            id: Set(id),
            user_id: Set(user.id),
            ..Default::default()
        })
        .exec(self.db)
        .await
        .map(|_| ())
    }
}
