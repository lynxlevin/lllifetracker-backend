use crate::entities::{action, ambition, ambitions_objectives, objective, objectives_actions};
use crate::types::{CustomDbErr, ObjectiveVisible, ObjectiveWithLinksQueryResult};
use sea_orm::{entity::prelude::*, JoinType::LeftJoin, QueryOrder, QuerySelect};

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewObjective {
    pub name: String,
    pub user_id: uuid::Uuid,
}

pub struct ObjectiveQuery;

impl ObjectiveQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<ObjectiveVisible>, DbErr> {
        objective::Entity::find()
            .filter(objective::Column::UserId.eq(user_id))
            .order_by_asc(objective::Column::CreatedAt)
            .into_partial_model::<ObjectiveVisible>()
            .all(db)
            .await
    }

    pub async fn find_all_with_linked_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<ObjectiveWithLinksQueryResult>, DbErr> {
        objective::Entity::find()
            .filter(objective::Column::UserId.eq(user_id))
            .column_as(ambition::Column::Id, "ambition_id")
            .column_as(ambition::Column::Name, "ambition_name")
            .column_as(ambition::Column::Description, "ambition_description")
            .column_as(ambition::Column::CreatedAt, "ambition_created_at")
            .column_as(ambition::Column::UpdatedAt, "ambition_updated_at")
            .column_as(action::Column::Id, "action_id")
            .column_as(action::Column::Name, "action_name")
            .column_as(action::Column::CreatedAt, "action_created_at")
            .column_as(action::Column::UpdatedAt, "action_updated_at")
            .join_rev(LeftJoin, objectives_actions::Relation::Objective.def())
            .join(LeftJoin, objectives_actions::Relation::Action.def())
            .join_rev(LeftJoin, ambitions_objectives::Relation::Objective.def())
            .join(LeftJoin, ambitions_objectives::Relation::Ambition.def())
            .order_by_asc(objective::Column::CreatedAt)
            .order_by_asc(ambition::Column::CreatedAt)
            .order_by_asc(action::Column::CreatedAt)
            .into_model::<ObjectiveWithLinksQueryResult>()
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        objective_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<objective::Model, DbErr> {
        objective::Entity::find_by_id(objective_id)
            .filter(objective::Column::UserId.eq(user_id))
            .one(db)
            .await?
            .ok_or(DbErr::Custom(CustomDbErr::NotFound.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use sea_orm::Set;

    use crate::test_utils;

    use super::*;

    #[actix_web::test]
    async fn find_all_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (objective_0, _) =
            test_utils::seed::create_objective_and_tag(&db, "objective_0".to_string(), user.id)
                .await?;
        let (objective_1, _) =
            test_utils::seed::create_objective_and_tag(&db, "objective_1".to_string(), user.id)
                .await?;

        let res = ObjectiveQuery::find_all_by_user_id(&db, user.id).await?;

        let expected = vec![
            ObjectiveVisible {
                id: objective_0.id,
                name: objective_0.name,
                created_at: objective_0.created_at,
                updated_at: objective_0.updated_at,
            },
            ObjectiveVisible {
                id: objective_1.id,
                name: objective_1.name,
                created_at: objective_1.created_at,
                updated_at: objective_1.updated_at,
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
        let _ = objectives_actions::ActiveModel {
            objective_id: Set(objective_0.id),
            action_id: Set(action_1.id),
        }
        .insert(&db)
        .await?;
        let _ = objectives_actions::ActiveModel {
            objective_id: Set(objective_1.id),
            action_id: Set(action_1.id),
        }
        .insert(&db)
        .await?;
        let _ = ambitions_objectives::ActiveModel {
            ambition_id: Set(ambition_1.id),
            objective_id: Set(objective_0.id),
        }
        .insert(&db)
        .await?;

        let res = ObjectiveQuery::find_all_with_linked_by_user_id(&db, user.id).await?;
        dbg!(&res);

        assert_eq!(res.len(), 5);

        // NOTE: Check only ids for convenience.
        let res_organized = vec![
            (res[0].id, res[0].ambition_id, res[0].action_id),
            (res[1].id, res[1].ambition_id, res[1].action_id),
            (res[2].id, res[2].ambition_id, res[2].action_id),
            (res[3].id, res[3].ambition_id, res[3].action_id),
            (res[4].id, res[4].ambition_id, res[4].action_id),
        ];
        let expected = vec![
            (objective_0.id, Some(ambition_0.id), Some(action_0.id)),
            (objective_0.id, Some(ambition_0.id), Some(action_1.id)),
            (objective_0.id, Some(ambition_1.id), Some(action_0.id)),
            (objective_0.id, Some(ambition_1.id), Some(action_1.id)),
            (objective_1.id, None, Some(action_1.id)),
        ];
        assert_eq!(res_organized[0], expected[0]);
        assert_eq!(res_organized[1], expected[1]);
        assert_eq!(res_organized[2], expected[2]);
        assert_eq!(res_organized[3], expected[3]);
        assert_eq!(res_organized[4], expected[4]);

        Ok(())
    }
}
