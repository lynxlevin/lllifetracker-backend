use entities::{action, ambition, ambitions_objectives, objective, objectives_actions};
use ::types::{AmbitionVisible, AmbitionWithLinksQueryResult, CustomDbErr};
use migration::{Alias, IntoCondition, NullOrdering::Last};
use sea_orm::{
    entity::prelude::*, JoinType::LeftJoin, Order::Asc, QueryOrder, QuerySelect,
};

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
            .join_as(
                LeftJoin,
                ambitions_objectives::Relation::Objective
                    .def()
                    .on_condition(|_left, right| {
                        Expr::col((right, objective::Column::Archived))
                            .eq(false)
                            .into_condition()
                    }),
                Alias::new("objective"),
            )
            .join_rev(LeftJoin, objectives_actions::Relation::Objective.def())
            .join_as(
                LeftJoin,
                objectives_actions::Relation::Action
                    .def()
                    .on_condition(|_left, right| {
                        Expr::col((right, action::Column::Archived))
                            .eq(false)
                            .into_condition()
                    }),
                Alias::new("action"),
            )
            .order_by_asc(ambition::Column::CreatedAt)
            .order_by_with_nulls(objective::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(action::Column::CreatedAt, Asc, Last)
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
    use crate::test_utils::{self, *};

    use super::*;

    #[actix_web::test]
    async fn find_all_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
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

        let expected = [
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
        let user = factory::user().insert(&db).await?;
        let ambition_0 = factory::ambition(user.id).insert(&db).await?;
        let objective_0 = factory::objective(user.id).insert(&db).await?;
        let action_0 = factory::action(user.id).insert(&db).await?;
        let ambition_1 = factory::ambition(user.id).insert(&db).await?;
        let objective_1 = factory::objective(user.id).insert(&db).await?;
        let action_1 = factory::action(user.id).insert(&db).await?;
        factory::link_ambition_objective(&db, ambition_0.id, objective_0.id).await?;
        factory::link_ambition_objective(&db, ambition_0.id, objective_1.id).await?;
        factory::link_objective_action(&db, objective_0.id, action_0.id).await?;
        factory::link_objective_action(&db, objective_0.id, action_1.id).await?;

        let res = AmbitionQuery::find_all_with_linked_by_user_id(&db, user.id).await?;

        assert_eq!(res.len(), 4);

        // NOTE: Check only ids for convenience.
        let res_organized = [
            (res[0].id, res[0].objective_id, res[0].action_id),
            (res[1].id, res[1].objective_id, res[1].action_id),
            (res[2].id, res[2].objective_id, res[2].action_id),
            (res[3].id, res[3].objective_id, res[3].action_id),
        ];
        let expected = [
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
    async fn find_all_with_linked_by_user_id_archived_items_should_be_returned_as_none(
    ) -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let ambition_0 = factory::ambition(user.id).insert(&db).await?;
        let objective_0 = factory::objective(user.id).insert(&db).await?;
        let action_0 = factory::action(user.id).insert(&db).await?;
        let _archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let archived_objective = factory::objective(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let archived_action = factory::action(user.id).archived(true).insert(&db).await?;
        factory::link_ambition_objective(&db, ambition_0.id, objective_0.id).await?;
        factory::link_ambition_objective(&db, ambition_0.id, archived_objective.id).await?;
        factory::link_objective_action(&db, objective_0.id, action_0.id).await?;
        factory::link_objective_action(&db, objective_0.id, archived_action.id).await?;

        let res = AmbitionQuery::find_all_with_linked_by_user_id(&db, user.id).await?;

        assert_eq!(res.len(), 3);

        // NOTE: Check only ids for convenience.
        let res_organized = [
            (res[0].id, res[0].objective_id, res[0].action_id),
            (res[1].id, res[1].objective_id, res[1].action_id),
            (res[2].id, res[2].objective_id, res[2].action_id),
        ];
        let expected = [
            (ambition_0.id, Some(objective_0.id), Some(action_0.id)),
            (ambition_0.id, Some(objective_0.id), None),
            (ambition_0.id, None, None),
        ];
        assert_eq!(res_organized[0], expected[0]);
        assert_eq!(res_organized[1], expected[1]);
        assert_eq!(res_organized[2], expected[2]);

        Ok(())
    }

    #[actix_web::test]
    async fn find_all_with_linked_by_user_id_item_linked_to_archived_items_should_be_returned(
    ) -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let ambition = factory::ambition(user.id).insert(&db).await?;
        let archived_objective = factory::objective(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let archived_action = factory::action(user.id).archived(true).insert(&db).await?;
        factory::link_ambition_objective(&db, ambition.id, archived_objective.id).await?;
        factory::link_objective_action(&db, archived_objective.id, archived_action.id).await?;

        let res = AmbitionQuery::find_all_with_linked_by_user_id(&db, user.id).await?;

        assert_eq!(res.len(), 1);

        // NOTE: Check only ids for convenience.
        let res_organized = [(res[0].id, res[0].objective_id, res[0].action_id)];
        let expected = [(ambition.id, None, None)];
        assert_eq!(res_organized[0], expected[0]);

        Ok(())
    }
}
