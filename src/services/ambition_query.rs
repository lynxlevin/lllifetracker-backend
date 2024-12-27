use crate::entities::{action, ambition, ambitions_objectives, objective, objectives_actions};
use crate::types::{AmbitionVisible, AmbitionWithLinksQueryResult, CustomDbErr};
use sea_orm::{entity::prelude::*, Condition, JoinType::LeftJoin, QueryOrder, QuerySelect};

pub struct AmbitionQuery;

impl AmbitionQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<AmbitionVisible>, DbErr> {
        ambition::Entity::find()
            .filter(ambition::Column::UserId.eq(user_id))
            .filter(ambition::Column::Archived.eq(false))
            .order_by_asc(ambition::Column::CreatedAt)
            .into_partial_model::<AmbitionVisible>()
            .all(db)
            .await
    }

    pub async fn find_all_with_linked_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<AmbitionWithLinksQueryResult>, DbErr> {
        ambition::Entity::find()
            .filter(ambition::Column::UserId.eq(user_id))
            .filter(ambition::Column::Archived.eq(false))
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
            .column_as(objective::Column::Id, "objective_id")
            .column_as(objective::Column::Name, "objective_name")
            .column_as(objective::Column::Description, "objective_description")
            .column_as(objective::Column::CreatedAt, "objective_created_at")
            .column_as(objective::Column::UpdatedAt, "objective_updated_at")
            .column_as(action::Column::Id, "action_id")
            .column_as(action::Column::Name, "action_name")
            .column_as(action::Column::Description, "action_description")
            .column_as(action::Column::CreatedAt, "action_created_at")
            .column_as(action::Column::UpdatedAt, "action_updated_at")
            .join_rev(LeftJoin, ambitions_objectives::Relation::Ambition.def())
            .join(LeftJoin, ambitions_objectives::Relation::Objective.def())
            .join_rev(LeftJoin, objectives_actions::Relation::Objective.def())
            .join(LeftJoin, objectives_actions::Relation::Action.def())
            .order_by_asc(ambition::Column::CreatedAt)
            .order_by_asc(objective::Column::CreatedAt)
            .order_by_asc(action::Column::CreatedAt)
            .into_model::<AmbitionWithLinksQueryResult>()
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        ambition_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<ambition::Model, DbErr> {
        ambition::Entity::find_by_id(ambition_id)
            .filter(ambition::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::test_utils::{self, factory};

    use super::*;

    #[actix_web::test]
    async fn find_all_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let ambition_0 = factory::ambition(user.id)
            .name("ambition_0".to_string())
            .description(Some("desc_0".to_string()))
            .insert(&db)
            .await?;
        let ambition_1 = factory::ambition(user.id)
            .name("ambition_1".to_string())
            .insert(&db)
            .await?;
        let _archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let res = AmbitionQuery::find_all_by_user_id(&db, user.id).await?;

        let expected = vec![
            AmbitionVisible {
                id: ambition_0.id,
                name: ambition_0.name,
                description: ambition_0.description,
                created_at: ambition_0.created_at,
                updated_at: ambition_0.updated_at,
            },
            AmbitionVisible {
                id: ambition_1.id,
                name: ambition_1.name,
                description: ambition_1.description,
                created_at: ambition_1.created_at,
                updated_at: ambition_1.updated_at,
            },
        ];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);
        assert_eq!(res[1], expected[1]);

        Ok(())
    }

    #[actix_web::test]
    async fn find_all_with_linked_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (ambition_0, objective_0, action_0) =
            test_utils::seed::create_set_of_ambition_objective_action(&db, user.id, true, true)
                .await?;
        let (ambition_1, objective_1, action_1) =
            test_utils::seed::create_set_of_ambition_objective_action(&db, user.id, false, false)
                .await?;
        let ambition_0 = ambition_0.connect_objective(&db, objective_1.id).await?;
        let objective_0 = objective_0.connect_action(&db, action_1.id).await?;

        let res = AmbitionQuery::find_all_with_linked_by_user_id(&db, user.id).await?;

        assert_eq!(res.len(), 4);

        // NOTE: Check only ids for convenience.
        let res_organized = vec![
            (res[0].id, res[0].objective_id, res[0].action_id),
            (res[1].id, res[1].objective_id, res[1].action_id),
            (res[2].id, res[2].objective_id, res[2].action_id),
            (res[3].id, res[3].objective_id, res[3].action_id),
        ];
        let expected = vec![
            (ambition_0.id, Some(objective_0.id), Some(action_0.id)),
            (ambition_0.id, Some(objective_0.id), Some(action_1.id)),
            (ambition_0.id, Some(objective_1.id), None),
            (ambition_1.id, None, None),
        ];
        assert_eq!(res_organized[0], expected[0]);
        assert_eq!(res_organized[1], expected[1]);
        assert_eq!(res_organized[2], expected[2]);
        assert_eq!(res_organized[3], expected[3]);

        Ok(())
    }

    #[actix_web::test]
    async fn find_all_with_linked_by_user_id_archived_items_should_not_be_returned(
    ) -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (ambition_0, objective_0, action_0) =
            test_utils::seed::create_set_of_ambition_objective_action(&db, user.id, true, true)
                .await?;
        let _archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let archived_objective = factory::objective(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let archived_action = factory::action(user.id).archived(true).insert(&db).await?;
        let ambition_0 = ambition_0
            .connect_objective(&db, archived_objective.id)
            .await?;
        let objective_0 = objective_0.connect_action(&db, archived_action.id).await?;

        let res = AmbitionQuery::find_all_with_linked_by_user_id(&db, user.id).await?;

        assert_eq!(res.len(), 1);

        // NOTE: Check only ids for convenience.
        let res_organized = vec![(res[0].id, res[0].objective_id, res[0].action_id)];
        let expected = vec![(ambition_0.id, Some(objective_0.id), Some(action_0.id))];
        assert_eq!(res_organized, expected);

        Ok(())
    }
}
