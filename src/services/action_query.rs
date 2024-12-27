use crate::entities::{action, ambition, ambitions_objectives, objective, objectives_actions};
use crate::types::{ActionVisible, ActionWithLinksQueryResult, CustomDbErr};
use sea_orm::{entity::prelude::*, Condition, JoinType::LeftJoin, QueryOrder, QuerySelect};

pub struct ActionQuery;

impl ActionQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<ActionVisible>, DbErr> {
        action::Entity::find()
            .filter(action::Column::UserId.eq(user_id))
            .filter(action::Column::Archived.eq(false))
            .order_by_asc(action::Column::CreatedAt)
            .into_partial_model::<ActionVisible>()
            .all(db)
            .await
    }

    pub async fn find_all_with_linked_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<ActionWithLinksQueryResult>, DbErr> {
        action::Entity::find()
            .filter(action::Column::UserId.eq(user_id))
            .filter(action::Column::Archived.eq(false))
            .filter(
                Condition::any()
                    .add(objective::Column::Archived.eq(false))
                    .add(objective::Column::Archived.is_null()),
            )
            .filter(
                Condition::any()
                    .add(ambition::Column::Archived.eq(false))
                    .add(ambition::Column::Archived.is_null()),
            )
            .column_as(objective::Column::Id, "objective_id")
            .column_as(objective::Column::Name, "objective_name")
            .column_as(objective::Column::Description, "objective_description")
            .column_as(objective::Column::CreatedAt, "objective_created_at")
            .column_as(objective::Column::UpdatedAt, "objective_updated_at")
            .column_as(ambition::Column::Id, "ambition_id")
            .column_as(ambition::Column::Name, "ambition_name")
            .column_as(ambition::Column::Description, "ambition_description")
            .column_as(ambition::Column::CreatedAt, "ambition_created_at")
            .column_as(ambition::Column::UpdatedAt, "ambition_updated_at")
            .join_rev(LeftJoin, objectives_actions::Relation::Action.def())
            .join(LeftJoin, objectives_actions::Relation::Objective.def())
            .join_rev(LeftJoin, ambitions_objectives::Relation::Objective.def())
            .join(LeftJoin, ambitions_objectives::Relation::Ambition.def())
            .order_by_asc(action::Column::CreatedAt)
            .order_by_asc(objective::Column::CreatedAt)
            .order_by_asc(ambition::Column::CreatedAt)
            .into_model::<ActionWithLinksQueryResult>()
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        action_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<action::Model, DbErr> {
        action::Entity::find_by_id(action_id)
            .filter(action::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
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
        let action_0 =
            test_utils::seed::create_action(&db, "action_0".to_string(), None, user.id).await?;
        let action_1 = test_utils::seed::create_action(
            &db,
            "action_1".to_string(),
            Some("Action_1".to_string()),
            user.id,
        )
        .await?;
        let _archived_action =
            test_utils::seed::create_action(&db, "archived_action".to_string(), None, user.id)
                .await?
                .archive(&db)
                .await?;

        let res = ActionQuery::find_all_by_user_id(&db, user.id).await?;

        let expected = vec![
            ActionVisible {
                id: action_0.id,
                name: action_0.name,
                description: action_0.description,
                created_at: action_0.created_at,
                updated_at: action_0.updated_at,
            },
            ActionVisible {
                id: action_1.id,
                name: action_1.name,
                description: action_1.description,
                created_at: action_1.created_at,
                updated_at: action_1.updated_at,
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
        let objective_1 = objective_1.connect_action(&db, action_0.id).await?;
        let ambition_1 = ambition_1.connect_objective(&db, objective_0.id).await?;

        let res = ActionQuery::find_all_with_linked_by_user_id(&db, user.id).await?;

        assert_eq!(res.len(), 4);

        // NOTE: Check only ids for convenience.
        let res_organized = vec![
            (res[0].id, res[0].objective_id, res[0].ambition_id),
            (res[1].id, res[1].objective_id, res[1].ambition_id),
            (res[2].id, res[2].objective_id, res[2].ambition_id),
            (res[3].id, res[3].objective_id, res[3].ambition_id),
        ];
        let expected = vec![
            (action_0.id, Some(objective_0.id), Some(ambition_0.id)),
            (action_0.id, Some(objective_0.id), Some(ambition_1.id)),
            (action_0.id, Some(objective_1.id), None),
            (action_1.id, None, None),
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
        let _archived_action =
            test_utils::seed::create_action(&db, "archived".to_string(), None, user.id)
                .await?
                .archive(&db)
                .await?;
        let _archived_objective =
            test_utils::seed::create_objective(&db, "archived".to_string(), None, user.id)
                .await?
                .archive(&db)
                .await?
                .connect_action(&db, action_0.id)
                .await?;
        let _archived_ambition =
            test_utils::seed::create_ambition(&db, "archived".to_string(), None, user.id)
                .await?
                .archive(&db)
                .await?
                .connect_objective(&db, objective_0.id)
                .await?;

        let res = ActionQuery::find_all_with_linked_by_user_id(&db, user.id).await?;

        assert_eq!(res.len(), 1);

        // NOTE: Check only ids for convenience.
        let res_organized = vec![(res[0].id, res[0].objective_id, res[0].ambition_id)];
        let expected = vec![(action_0.id, Some(objective_0.id), Some(ambition_0.id))];
        assert_eq!(res_organized[0], expected[0]);

        Ok(())
    }
}
