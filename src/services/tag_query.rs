use crate::entities::{action, ambition, objective, tag};
use crate::types::TagQueryResult;
use migration::NullOrdering::Last;
use sea_orm::{entity::prelude::*, JoinType::LeftJoin, QueryOrder, QuerySelect, Order::Asc};

pub struct TagQuery;

impl TagQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<TagQueryResult>, DbErr> {
        tag::Entity::find()
            .filter(tag::Column::UserId.eq(user_id))
            .column_as(ambition::Column::Name, "ambition_name")
            .column_as(objective::Column::Name, "objective_name")
            .column_as(action::Column::Name, "action_name")
            .join(LeftJoin, tag::Relation::Ambition.def())
            .join(LeftJoin, tag::Relation::Objective.def())
            .join(LeftJoin, tag::Relation::Action.def())
            .order_by_with_nulls(ambition::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(objective::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(action::Column::CreatedAt, Asc, Last)
            .into_model::<TagQueryResult>()
            .all(db)
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils;

    use super::*;

    #[actix_web::test]
    async fn find_all_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (_, objective_tag) =
            test_utils::seed::create_objective_and_tag(&db, "objective".to_string(), None, user.id)
                .await?;
        let (_, ambition_tag) =
            test_utils::seed::create_ambition_and_tag(&db, "ambition".to_string(), None, user.id)
                .await?;
        let (_, action_tag) =
            test_utils::seed::create_action_and_tag(&db, "action".to_string(), None, user.id).await?;

        let res = TagQuery::find_all_by_user_id(&db, user.id).await?;

        let expected = vec![
            TagQueryResult {
                id: ambition_tag.id,
                ambition_name: Some("ambition".to_string()),
                objective_name: None,
                action_name: None,
                created_at: ambition_tag.created_at,
            },
            TagQueryResult {
                id: objective_tag.id,
                ambition_name: None,
                objective_name: Some("objective".to_string()),
                action_name: None,
                created_at: objective_tag.created_at,
            },
            TagQueryResult {
                id: action_tag.id,
                ambition_name: None,
                objective_name: None,
                action_name: Some("action".to_string()),
                created_at: action_tag.created_at,
            },
        ];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);
        assert_eq!(res[1], expected[1]);
        assert_eq!(res[2], expected[2]);

        Ok(())
    }
}
