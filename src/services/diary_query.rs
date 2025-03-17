use ::types::{CustomDbErr, DiaryWithTagQueryResult};
use entities::{action, ambition, desired_state, diaries_tags, diary, tag};
use migration::NullOrdering::Last;
use sea_orm::entity::prelude::*;
use sea_orm::{JoinType::LeftJoin, Order::Asc, QueryOrder, QuerySelect};

pub struct DiaryQuery;

impl DiaryQuery {
    pub async fn find_all_with_tags_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<DiaryWithTagQueryResult>, DbErr> {
        diary::Entity::find()
            .filter(diary::Column::UserId.eq(user_id))
            .column_as(tag::Column::Id, "tag_id")
            .column_as(tag::Column::CreatedAt, "tag_created_at")
            .column_as(ambition::Column::Name, "tag_ambition_name")
            .column_as(desired_state::Column::Name, "tag_desired_state_name")
            .column_as(action::Column::Name, "tag_action_name")
            .join_rev(LeftJoin, diaries_tags::Relation::Diary.def())
            .join(LeftJoin, diaries_tags::Relation::Tag.def())
            .join(LeftJoin, tag::Relation::Ambition.def())
            .join(LeftJoin, tag::Relation::DesiredState.def())
            .join(LeftJoin, tag::Relation::Action.def())
            .order_by_desc(diary::Column::Date)
            .order_by_with_nulls(ambition::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(desired_state::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(action::Column::CreatedAt, Asc, Last)
            .into_model::<DiaryWithTagQueryResult>()
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        diary_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<diary::Model, DbErr> {
        diary::Entity::find_by_id(diary_id)
            .filter(diary::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use test_utils::{self, *};

    use super::*;

    #[actix_web::test]
    async fn find_all_with_tags_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let now = Utc::now();
        let diary_0 = factory::diary(user.id)
            .text(Some("diary_0".to_string()))
            .insert(&db)
            .await?;
        let diary_1 = factory::diary(user.id)
            .text(Some("diary_1".to_string()))
            .date((now - Duration::days(1)).date_naive())
            .insert(&db)
            .await?;
        let (action, action_tag) = factory::action(user.id).insert_with_tag(&db).await?;
        let (ambition, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        let (desired_state, desired_state_tag) =
            factory::desired_state(user.id).insert_with_tag(&db).await?;
        factory::link_diary_tag(&db, diary_0.id, ambition_tag.id).await?;
        factory::link_diary_tag(&db, diary_1.id, desired_state_tag.id).await?;
        factory::link_diary_tag(&db, diary_1.id, action_tag.id).await?;

        let res: Vec<DiaryWithTagQueryResult> =
            DiaryQuery::find_all_with_tags_by_user_id(&db, user.id).await?;

        let expected = vec![
            DiaryWithTagQueryResult {
                id: diary_0.id,
                text: diary_0.text.clone(),
                date: diary_0.date,
                score: diary_0.score,
                tag_id: Some(ambition_tag.id),
                tag_ambition_name: Some(ambition.name),
                tag_desired_state_name: None,
                tag_action_name: None,
                tag_created_at: Some(ambition_tag.created_at),
            },
            DiaryWithTagQueryResult {
                id: diary_1.id,
                text: diary_1.text.clone(),
                date: diary_1.date,
                score: diary_1.score,
                tag_id: Some(desired_state_tag.id),
                tag_ambition_name: None,
                tag_desired_state_name: Some(desired_state.name),
                tag_action_name: None,
                tag_created_at: Some(desired_state_tag.created_at),
            },
            DiaryWithTagQueryResult {
                id: diary_1.id,
                text: diary_1.text.clone(),
                date: diary_1.date,
                score: diary_1.score,
                tag_id: Some(action_tag.id),
                tag_ambition_name: None,
                tag_desired_state_name: None,
                tag_action_name: Some(action.name),
                tag_created_at: Some(action_tag.created_at),
            },
        ];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);
        assert_eq!(res[1], expected[1]);
        assert_eq!(res[2], expected[2]);

        Ok(())
    }
}
