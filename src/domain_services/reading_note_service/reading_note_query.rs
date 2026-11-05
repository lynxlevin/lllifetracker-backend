use std::future::Future;

use chrono::{DateTime, FixedOffset, NaiveDate};
use entities::{
    action, ambition, direction,
    reading_note::{Column, Entity},
    reading_notes_tags,
    tag::{self, TagType},
};
use sea_orm::{
    prelude::Expr, sea_query::NullOrdering::Last, ColumnAsExpr, ColumnTrait, Condition, EntityTrait, ExprTrait,
    FromQueryResult, JoinType::LeftJoin, Order::Asc, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use serde::Serialize;
use uuid::Uuid;

use crate::reading_note_service::{ReadingNoteService, ReadingNoteServiceError};

#[derive(FromQueryResult, Debug, Serialize, PartialEq, Clone)]
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
    pub tag_type: Option<TagType>,
    pub tag_created_at: Option<DateTime<FixedOffset>>,
}

pub trait ReadingNoteServiceQuery {
    fn search(
        &self,
        user_id: Uuid,
        text_query: Vec<String>,
        tag_ids: Vec<Uuid>,
    ) -> impl Future<Output = Result<Vec<ReadingNoteWithTag>, ReadingNoteServiceError>>;
}

impl ReadingNoteServiceQuery for ReadingNoteService<'_> {
    async fn search(
        &self,
        user_id: Uuid,
        text_query: Vec<String>,
        tag_ids: Vec<Uuid>,
    ) -> Result<Vec<ReadingNoteWithTag>, ReadingNoteServiceError> {
        let mut reading_note_ids_query = Entity::find()
            .join_rev(LeftJoin, reading_notes_tags::Relation::ReadingNote.def())
            .select_only()
            .column(Column::Id)
            .filter(Column::UserId.eq(user_id));
        if text_query.len() > 0 {
            let mut text_cond = Condition::all();
            for text in text_query {
                text_cond = text_cond.add(
                    Condition::any()
                        .add(Column::Title.contains(&text))
                        .add(Column::Text.contains(&text)),
                );
            }
            reading_note_ids_query = reading_note_ids_query.filter(text_cond)
        }
        if tag_ids.len() > 0 {
            reading_note_ids_query =
                reading_note_ids_query.filter(reading_notes_tags::Column::TagId.is_in(tag_ids))
        }
        let reading_note_ids: Vec<Uuid> = reading_note_ids_query.into_tuple().all(self.db).await?;

        let reading_notes = Entity::find()
            .join_rev(LeftJoin, reading_notes_tags::Relation::ReadingNote.def())
            .join(LeftJoin, reading_notes_tags::Relation::Tag.def())
            .join(LeftJoin, tag::Relation::Ambition.def())
            .join(LeftJoin, tag::Relation::Direction.def())
            .join(LeftJoin, tag::Relation::Action.def())
            .filter(Column::Id.is_in(reading_note_ids))
            .order_by_desc(Column::Date)
            .order_by_desc(Column::CreatedAt)
            .order_by_with_nulls(ambition::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(direction::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(action::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(tag::Column::CreatedAt, Asc, Last)
            .column_as(tag::Column::Id, "tag_id")
            .expr_as(
                Expr::case(
                    Expr::col(tag::Column::Type).cast_as("text").eq(TagType::Ambition),
                    ambition::Column::Name.into_column_as_expr(),
                )
                .case(
                    Expr::col(tag::Column::Type).cast_as("text").eq(TagType::Direction),
                    direction::Column::Name.into_column_as_expr(),
                )
                .case(
                    Expr::col(tag::Column::Type).cast_as("text").eq(TagType::Action),
                    action::Column::Name.into_column_as_expr(),
                )
                .case(
                    Expr::col(tag::Column::Type).cast_as("text").eq(TagType::Plain),
                    tag::Column::Name.into_column_as_expr(),
                )
                .finally("no_name"),
                "tag_name",
            )
            .column_as(tag::Column::Type, "tag_type")
            .column_as(tag::Column::CreatedAt, "tag_created_at")
            .into_model::<ReadingNoteWithTag>()
            .all(self.db)
            .await?;

        Ok(reading_notes)
    }
}
