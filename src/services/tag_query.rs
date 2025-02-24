use entities::{action, ambition, objective, tag};
use ::types::TagQueryResult;
use migration::NullOrdering::Last;
use sea_orm::{
    entity::prelude::*, Condition, JoinType::LeftJoin, Order::Asc, QueryOrder, QuerySelect,
};

pub struct TagQuery;

impl TagQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<TagQueryResult>, DbErr> {
        tag::Entity::find()
            .filter(tag::Column::UserId.eq(user_id))
            .filter(
                Condition::any()
                    .add(ambition::Column::Archived.eq(false))
                    .add(ambition::Column::Archived.is_null()),
            )
            .filter(
                Condition::any()
                    .add(objective::Column::Archived.eq(false))
                    .add(objective::Column::Archived.is_null()),
            )
            .filter(
                Condition::any()
                    .add(action::Column::Archived.eq(false))
                    .add(action::Column::Archived.is_null()),
            )
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
    use test_utils::{self, *};

    use super::*;

    #[actix_web::test]
    async fn find_all_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let (_, objective_tag) = factory::objective(user.id).insert_with_tag(&db).await?;
        let (_, ambition_tag) = factory::ambition(user.id).insert_with_tag(&db).await?;
        let (_, action_tag) = factory::action(user.id).insert_with_tag(&db).await?;
        let _archived_action = factory::action(user.id)
            .archived(true)
            .insert_with_tag(&db)
            .await?;
        let _archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert_with_tag(&db)
            .await?;
        let _archived_objective = factory::objective(user.id)
            .archived(true)
            .insert_with_tag(&db)
            .await?;

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
