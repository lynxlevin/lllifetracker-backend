use std::future::Future;

use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use sea_orm::{
    sea_query::NullOrdering::Last, sqlx::error::Error::Database, ActiveModelTrait, ColumnTrait,
    DbConn, DbErr, EntityTrait, FromQueryResult, IntoActiveModel, JoinType::LeftJoin, ModelTrait,
    Order, QueryFilter, QueryOrder, QuerySelect, RelationTrait, RuntimeErr::SqlxError, Select, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use entities::{
    action, ambition, desired_state,
    reading_note::{ActiveModel, Column, Entity, Model},
    reading_notes_tags,
    sea_orm_active_enums::TagType,
    tag, user,
};

use crate::{tag_adapter::TagWithNames, CustomDbErr};

#[derive(Clone)]
pub struct ReadingNoteAdapter<'a> {
    pub db: &'a DbConn,
    pub query: Select<Entity>,
}

impl<'a> ReadingNoteAdapter<'a> {
    pub fn init(db: &'a DbConn) -> Self {
        Self {
            db,
            query: Entity::find(),
        }
    }
}

pub trait ReadingNoteJoin {
    fn join_tags(self) -> Self;
    fn join_my_way_via_tags(self) -> Self;
}

impl ReadingNoteJoin for ReadingNoteAdapter<'_> {
    fn join_tags(mut self) -> Self {
        self.query = self
            .query
            .join_rev(LeftJoin, reading_notes_tags::Relation::ReadingNote.def())
            .join(LeftJoin, reading_notes_tags::Relation::Tag.def());
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

pub trait ReadingNoteFilter {
    fn filter_eq_id(self, id: Uuid) -> Self;
    fn filter_eq_user(self, user: &user::Model) -> Self;
}

impl ReadingNoteFilter for ReadingNoteAdapter<'_> {
    fn filter_eq_id(mut self, id: Uuid) -> Self {
        self.query = self.query.filter(Column::Id.eq(id));
        self
    }

    fn filter_eq_user(mut self, user: &user::Model) -> Self {
        self.query = self.query.filter(Column::UserId.eq(user.id));
        self
    }
}

pub trait ReadingNoteOrder {
    fn order_by_date(self, order: Order) -> Self;
    fn order_by_created_at(self, order: Order) -> Self;
    fn order_by_ambition_created_at_nulls_last(self, order: Order) -> Self;
    fn order_by_desired_state_created_at_nulls_last(self, order: Order) -> Self;
    fn order_by_action_created_at_nulls_last(self, order: Order) -> Self;
    fn order_by_tag_created_at_nulls_last(self, order: Order) -> Self;
}

