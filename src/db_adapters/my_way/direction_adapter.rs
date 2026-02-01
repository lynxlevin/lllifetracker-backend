use std::future::Future;

use chrono::Utc;
use sea_orm::{
    sea_query::{
        Func,
        NullOrdering::{First, Last},
        SimpleExpr,
    },
    ActiveModelTrait, ColumnTrait, DbConn, DbErr, EntityTrait, IntoActiveModel,
    JoinType::LeftJoin,
    ModelTrait, Order, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Select, Set,
    TransactionError, TransactionTrait,
};
use uuid::Uuid;

use entities::{
    direction::{ActiveModel, Column, Entity, Model, Relation},
    direction_category,
    sea_orm_active_enums::TagType,
    tag, user,
};

#[derive(Clone)]
pub struct DirectionAdapter<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> DirectionAdapter<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self {
            db,
            query: Entity::find(),
        }
    }
}

pub trait DirectionJoin {
    fn join_category(self) -> Self;
}

impl DirectionJoin for DirectionAdapter<'_> {
    fn join_category(mut self) -> Self {
        self.query = self
            .query
            .join(LeftJoin, Relation::DirectionCategory.def());
        self
    }
}

pub trait DirectionFilter {
    fn filter_eq_user_id(self, user_id: Uuid) -> Self;
    fn filter_eq_user(self, user: &user::Model) -> Self;
    fn filter_eq_archived(self, archived: bool) -> Self;
    fn filter_in_ids(self, ids: Vec<Uuid>) -> Self;
}

impl DirectionFilter for DirectionAdapter<'_> {
    fn filter_eq_user_id(mut self, user_id: Uuid) -> Self {
        self.query = self.query.filter(Column::UserId.eq(user_id));
        self
    }

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

pub trait DirectionOrder {
    fn order_by_category_ordering_nulls_last(self, order: Order) -> Self;
    fn order_by_ordering_nulls_first(self, order: Order) -> Self;
    fn order_by_ordering_nulls_last(self, order: Order) -> Self;
    fn order_by_created_at(self, order: Order) -> Self;
}

impl DirectionOrder for DirectionAdapter<'_> {
    fn order_by_category_ordering_nulls_last(mut self, order: Order) -> Self {
        self.query =
            self.query
                .order_by_with_nulls(direction_category::Column::Ordering, order, Last);
        self
    }

    fn order_by_ordering_nulls_first(mut self, order: Order) -> Self {
        self.query = self
            .query
            .order_by_with_nulls(Column::Ordering, order, First);
        self
    }

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

pub trait DirectionQuery {
    fn get_all(self) -> impl Future<Output = Result<Vec<Model>, DbErr>>;
    fn get_by_id(self, id: Uuid) -> impl Future<Output = Result<Option<Model>, DbErr>>;
    fn get_random(self) -> impl Future<Output = Result<Option<Model>, DbErr>>;
    fn get_random_with_category(
        self,
    ) -> impl Future<Output = Result<Option<(Model, Option<direction_category::Model>)>, DbErr>>;
}

impl DirectionQuery for DirectionAdapter<'_> {
    async fn get_all(self) -> Result<Vec<Model>, DbErr> {
        self.query.all(self.db).await
    }

    async fn get_by_id(self, id: Uuid) -> Result<Option<Model>, DbErr> {
        self.query.filter(Column::Id.eq(id)).one(self.db).await
    }

    async fn get_random(self) -> Result<Option<Model>, DbErr> {
        self.query
            .order_by(SimpleExpr::FunctionCall(Func::random()), Order::Asc)
            .limit(1)
            .one(self.db)
            .await
    }

    async fn get_random_with_category(
        self,
    ) -> Result<Option<(Model, Option<direction_category::Model>)>, DbErr> {
        self.query
            .select_also(direction_category::Entity)
            .order_by(SimpleExpr::FunctionCall(Func::random()), Order::Asc)
            .limit(1)
            .one(self.db)
            .await
    }
}

#[derive(Debug, Clone)]
pub struct CreateDirectionParams {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct UpdateDirectionParams {
    pub name: String,
    pub description: Option<String>,
    pub category_id: Option<Uuid>,
}

pub trait DirectionMutation {
    fn create_with_tag(
        self,
        params: CreateDirectionParams,
    ) -> impl Future<Output = Result<Model, TransactionError<DbErr>>>;
    fn update(
        self,
        direction: Model,
        params: UpdateDirectionParams,
    ) -> impl Future<Output = Result<Model, DbErr>>;
    fn archive(self, direction: Model) -> impl Future<Output = Result<Model, DbErr>>;
    fn unarchive(self, direction: Model) -> impl Future<Output = Result<Model, DbErr>>;
    fn bulk_update_ordering(
        self,
        directions: Vec<Model>,
        ordering: Vec<Uuid>,
    ) -> impl Future<Output = Result<(), DbErr>>;
    fn delete(self, direction: Model) -> impl Future<Output = Result<(), DbErr>>;
}

impl DirectionMutation for DirectionAdapter<'_> {
    async fn create_with_tag(
        self,
        params: CreateDirectionParams,
    ) -> Result<Model, TransactionError<DbErr>> {
        self.db
            .transaction::<_, Model, DbErr>(|txn| {
                Box::pin(async move {
                    let direction_id = uuid::Uuid::now_v7();
                    let created_direction = ActiveModel {
                        id: Set(direction_id),
                        user_id: Set(params.user_id),
                        name: Set(params.name.to_owned()),
                        description: Set(params.description.to_owned()),
                        category_id: Set(params.category_id),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await?;
                    tag::ActiveModel {
                        id: Set(uuid::Uuid::now_v7()),
                        user_id: Set(params.user_id),
                        direction_id: Set(Some(direction_id)),
                        r#type: Set(TagType::Direction),
                        ..Default::default()
                    }
                    .insert(txn)
                    .await?;

                    Ok(created_direction)
                })
            })
            .await
    }

    async fn update(
        self,
        direction: Model,
        params: UpdateDirectionParams,
    ) -> Result<Model, DbErr> {
        let mut direction = direction.into_active_model();
        direction.name = Set(params.name);
        direction.description = Set(params.description);
        direction.category_id = Set(params.category_id);
        direction.updated_at = Set(Utc::now().into());
        direction.update(self.db).await
    }

    async fn archive(self, direction: Model) -> Result<Model, DbErr> {
        let mut direction = direction.into_active_model();
        direction.archived = Set(true);
        direction.updated_at = Set(Utc::now().into());
        direction.update(self.db).await
    }

    async fn unarchive(self, direction: Model) -> Result<Model, DbErr> {
        let mut direction = direction.into_active_model();
        direction.archived = Set(false);
        direction.updated_at = Set(Utc::now().into());
        direction.update(self.db).await
    }

    async fn bulk_update_ordering(
        self,
        directions: Vec<Model>,
        ordering: Vec<Uuid>,
    ) -> Result<(), DbErr> {
        for direction in directions {
            let order = &ordering.iter().position(|id| id == &direction.id);
            if let Some(order) = order {
                let mut direction = direction.into_active_model();
                direction.ordering = Set(Some((order + 1) as i32));
                direction.update(self.db).await?;
            }
        }
        Ok(())
    }

    async fn delete(self, direction: Model) -> Result<(), DbErr> {
        direction.delete(self.db).await.map(|_| ())
    }
}
