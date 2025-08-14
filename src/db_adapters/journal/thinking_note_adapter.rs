use std::future::Future;

use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{
    sea_query::NullOrdering::Last, sqlx::error::Error::Database, ActiveModelTrait, ColumnTrait,
    DbConn, DbErr, EntityTrait, FromQueryResult, IntoActiveModel, JoinType::LeftJoin, ModelTrait,
    Order, QueryFilter, QueryOrder, QuerySelect, RelationTrait, RuntimeErr::SqlxError, Select, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use entities::{
    action, ambition, desired_state, tag,
    thinking_note::{ActiveModel, Column, Entity, Model},
    thinking_note_tags, user,
};

use crate::CustomDbErr;

#[derive(Clone)]
pub struct ThinkingNoteAdapter<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> ThinkingNoteAdapter<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self {
            db,
            query: Entity::find(),
        }
    }
}

pub trait ThinkingNoteJoin {
    fn join_tags(self) -> Self;
    fn join_my_way_via_tags(self) -> Self;
}

impl ThinkingNoteJoin for ThinkingNoteAdapter<'_> {
    fn join_tags(mut self) -> Self {
        self.query = self
            .query
            .join_rev(LeftJoin, thinking_note_tags::Relation::ThinkingNote.def())
            .join(LeftJoin, thinking_note_tags::Relation::Tag.def());
        self
    }
    fn join_my_way_via_tags(mut self) -> Self {
        self.query = self
            .query
            .join(LeftJoin, tag::Relation::Ambition.def())
            .join(LeftJoin, tag::Relation::DesiredState.def())
            .join(LeftJoin, tag::Relation::Action.def());
        self
    }
}

pub trait ThinkingNoteFilter {
    fn filter_eq_id(self, id: Uuid) -> Self;
    fn filter_eq_user(self, user: &user::Model) -> Self;
    fn filter_null_resolved_at(self, is_null: bool) -> Self;
    fn filter_null_archived_at(self, is_null: bool) -> Self;
}

impl ThinkingNoteFilter for ThinkingNoteAdapter<'_> {
    fn filter_eq_id(mut self, id: Uuid) -> Self {
        self.query = self.query.filter(Column::Id.eq(id));
        self
    }

    fn filter_eq_user(mut self, user: &user::Model) -> Self {
        self.query = self.query.filter(Column::UserId.eq(user.id));
        self
    }

    fn filter_null_resolved_at(mut self, is_null: bool) -> Self {
        self.query = match is_null {
            true => self.query.filter(Column::ResolvedAt.is_null()),
            false => self.query.filter(Column::ResolvedAt.is_not_null()),
        };
        self
    }

    fn filter_null_archived_at(mut self, is_null: bool) -> Self {
        self.query = match is_null {
            true => self.query.filter(Column::ArchivedAt.is_null()),
            false => self.query.filter(Column::ArchivedAt.is_not_null()),
        };
        self
    }
}

pub trait ThinkingNoteOrder {
    fn order_by_updated_at(self, order: Order) -> Self;
    fn order_by_ambition_created_at_nulls_last(self, order: Order) -> Self;
    fn order_by_desired_state_created_at_nulls_last(self, order: Order) -> Self;
    fn order_by_action_created_at_nulls_last(self, order: Order) -> Self;
    fn order_by_tag_created_at_nulls_last(self, order: Order) -> Self;
}

