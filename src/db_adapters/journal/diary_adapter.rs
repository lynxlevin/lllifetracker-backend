use std::future::Future;

use chrono::{DateTime, FixedOffset, NaiveDate};
use sea_orm::{
    sea_query::NullOrdering::Last, sqlx::error::Error::Database, ActiveModelTrait, ColumnTrait,
    DbConn, DbErr, EntityTrait, FromQueryResult, IntoActiveModel, JoinType::LeftJoin, ModelTrait,
    Order, QueryFilter, QueryOrder, QuerySelect, RelationTrait, RuntimeErr::SqlxError, Select, Set,
    TransactionError, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use entities::{
    action, ambition, desired_state, diaries_tags,
    diary::{ActiveModel, Column, Entity, Model},
    tag, user,
};

use crate::CustomDbErr;

#[derive(Clone)]
pub struct DiaryAdapter<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> DiaryAdapter<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self {
            db,
            query: Entity::find(),
        }
    }
}

pub trait DiaryJoin {
    fn join_diary_with_my_ways(self) -> Self;
}

impl DiaryJoin for DiaryAdapter<'_> {
    fn join_diary_with_my_ways(mut self) -> Self {
        self.query = self
            .query
            .join_rev(LeftJoin, diaries_tags::Relation::Diary.def())
            .join(LeftJoin, diaries_tags::Relation::Tag.def())
            .join(LeftJoin, tag::Relation::Ambition.def())
            .join(LeftJoin, tag::Relation::DesiredState.def())
            .join(LeftJoin, tag::Relation::Action.def());
        self
    }
}

pub trait DiaryFilter {
    fn filter_eq_user(self, user: &user::Model) -> Self;
}

impl DiaryFilter for DiaryAdapter<'_> {
    fn filter_eq_user(mut self, user: &user::Model) -> Self {
        self.query = self.query.filter(Column::UserId.eq(user.id));
        self
    }
}

pub trait DiaryOrder {
    fn order_by_date(self, order: Order) -> Self;
    fn order_by_ambition_created_at_nulls_last(self, order: Order) -> Self;
    fn order_by_desired_state_created_at_nulls_last(self, order: Order) -> Self;
    fn order_by_action_created_at_nulls_last(self, order: Order) -> Self;
    fn order_by_tag_created_at_nulls_last(self, order: Order) -> Self;
}

impl DiaryOrder for DiaryAdapter<'_> {
    fn order_by_date(mut self, order: Order) -> Self {
        self.query = self.query.order_by(Column::Date, order);
        self
    }

    fn order_by_ambition_created_at_nulls_last(mut self, order: Order) -> Self {
        self.query = self
            .query
            .order_by_with_nulls(ambition::Column::CreatedAt, order, Last);
        self
    }

    fn order_by_desired_state_created_at_nulls_last(mut self, order: Order) -> Self {
        self.query = self
            .query
            .order_by_with_nulls(desired_state::Column::CreatedAt, order, Last);
        self
    }

    fn order_by_action_created_at_nulls_last(mut self, order: Order) -> Self {
        self.query = self
            .query
            .order_by_with_nulls(action::Column::CreatedAt, order, Last);
        self
    }

    fn order_by_tag_created_at_nulls_last(mut self, order: Order) -> Self {
        self.query = self
            .query
            .order_by_with_nulls(tag::Column::CreatedAt, order, Last);
        self
    }
}

#[derive(FromQueryResult, Debug, Serialize, Deserialize, PartialEq)]
pub struct DiaryWithTag {
    pub id: Uuid,
    pub text: Option<String>,
    pub date: NaiveDate,
    pub score: Option<i16>,
    pub tag_id: Option<Uuid>,
    pub tag_name: Option<String>,
    pub tag_ambition_name: Option<String>,
    pub tag_desired_state_name: Option<String>,
    pub tag_action_name: Option<String>,
    pub tag_created_at: Option<DateTime<FixedOffset>>,
}

pub trait DiaryQuery {
    fn get_all_with_tag(self) -> impl Future<Output = Result<Vec<DiaryWithTag>, DbErr>>;
    fn get_by_id(self, id: Uuid) -> impl Future<Output = Result<Option<Model>, DbErr>>;
}

impl DiaryQuery for DiaryAdapter<'_> {
    async fn get_all_with_tag(self) -> Result<Vec<DiaryWithTag>, DbErr> {
        self.query
            .column_as(tag::Column::Id, "tag_id")
            .column_as(tag::Column::Name, "tag_name")
            .column_as(tag::Column::CreatedAt, "tag_created_at")
            .column_as(ambition::Column::Name, "tag_ambition_name")
            .column_as(desired_state::Column::Name, "tag_desired_state_name")
            .column_as(action::Column::Name, "tag_action_name")
            .into_model::<DiaryWithTag>()
            .all(self.db)
            .await
    }

    async fn get_by_id(self, id: Uuid) -> Result<Option<Model>, DbErr> {
        self.query.filter(Column::Id.eq(id)).one(self.db).await
    }
}

