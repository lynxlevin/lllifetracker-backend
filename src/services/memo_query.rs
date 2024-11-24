use crate::entities::{action, ambition, memo, memos_tags, objective, tag};
use crate::types::MemoWithTagQueryResult;
use sea_orm::entity::prelude::*;
use sea_orm::{QueryOrder, QuerySelect, JoinType::LeftJoin};

pub struct MemoQuery;

impl MemoQuery {
    pub async fn find_all_with_tags_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<MemoWithTagQueryResult>, DbErr> {
        memo::Entity::find()
            .filter(memo::Column::UserId.eq(user_id))
            .column_as(tag::Column::Id, "tag_id")
            .column_as(ambition::Column::Name, "tag_ambition_name")
            .column_as(objective::Column::Name, "tag_objective_name")
            .column_as(action::Column::Name, "tag_action_name")
            .join_rev(LeftJoin, memos_tags::Relation::Memo.def())
            .join(LeftJoin, memos_tags::Relation::Tag.def())
            .join(LeftJoin, tag::Relation::Ambition.def())
            .join(LeftJoin, tag::Relation::Objective.def())
            .join(LeftJoin, tag::Relation::Action.def())
            .order_by_desc(memo::Column::CreatedAt)
            .order_by_desc(tag::Column::CreatedAt)
            .into_model::<MemoWithTagQueryResult>()
            .all(db)
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils;
    use sea_orm::ActiveValue::Set;

    use super::*;

    #[actix_web::test]
    async fn find_all_with_tags_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let memo_0 = test_utils::seed::create_memo(&db, "memo_0".to_string(), user.id).await?;
        let memo_1 = test_utils::seed::create_memo(&db, "memo_1".to_string(), user.id).await?;
        let (ambition, ambition_tag) = test_utils::seed::create_ambition_and_tag(&db, "ambition".to_string(), None, user.id).await?;
        let (objective, objective_tag) = test_utils::seed::create_objective_and_tag(&db, "objective".to_string(), user.id).await?;
        let (action, action_tag) = test_utils::seed::create_action_and_tag(&db, "action".to_string(), user.id).await?;
        memos_tags::ActiveModel {
            memo_id: Set(memo_0.id),
            tag_id: Set(ambition_tag.id),
        }.insert(&db).await?;
        memos_tags::ActiveModel {
            memo_id: Set(memo_1.id),
            tag_id: Set(objective_tag.id),
        }.insert(&db).await?;
        memos_tags::ActiveModel {
            memo_id: Set(memo_1.id),
            tag_id: Set(action_tag.id),
        }.insert(&db).await?;

        let res: Vec<MemoWithTagQueryResult> = MemoQuery::find_all_with_tags_by_user_id(&db, user.id).await?;

        let expected = vec![
            MemoWithTagQueryResult {
                id: memo_1.id,
                title: memo_1.title.clone(),
                text: memo_1.text.clone(),
                date: memo_1.date,
                created_at: memo_1.created_at,
                updated_at: memo_1.updated_at,
                tag_id: Some(action_tag.id),
                tag_ambition_name: None,
                tag_objective_name: None,
                tag_action_name: Some(action.name),
            },
            MemoWithTagQueryResult {
                id: memo_1.id,
                title: memo_1.title.clone(),
                text: memo_1.text.clone(),
                date: memo_1.date,
                created_at: memo_1.created_at,
                updated_at: memo_1.updated_at,
                tag_id: Some(objective_tag.id),
                tag_ambition_name: None,
                tag_objective_name: Some(objective.name),
                tag_action_name: None,
            },
            MemoWithTagQueryResult {
                id: memo_0.id,
                title: memo_0.title.clone(),
                text: memo_0.text.clone(),
                date: memo_0.date,
                created_at: memo_0.created_at,
                updated_at: memo_0.updated_at,
                tag_id: Some(ambition_tag.id),
                tag_ambition_name: Some(ambition.name),
                tag_objective_name: None,
                tag_action_name: None,
            },
        ];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);
        assert_eq!(res[1], expected[1]);
        assert_eq!(res[2], expected[2]);

        Ok(())
    }
}