impl ThinkingNoteOrder for ThinkingNoteAdapter<'_> {
    fn order_by_updated_at(mut self, order: Order) -> Self {
        self.query = self.query.order_by(Column::UpdatedAt, order);
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
pub struct ThinkingNoteWithTag {
    pub id: Uuid,
    pub question: Option<String>,
    pub thought: Option<String>,
    pub answer: Option<String>,
    pub resolved_at: Option<DateTime<FixedOffset>>,
    pub archived_at: Option<DateTime<FixedOffset>>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub tag_id: Option<Uuid>,
    pub tag_name: Option<String>,
    pub tag_ambition_name: Option<String>,
    pub tag_desired_state_name: Option<String>,
    pub tag_action_name: Option<String>,
    pub tag_created_at: Option<DateTime<FixedOffset>>,
}

pub trait ThinkingNoteQuery {
    fn get_all_with_tags(self) -> impl Future<Output = Result<Vec<ThinkingNoteWithTag>, DbErr>>;
    fn get_by_id(self, id: Uuid) -> impl Future<Output = Result<Option<Model>, DbErr>>;
    fn get_with_tags(self)
        -> impl Future<Output = Result<Option<(Model, Vec<tag::Model>)>, DbErr>>;
}

impl ThinkingNoteQuery for ThinkingNoteAdapter<'_> {
    async fn get_all_with_tags(self) -> Result<Vec<ThinkingNoteWithTag>, DbErr> {
        self.query
            .column_as(tag::Column::Id, "tag_id")
            .column_as(tag::Column::Name, "tag_name")
            .column_as(tag::Column::CreatedAt, "tag_created_at")
            .column_as(ambition::Column::Name, "tag_ambition_name")
            .column_as(desired_state::Column::Name, "tag_desired_state_name")
            .column_as(action::Column::Name, "tag_action_name")
            .into_model::<ThinkingNoteWithTag>()
            .all(self.db)
            .await
    }

    async fn get_by_id(self, id: Uuid) -> Result<Option<Model>, DbErr> {
        self.query.filter(Column::Id.eq(id)).one(self.db).await
    }

    async fn get_with_tags(self) -> Result<Option<(Model, Vec<tag::Model>)>, DbErr> {
        match self.query.select_with(tag::Entity).all(self.db).await {
            Ok(thinking_notes) => match thinking_notes.len() > 0 {
                true => Ok(thinking_notes.into_iter().nth(0)),
                false => Ok(None),
            },
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreateThinkingNoteParams {
    pub question: Option<String>,
    pub thought: Option<String>,
    pub answer: Option<String>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct UpdateThinkingNoteParams {
    pub question: Option<String>,
    pub thought: Option<String>,
    pub answer: Option<String>,
    pub resolved_at: Option<DateTime<FixedOffset>>,
    pub archived_at: Option<DateTime<FixedOffset>>,
}

pub trait ThinkingNoteMutation {
    fn create(self, params: CreateThinkingNoteParams)
        -> impl Future<Output = Result<Model, DbErr>>;
    fn update(
        self,
        params: UpdateThinkingNoteParams,
        thinking_note: Model,
    ) -> impl Future<Output = Result<Model, DbErr>>;
    fn delete(self, thinking_note: Model) -> impl Future<Output = Result<(), DbErr>>;
    fn link_tags(
        &self,
        thinking_note: &Model,
        tag_ids: impl IntoIterator<Item = Uuid>,
    ) -> impl Future<Output = Result<(), DbErr>>;
    fn unlink_tags(
        &self,
        thinking_note: &Model,
        tag_ids: impl IntoIterator<Item = Uuid>,
    ) -> impl Future<Output = Result<(), DbErr>>;
}

impl ThinkingNoteMutation for ThinkingNoteAdapter<'_> {
    async fn create(self, params: CreateThinkingNoteParams) -> Result<Model, DbErr> {
        let now = Utc::now();
        ActiveModel {
            id: Set(Uuid::now_v7()),
            user_id: Set(params.user_id),
            question: Set(params.question),
            thought: Set(params.thought),
            answer: Set(params.answer),
            resolved_at: Set(None),
            archived_at: Set(None),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        }
        .insert(self.db)
        .await
    }

    async fn update(
        self,
        params: UpdateThinkingNoteParams,
        thinking_note: Model,
    ) -> Result<Model, DbErr> {
        let mut thinking_note = thinking_note.into_active_model();
        thinking_note.question = Set(params.question);
        thinking_note.thought = Set(params.thought);
        thinking_note.answer = Set(params.answer);
        thinking_note.resolved_at = Set(params.resolved_at);
        thinking_note.archived_at = Set(params.archived_at);
        thinking_note.updated_at = Set(Utc::now().into());
        thinking_note.update(self.db).await
    }

    async fn delete(self, thinking_note: Model) -> Result<(), DbErr> {
        thinking_note.delete(self.db).await.map(|_| ())
    }

    async fn link_tags(
        &self,
        thinking_note: &Model,
        tag_ids: impl IntoIterator<Item = Uuid>,
    ) -> Result<(), DbErr> {
        let tag_links = tag_ids
            .into_iter()
            .map(|tag_id| thinking_note_tags::ActiveModel {
                thinking_note_id: Set(thinking_note.id),
                tag_id: Set(tag_id),
            });
        thinking_note_tags::Entity::insert_many(tag_links)
            .on_empty_do_nothing()
            .exec(self.db)
            .await
            .map(|_| ())
            .map_err(|e| match &e {
                DbErr::Exec(SqlxError(Database(err))) => match err.constraint() {
                    Some("fk-thinking_note_tags-tag_id") => {
                        DbErr::Custom(CustomDbErr::NotFound.to_string())
                    }
                    _ => e,
                },
                _ => e,
            })
    }

    async fn unlink_tags(
        &self,
        thinking_note: &Model,
        tag_ids: impl IntoIterator<Item = Uuid>,
    ) -> Result<(), DbErr> {
        thinking_note_tags::Entity::delete_many()
            .filter(thinking_note_tags::Column::ThinkingNoteId.eq(thinking_note.id))
            .filter(thinking_note_tags::Column::TagId.is_in(tag_ids))
            .exec(self.db)
            .await
            .map(|_| ())
    }
}