impl ReadingNoteOrder for ReadingNoteAdapter<'_> {
    fn order_by_date(mut self, order: Order) -> Self {
        self.query = self.query.order_by(Column::Date, order);
        self
    }

    fn order_by_created_at(mut self, order: Order) -> Self {
        self.query = self.query.order_by(Column::CreatedAt, order);
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
pub struct ReadingNoteWithTag {
    pub id: Uuid,
    pub title: String,
    pub page_number: i16,
    pub text: String,
    pub date: NaiveDate,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub tag_id: Option<Uuid>,
    pub tag_name: Option<String>,
    pub tag_ambition_name: Option<String>,
    pub tag_desired_state_name: Option<String>,
    pub tag_action_name: Option<String>,
    pub tag_type: Option<TagType>,
    pub tag_created_at: Option<DateTime<FixedOffset>>,
}

impl Into<TagWithNames> for &ReadingNoteWithTag {
    /// Unsafe: panics if tag_id, tag_type, tag_created_at are None.
    fn into(self) -> TagWithNames {
        TagWithNames {
            id: self.tag_id.unwrap(),
            name: self.tag_name.clone(),
            ambition_name: self.tag_ambition_name.clone(),
            desired_state_name: self.tag_desired_state_name.clone(),
            action_name: self.tag_action_name.clone(),
            r#type: self.tag_type.clone().unwrap(),
            created_at: self.tag_created_at.unwrap(),
        }
    }
}

pub trait ReadingNoteQuery {
    fn get_all_with_tags(self) -> impl Future<Output = Result<Vec<ReadingNoteWithTag>, DbErr>>;
    fn get_by_id(self, id: Uuid) -> impl Future<Output = Result<Option<Model>, DbErr>>;
    fn get_with_tags(self)
        -> impl Future<Output = Result<Option<(Model, Vec<tag::Model>)>, DbErr>>;
}

impl ReadingNoteQuery for ReadingNoteAdapter<'_> {
    async fn get_all_with_tags(self) -> Result<Vec<ReadingNoteWithTag>, DbErr> {
        self.query
            .column_as(tag::Column::Id, "tag_id")
            .column_as(tag::Column::Name, "tag_name")
            .column_as(tag::Column::Type, "tag_type")
            .column_as(tag::Column::CreatedAt, "tag_created_at")
            .column_as(ambition::Column::Name, "tag_ambition_name")
            .column_as(desired_state::Column::Name, "tag_desired_state_name")
            .column_as(action::Column::Name, "tag_action_name")
            .into_model::<ReadingNoteWithTag>()
            .all(self.db)
            .await
    }

    async fn get_by_id(self, id: Uuid) -> Result<Option<Model>, DbErr> {
        self.query.filter(Column::Id.eq(id)).one(self.db).await
    }

    async fn get_with_tags(self) -> Result<Option<(Model, Vec<tag::Model>)>, DbErr> {
        match self.query.select_with(tag::Entity).all(self.db).await {
            Ok(reading_notes) => match reading_notes.len() > 0 {
                true => Ok(reading_notes.into_iter().nth(0)),
                false => Ok(None),
            },
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreateReadingNoteParams {
    pub title: String,
    pub page_number: i16,
    pub text: String,
    pub date: NaiveDate,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct UpdateReadingNoteParams {
    pub title: Option<String>,
    pub page_number: Option<i16>,
    pub text: Option<String>,
    pub date: Option<NaiveDate>,
}

pub trait ReadingNoteMutation {
    fn create(self, params: CreateReadingNoteParams) -> impl Future<Output = Result<Model, DbErr>>;
    fn partial_update(
        self,
        reading_note: Model,
        params: UpdateReadingNoteParams,
    ) -> impl Future<Output = Result<Model, DbErr>>;
    fn delete(self, reading_note: Model) -> impl Future<Output = Result<(), DbErr>>;
    fn link_tags(
        &self,
        reading_note: &Model,
        tag_ids: impl IntoIterator<Item = Uuid>,
    ) -> impl Future<Output = Result<(), DbErr>>;
    fn unlink_tags(
        &self,
        reading_note: &Model,
        tag_ids: impl IntoIterator<Item = Uuid>,
    ) -> impl Future<Output = Result<(), DbErr>>;
}

impl ReadingNoteMutation for ReadingNoteAdapter<'_> {
    async fn create(self, params: CreateReadingNoteParams) -> Result<Model, DbErr> {
        let now = Utc::now();
        ActiveModel {
            id: Set(Uuid::now_v7()),
            user_id: Set(params.user_id),
            title: Set(params.title.to_owned()),
            page_number: Set(params.page_number),
            text: Set(params.text.to_owned()),
            date: Set(params.date),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        }
        .insert(self.db)
        .await
    }

    async fn partial_update(
        self,
        reading_note: Model,
        params: UpdateReadingNoteParams,
    ) -> Result<Model, DbErr> {
        let mut reading_note = reading_note.into_active_model();
        if let Some(title) = params.title {
            reading_note.title = Set(title);
        }
        if let Some(page_number) = params.page_number {
            reading_note.page_number = Set(page_number);
        }
        if let Some(text) = params.text {
            reading_note.text = Set(text);
        }
        if let Some(date) = params.date {
            reading_note.date = Set(date);
        }
        reading_note.updated_at = Set(Utc::now().into());
        reading_note.update(self.db).await
    }

    async fn delete(self, reading_note: Model) -> Result<(), DbErr> {
        reading_note.delete(self.db).await.map(|_| ())
    }

    async fn link_tags(
        &self,
        reading_note: &Model,
        tag_ids: impl IntoIterator<Item = Uuid>,
    ) -> Result<(), DbErr> {
        let tag_links = tag_ids
            .into_iter()
            .map(|tag_id| reading_notes_tags::ActiveModel {
                reading_note_id: Set(reading_note.id),
                tag_id: Set(tag_id),
            });
        reading_notes_tags::Entity::insert_many(tag_links)
            .on_empty_do_nothing()
            .exec(self.db)
            .await
            .map(|_| ())
            .map_err(|e| match &e {
                DbErr::Exec(SqlxError(Database(err))) => match err.constraint() {
                    Some("fk-book_excerpts_tags-tag_id") => {
                        DbErr::Custom(CustomDbErr::NotFound.to_string())
                    }
                    _ => e,
                },
                _ => e,
            })
    }

    async fn unlink_tags(
        &self,
        reading_note: &Model,
        tag_ids: impl IntoIterator<Item = Uuid>,
    ) -> Result<(), DbErr> {
        reading_notes_tags::Entity::delete_many()
            .filter(reading_notes_tags::Column::ReadingNoteId.eq(reading_note.id))
            .filter(reading_notes_tags::Column::TagId.is_in(tag_ids))
            .exec(self.db)
            .await
            .map(|_| ())
    }
}
