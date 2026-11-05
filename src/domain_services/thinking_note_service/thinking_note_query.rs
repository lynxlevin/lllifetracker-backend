use std::future::Future;

use chrono::{DateTime, FixedOffset};
use entities::{
    action, ambition, direction,
    tag::{self, TagType},
    thinking_note::{Column, Entity},
    thinking_note_tags,
};
use sea_orm::{
    prelude::Expr, sea_query::NullOrdering::Last, ColumnAsExpr, ColumnTrait, Condition, EntityTrait, ExprTrait,
    FromQueryResult, JoinType::LeftJoin, Order::Asc, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};
use serde::Serialize;
use uuid::Uuid;

use crate::thinking_note_service::{ThinkingNoteService, ThinkingNoteServiceError};

#[derive(FromQueryResult, Debug, Serialize, PartialEq, Clone)]
pub struct ThinkingNoteWithTag {
    pub id: Uuid,
    pub question: Option<String>,
    pub thought: Option<String>,
    pub answer: Option<String>,
    pub resolved_at: Option<DateTime<FixedOffset>>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub tag_id: Option<Uuid>,
    pub tag_name: Option<String>,
    pub tag_type: Option<TagType>,
    pub tag_created_at: Option<DateTime<FixedOffset>>,
}
pub trait ThinkingNoteServiceQuery {
    fn search(
        &self,
        user_id: Uuid,
        text_query: Vec<String>,
        tag_ids: Vec<Uuid>,
    ) -> impl Future<Output = Result<Vec<ThinkingNoteWithTag>, ThinkingNoteServiceError>>;
}

impl ThinkingNoteServiceQuery for ThinkingNoteService<'_> {
    async fn search(
        &self,
        user_id: Uuid,
        text_query: Vec<String>,
        tag_ids: Vec<Uuid>,
    ) -> Result<Vec<ThinkingNoteWithTag>, ThinkingNoteServiceError> {
        let mut thinking_note_ids_query = Entity::find()
            .join_rev(LeftJoin, thinking_note_tags::Relation::ThinkingNote.def())
            .select_only()
            .column(Column::Id)
            .filter(Column::UserId.eq(user_id));
        if text_query.len() > 0 {
            let mut text_cond = Condition::all();
            for text in text_query {
                text_cond = text_cond.add(
                    Condition::any()
                        .add(Column::Question.contains(&text))
                        .add(Column::Thought.contains(&text))
                        .add(Column::Answer.contains(&text)),
                );
            }
            thinking_note_ids_query = thinking_note_ids_query.filter(text_cond)
        }
        if tag_ids.len() > 0 {
            thinking_note_ids_query =
                thinking_note_ids_query.filter(thinking_note_tags::Column::TagId.is_in(tag_ids))
        }
        let thinking_note_ids: Vec<Uuid> = thinking_note_ids_query.into_tuple().all(self.db).await?;

        let thinking_notes = Entity::find()
            .join_rev(LeftJoin, thinking_note_tags::Relation::ThinkingNote.def())
            .join(LeftJoin, thinking_note_tags::Relation::Tag.def())
            .join(LeftJoin, tag::Relation::Ambition.def())
            .join(LeftJoin, tag::Relation::Direction.def())
            .join(LeftJoin, tag::Relation::Action.def())
            .filter(Column::Id.is_in(thinking_note_ids))
            .order_by_desc(Column::ResolvedAt)
            .order_by_desc(Column::UpdatedAt)
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
            .into_model::<ThinkingNoteWithTag>()
            .all(self.db)
            .await?;

        Ok(thinking_notes)
    }
}
