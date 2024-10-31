use crate::entities::{action, ambition, ambitions_objectives, objective, objectives_actions};
use crate::types::{AmbitionVisible, AmbitionWithLinksQueryResult, CustomDbErr};
use sea_orm::{entity::prelude::*, JoinType::LeftJoin, QueryOrder, QuerySelect};

pub struct AmbitionQuery;

impl AmbitionQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<AmbitionVisible>, DbErr> {
        ambition::Entity::find()
            .filter(ambition::Column::UserId.eq(user_id))
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
            .column_as(objective::Column::Id, "objective_id")
            .column_as(objective::Column::Name, "objective_name")
            .column_as(objective::Column::CreatedAt, "objective_created_at")
            .column_as(objective::Column::UpdatedAt, "objective_updated_at")
            .column_as(action::Column::Id, "action_id")
            .column_as(action::Column::Name, "action_name")
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

    use sea_orm::Set;

    use crate::test_utils;

    use super::*;

    #[actix_web::test]
    async fn find_all_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = test_utils::seed::create_active_user(&db).await?;
        let (ambition_1, _) = test_utils::seed::create_ambition_and_tag(
            &db,
            "ambition_1".to_string(),
            Some("desc_1".to_string()),
            user.id,
        )
        .await?;
        let (ambition_2, _) =
            test_utils::seed::create_ambition_and_tag(&db, "ambition_2".to_string(), None, user.id)
                .await?;

        let res = AmbitionQuery::find_all_by_user_id(&db, user.id).await?;

        let expected = vec![
            AmbitionVisible {
                id: ambition_1.id,
                name: ambition_1.name,
                description: ambition_1.description,
                created_at: ambition_1.created_at,
                updated_at: ambition_1.updated_at,
            },
            AmbitionVisible {
                id: ambition_2.id,
                name: ambition_2.name,
                description: ambition_2.description,
                created_at: ambition_2.created_at,
                updated_at: ambition_2.updated_at,
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
        let (ambition_1, objective_1, action_1) =
            test_utils::seed::create_set_of_ambition_objective_action(&db, user.id, true, true)
                .await?;
        let (ambition_2, objective_2, action_2) =
            test_utils::seed::create_set_of_ambition_objective_action(&db, user.id, false, false)
                .await?;
        let _ = objectives_actions::ActiveModel {
            objective_id: Set(objective_1.id),
            action_id: Set(action_2.id),
        }
        .insert(&db)
        .await?;
        let _ = ambitions_objectives::ActiveModel {
            ambition_id: Set(ambition_1.id),
            objective_id: Set(objective_2.id),
        }
        .insert(&db)
        .await?;

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
            (ambition_1.id, Some(objective_1.id), Some(action_1.id)),
            (ambition_1.id, Some(objective_1.id), Some(action_2.id)),
            (ambition_1.id, Some(objective_2.id), None),
            (ambition_2.id, None, None),
        ];
        assert_eq!(res_organized[0], expected[0]);
        assert_eq!(res_organized[1], expected[1]);
        assert_eq!(res_organized[2], expected[2]);
        assert_eq!(res_organized[3], expected[3]);

        Ok(())
    }
}
