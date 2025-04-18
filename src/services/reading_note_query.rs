use ::types::{CustomDbErr, ReadingNoteWithTagQueryResult};
use entities::{action, ambition, desired_state, reading_note, reading_notes_tags, tag};
use sea_orm::{
    sea_query::NullOrdering::Last, ColumnTrait, DbConn, DbErr, EntityTrait,
    JoinType::LeftJoin, Order::Asc, QueryFilter, QueryOrder, QuerySelect, RelationTrait,
};

pub struct ReadingNoteQuery;

impl ReadingNoteQuery {
    pub async fn find_all_with_tags_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<ReadingNoteWithTagQueryResult>, DbErr> {
        reading_note::Entity::find()
            .filter(reading_note::Column::UserId.eq(user_id))
            .column_as(tag::Column::Id, "tag_id")
            .column_as(tag::Column::Name, "tag_name")
            .column_as(tag::Column::CreatedAt, "tag_created_at")
            .column_as(ambition::Column::Name, "tag_ambition_name")
            .column_as(desired_state::Column::Name, "tag_desired_state_name")
            .column_as(action::Column::Name, "tag_action_name")
            .join_rev(LeftJoin, reading_notes_tags::Relation::ReadingNote.def())
            .join(LeftJoin, reading_notes_tags::Relation::Tag.def())
            .join(LeftJoin, tag::Relation::Ambition.def())
            .join(LeftJoin, tag::Relation::DesiredState.def())
            .join(LeftJoin, tag::Relation::Action.def())
            .order_by_desc(reading_note::Column::Date)
            .order_by_desc(reading_note::Column::CreatedAt)
            .order_by_with_nulls(ambition::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(desired_state::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(action::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(tag::Column::CreatedAt, Asc, Last)
            .into_model::<ReadingNoteWithTagQueryResult>()
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        reading_note_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<reading_note::Model, DbErr> {
        reading_note::Entity::find_by_id(reading_note_id)
            .filter(reading_note::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::ActiveModelTrait;
    use test_utils::{self, *};

    use super::*;

    #[actix_web::test]
    async fn find_all_with_tags_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let reading_note_0 = factory::reading_note(user.id)
            .title("reading_note_0".to_string())
            .insert(&db)
            .await?;
        let reading_note_1 = factory::reading_note(user.id)
            .title("reading_note_1".to_string())
            .insert(&db)
            .await?;
        let plain_tag = factory::tag(user.id).insert(&db).await?;
        let (action, action_tag) = factory::action(user.id).insert_with_tag(&db).await?;
        let (ambition, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        let (desired_state, desired_state_tag) =
            factory::desired_state(user.id).insert_with_tag(&db).await?;
        factory::link_reading_note_tag(&db, reading_note_0.id, plain_tag.id).await?;
        factory::link_reading_note_tag(&db, reading_note_0.id, ambition_tag.id).await?;
        factory::link_reading_note_tag(&db, reading_note_1.id, desired_state_tag.id).await?;
        factory::link_reading_note_tag(&db, reading_note_1.id, action_tag.id).await?;

        let res: Vec<ReadingNoteWithTagQueryResult> =
            ReadingNoteQuery::find_all_with_tags_by_user_id(&db, user.id).await?;

        let expected = vec![
            ReadingNoteWithTagQueryResult {
                id: reading_note_1.id,
                title: reading_note_1.title.clone(),
                page_number: reading_note_1.page_number,
                text: reading_note_1.text.clone(),
                date: reading_note_1.date,
                created_at: reading_note_1.created_at,
                updated_at: reading_note_1.updated_at,
                tag_id: Some(desired_state_tag.id),
                tag_name: None,
                tag_ambition_name: None,
                tag_desired_state_name: Some(desired_state.name),
                tag_action_name: None,
                tag_created_at: Some(desired_state_tag.created_at),
            },
            ReadingNoteWithTagQueryResult {
                id: reading_note_1.id,
                title: reading_note_1.title.clone(),
                page_number: reading_note_1.page_number,
                text: reading_note_1.text.clone(),
                date: reading_note_1.date,
                created_at: reading_note_1.created_at,
                updated_at: reading_note_1.updated_at,
                tag_id: Some(action_tag.id),
                tag_name: None,
                tag_ambition_name: None,
                tag_desired_state_name: None,
                tag_action_name: Some(action.name),
                tag_created_at: Some(action_tag.created_at),
            },
            ReadingNoteWithTagQueryResult {
                id: reading_note_0.id,
                title: reading_note_0.title.clone(),
                page_number: reading_note_0.page_number,
                text: reading_note_0.text.clone(),
                date: reading_note_0.date,
                created_at: reading_note_0.created_at,
                updated_at: reading_note_0.updated_at,
                tag_id: Some(ambition_tag.id),
                tag_name: None,
                tag_ambition_name: Some(ambition.name),
                tag_desired_state_name: None,
                tag_action_name: None,
                tag_created_at: Some(ambition_tag.created_at),
            },
            ReadingNoteWithTagQueryResult {
                id: reading_note_0.id,
                title: reading_note_0.title.clone(),
                page_number: reading_note_0.page_number,
                text: reading_note_0.text.clone(),
                date: reading_note_0.date,
                created_at: reading_note_0.created_at,
                updated_at: reading_note_0.updated_at,
                tag_id: Some(plain_tag.id),
                tag_name: Some(plain_tag.name.unwrap()),
                tag_ambition_name: None,
                tag_desired_state_name: None,
                tag_action_name: None,
                tag_created_at: Some(plain_tag.created_at),
            },
        ];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res, expected);

        Ok(())
    }
}
