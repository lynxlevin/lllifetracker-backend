use std::future::Future;

use sea_orm::{
    prelude::Expr,
    sea_query::NullOrdering::{First, Last},
    ActiveModelTrait, ColumnAsExpr, ColumnTrait, DbConn, DbErr, EntityTrait, FromQueryResult,
    IntoActiveModel,
    JoinType::LeftJoin,
    ModelTrait, Order, QueryFilter, QueryOrder, QuerySelect, RelationTrait, Select, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use entities::{
    action, ambition, direction,
    sea_orm_active_enums::TagType,
    tag::{ActiveModel, Column, Entity, Model, Relation},
    user,
};

#[derive(Clone)]
pub struct TagAdapter<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> TagAdapter<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self {
            db,
            query: Entity::find(),
        }
    }
}

pub trait TagJoin {
    fn join_ambition(self) -> Self;
    fn join_direction(self) -> Self;
    fn join_action(self) -> Self;
}

impl TagJoin for TagAdapter<'_> {
    fn join_ambition(mut self) -> Self {
        self.query = self.query.join(LeftJoin, Relation::Ambition.def());
        self
    }

    fn join_direction(mut self) -> Self {
        self.query = self.query.join(LeftJoin, Relation::Direction.def());
        self
    }

    fn join_action(mut self) -> Self {
        self.query = self.query.join(LeftJoin, Relation::Action.def());
        self
    }
}

pub trait TagFilter {
    fn filter_eq_user(self, user: &user::Model) -> Self;
}

impl TagFilter for TagAdapter<'_> {
    fn filter_eq_user(mut self, user: &user::Model) -> Self {
        self.query = self.query.filter(Column::UserId.eq(user.id));
        self
    }
}

pub trait TagOrder {
    fn order_by_ambition_ordering_nulls_last(self, order: Order) -> Self;
    fn order_by_direction_ordering_nulls_first(self, order: Order) -> Self;
    fn order_by_action_ordering_nulls_last(self, order: Order) -> Self;
    fn order_by_created_at(self, order: Order) -> Self;
    fn order_by_type(self, order: Order) -> Self;
}

impl TagOrder for TagAdapter<'_> {
    fn order_by_ambition_ordering_nulls_last(mut self, order: Order) -> Self {
        self.query = self
            .query
            .order_by_with_nulls(ambition::Column::Ordering, order, Last);
        self
    }

    fn order_by_direction_ordering_nulls_first(mut self, order: Order) -> Self {
        self.query = self
            .query
            .order_by_with_nulls(direction::Column::Ordering, order, First);
        self
    }

    fn order_by_action_ordering_nulls_last(mut self, order: Order) -> Self {
        self.query = self
            .query
            .order_by_with_nulls(action::Column::Ordering, order, Last);
        self
    }

    fn order_by_created_at(mut self, order: Order) -> Self {
        self.query = self.query.order_by(Column::CreatedAt, order);
        self
    }

    fn order_by_type(mut self, order: Order) -> Self {
        self.query = self.query.order_by(Column::Type, order);
        self
    }
}

#[derive(FromQueryResult, Debug, Serialize, Deserialize, PartialEq)]
pub struct TagWithName {
    pub id: uuid::Uuid,
    pub name: String,
    pub r#type: TagType,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

pub trait TagQuery {
    fn get_all_tags(self) -> impl Future<Output = Result<Vec<TagWithName>, DbErr>>;
    fn get_by_id(self, id: Uuid) -> impl Future<Output = Result<Option<Model>, DbErr>>;
}

impl TagQuery for TagAdapter<'_> {
    async fn get_all_tags(self) -> Result<Vec<TagWithName>, DbErr> {
        self.query
            .expr_as(
                Expr::case(
                    Expr::col(Column::Type)
                        .cast_as("text")
                        .eq(TagType::Ambition),
                    ambition::Column::Name.into_column_as_expr(),
                )
                .case(
                    Expr::col(Column::Type)
                        .cast_as("text")
                        .eq(TagType::Direction),
                    direction::Column::Name.into_column_as_expr(),
                )
                .case(
                    Expr::col(Column::Type).cast_as("text").eq(TagType::Action),
                    action::Column::Name.into_column_as_expr(),
                )
                .case(
                    Expr::col(Column::Type).cast_as("text").eq(TagType::Plain),
                    Column::Name.into_column_as_expr(),
                )
                .finally("no_name"),
                "name",
            )
            .into_model::<TagWithName>()
            .all(self.db)
            .await
    }

    async fn get_by_id(self, id: Uuid) -> Result<Option<Model>, DbErr> {
        self.query.filter(Column::Id.eq(id)).one(self.db).await
    }
}

#[derive(Debug, Clone)]
pub struct CreatePlainTagParams {
    pub name: String,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct UpdatePlainTagParams {
    pub name: String,
}

pub trait TagMutation {
    fn create_plain(
        self,
        params: CreatePlainTagParams,
    ) -> impl Future<Output = Result<Model, DbErr>>;
    fn update_plain(
        self,
        tag: Model,
        params: UpdatePlainTagParams,
    ) -> impl Future<Output = Result<Model, DbErr>>;
    fn delete(self, tag: Model) -> impl Future<Output = Result<(), DbErr>>;
}

impl TagMutation for TagAdapter<'_> {
    async fn create_plain(self, params: CreatePlainTagParams) -> Result<Model, DbErr> {
        ActiveModel {
            id: Set(Uuid::now_v7()),
            user_id: Set(params.user_id),
            name: Set(Some(params.name.to_owned())),
            ..Default::default()
        }
        .insert(self.db)
        .await
    }

    async fn update_plain(self, tag: Model, params: UpdatePlainTagParams) -> Result<Model, DbErr> {
        let mut tag = tag.into_active_model();
        tag.name = Set(Some(params.name));
        tag.update(self.db).await
    }

    async fn delete(self, tag: Model) -> Result<(), DbErr> {
        tag.delete(self.db).await.map(|_| ())
    }
}