#[derive(Debug, Clone)]
pub struct CreateDiaryParams {
    pub text: Option<String>,
    pub date: NaiveDate,
    pub score: Option<i16>,
    pub tag_ids: Vec<Uuid>,
    pub user_id: Uuid,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum DiaryUpdateKey {
    Text,
    Date,
    Score,
    TagIds,
}

#[derive(Debug, Clone)]
pub struct UpdateDiaryParams {
    pub text: Option<String>,
    pub date: NaiveDate,
    pub score: Option<i16>,
    pub tag_ids: Vec<Uuid>,
    pub update_keys: Vec<DiaryUpdateKey>,
}

pub trait DiaryMutation {
    fn create(
        self,
        params: CreateDiaryParams,
    ) -> impl Future<Output = Result<Model, TransactionError<DbErr>>>;
    fn partial_update(
        self,
        diary: Model,
        params: UpdateDiaryParams,
    ) -> impl Future<Output = Result<Model, TransactionError<DbErr>>>;
    fn delete(self, diary: Model) -> impl Future<Output = Result<(), DbErr>>;
}

impl DiaryMutation for DiaryAdapter<'_> {
    async fn create(self, params: CreateDiaryParams) -> Result<Model, TransactionError<DbErr>> {
        self.db
            .transaction::<_, Model, DbErr>(|txn| {
                Box::pin(async move {
                    let diary_id = Uuid::now_v7();
                    let diary = ActiveModel {
                        id: Set(diary_id),
                        user_id: Set(params.user_id),
                        text: Set(params.text),
                        date: Set(params.date),
                        score: Set(params.score),
                    }
                    .insert(txn)
                    .await
                    .map_err(|e| match &e {
                        DbErr::Query(SqlxError(Database(err))) => match err.constraint() {
                            Some("diaries_user_id_date_unique_index") => {
                                DbErr::Custom(CustomDbErr::Duplicate.to_string())
                            }
                            _ => e,
                        },
                        _ => e,
                    })?;

                    let tag_links_to_create = params
                        .tag_ids
                        .clone()
                        .into_iter()
                        .map(|tag_id| diaries_tags::ActiveModel {
                            diary_id: Set(diary.id),
                            tag_id: Set(tag_id),
                        })
                        .collect::<Vec<_>>();
                    diaries_tags::Entity::insert_many(tag_links_to_create)
                        .on_empty_do_nothing()
                        .exec(txn)
                        .await
                        .map_err(|e| match &e {
                            DbErr::Exec(SqlxError(Database(err))) => match err.constraint() {
                                Some("fk-diaries_tags-tag_id") => {
                                    DbErr::Custom(CustomDbErr::NotFound.to_string())
                                }
                                _ => e,
                            },
                            _ => e,
                        })?;

                    Ok(diary)
                })
            })
            .await
    }

    async fn partial_update(
        self,
        diary: Model,
        params: UpdateDiaryParams,
    ) -> Result<Model, TransactionError<DbErr>> {
        self.db
            .transaction::<_, Model, DbErr>(|txn| {
                Box::pin(async move {
                    let diary_id = diary.id;
                    let mut diary = diary.into_active_model();
                    if params.update_keys.contains(&DiaryUpdateKey::Text) {
                        diary.text = Set(params.text);
                    }
                    if params.update_keys.contains(&DiaryUpdateKey::Date) {
                        diary.date = Set(params.date);
                    }
                    if params.update_keys.contains(&DiaryUpdateKey::Score) {
                        diary.score = Set(params.score);
                    }
                    if params.update_keys.contains(&DiaryUpdateKey::TagIds) {
                        let tag_ids = params.tag_ids;
                        let tag_links = diaries_tags::Entity::find()
                            .filter(diaries_tags::Column::DiaryId.eq(diary_id))
                            .all(txn)
                            .await?;
                        let linked_tag_ids = tag_links
                            .into_iter()
                            .map(|link| link.tag_id)
                            .collect::<Vec<_>>();

                        let tag_links_to_create: Vec<diaries_tags::ActiveModel> = tag_ids
                            .clone()
                            .into_iter()
                            .filter(|id| !linked_tag_ids.contains(id))
                            .map(|tag_id| diaries_tags::ActiveModel {
                                diary_id: Set(diary_id),
                                tag_id: Set(tag_id),
                            })
                            .collect();
                        diaries_tags::Entity::insert_many(tag_links_to_create)
                            .on_empty_do_nothing()
                            .exec(txn)
                            .await
                            .map_err(|e| match &e {
                                DbErr::Exec(SqlxError(Database(err))) => match err.constraint() {
                                    Some("fk-diaries_tags-tag_id") => {
                                        DbErr::Custom(CustomDbErr::NotFound.to_string())
                                    }
                                    _ => e,
                                },
                                _ => e,
                            })?;

                        let ids_to_delete: Vec<uuid::Uuid> = linked_tag_ids
                            .into_iter()
                            .filter(|linked_tag_id| !tag_ids.contains(linked_tag_id))
                            .collect();
                        if ids_to_delete.len() > 0 {
                            diaries_tags::Entity::delete_many()
                                .filter(diaries_tags::Column::DiaryId.eq(diary_id))
                                .filter(diaries_tags::Column::TagId.is_in(ids_to_delete))
                                .exec(txn)
                                .await?;
                        }
                    }
                    diary.update(txn).await.map_err(|e| match &e {
                        DbErr::Query(SqlxError(Database(err))) => match err.constraint() {
                            Some("diaries_user_id_date_unique_index") => {
                                DbErr::Custom(CustomDbErr::Duplicate.to_string())
                            }
                            _ => e,
                        },
                        _ => e,
                    })
                })
            })
            .await
    }

    async fn delete(self, diary: Model) -> Result<(), DbErr> {
        diary.delete(self.db).await.map(|_| ())
    }
}
