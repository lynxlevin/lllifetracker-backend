use crate::entities::{action, ambition, ambitions_objectives, objective, objectives_actions};
use crate::types::{ActionVisible, ActionWithLinksQueryResult, CustomDbErr};
use migration::{Alias, IntoCondition, NullOrdering::Last};
use sea_orm::{entity::prelude::*, JoinType::LeftJoin, Order::Asc, QueryOrder, QuerySelect};

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
            .join_as(
                LeftJoin,
                objectives_actions::Relation::Objective
                    .def()
                    .on_condition(|_left, right| {
                        Expr::col((right, objective::Column::Archived))
                            .eq(false)
                            .into_condition()
                    }),
                Alias::new("objective"),
            )
            .join_rev(LeftJoin, ambitions_objectives::Relation::Objective.def())
            .join_as(
                LeftJoin,
                ambitions_objectives::Relation::Ambition
                    .def()
                    .on_condition(|_left, right| {
                        Expr::col((right, ambition::Column::Archived))
                            .eq(false)
                            .into_condition()
                    }),
                Alias::new("ambition"),
            )
            .order_by_asc(action::Column::CreatedAt)
            .order_by_with_nulls(objective::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(ambition::Column::CreatedAt, Asc, Last)
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
    use crate::test_utils::{self, *};

    use super::*;

    #[actix_web::test]
    async fn find_all_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let action_0 = factory::action(user.id)
            .name("action_0".to_string())
            .insert(&db)
            .await?;
        let action_1 = factory::action(user.id)
            .name("action_1".to_string())
            .description(Some("Action_1".to_string()))
            .insert(&db)
            .await?;
        let _archived_action = factory::action(user.id).archived(true).insert(&db).await?;

        let res = ActionQuery::find_all_by_user_id(&db, user.id).await?;

        let expected = [
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
        let user = factory::user().insert(&db).await?;
        let ambition_0 = factory::ambition(user.id).insert(&db).await?;
        let objective_0 = factory::objective(user.id).insert(&db).await?;
        let action_0 = factory::action(user.id).insert(&db).await?;
        let ambition_1 = factory::ambition(user.id).insert(&db).await?;
        let objective_1 = factory::objective(user.id).insert(&db).await?;
        let action_1 = factory::action(user.id).insert(&db).await?;
        factory::link_objective_action(&db, objective_0.id, action_0.id).await?;
        factory::link_objective_action(&db, objective_1.id, action_0.id).await?;
        factory::link_ambition_objective(&db, ambition_0.id, objective_0.id).await?;
        factory::link_ambition_objective(&db, ambition_1.id, objective_0.id).await?;

        let res = ActionQuery::find_all_with_linked_by_user_id(&db, user.id).await?;

        assert_eq!(res.len(), 4);

        // NOTE: Check only ids for convenience.
        let res_organized = [
            (res[0].id, res[0].objective_id, res[0].ambition_id),
            (res[1].id, res[1].objective_id, res[1].ambition_id),
            (res[2].id, res[2].objective_id, res[2].ambition_id),
            (res[3].id, res[3].objective_id, res[3].ambition_id),
        ];
        let expected = [
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
    async fn find_all_with_linked_by_user_id_archived_items_should_be_returned_as_none(
    ) -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let ambition_0 = factory::ambition(user.id).insert(&db).await?;
        let objective_0 = factory::objective(user.id).insert(&db).await?;
        let action_0 = factory::action(user.id).insert(&db).await?;
        let _archived_action = factory::action(user.id).archived(true).insert(&db).await?;
        let archived_objective = factory::objective(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        factory::link_objective_action(&db, objective_0.id, action_0.id).await?;
        factory::link_objective_action(&db, archived_objective.id, action_0.id).await?;
        factory::link_ambition_objective(&db, ambition_0.id, objective_0.id).await?;
        factory::link_ambition_objective(&db, archived_ambition.id, objective_0.id).await?;

        let res = ActionQuery::find_all_with_linked_by_user_id(&db, user.id).await?;

        assert_eq!(res.len(), 3);

        // NOTE: Check only ids for convenience.
        let res_organized = [
            (res[0].id, res[0].objective_id, res[0].ambition_id),
            (res[1].id, res[1].objective_id, res[1].ambition_id),
            (res[2].id, res[2].objective_id, res[2].ambition_id),
        ];
        let expected = [
            (action_0.id, Some(objective_0.id), Some(ambition_0.id)),
            (action_0.id, Some(objective_0.id), None),
            (action_0.id, None, None),
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
        let action = factory::action(user.id).insert(&db).await?;
        let archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let archived_objective = factory::objective(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        factory::link_objective_action(&db, archived_objective.id, action.id).await?;
        factory::link_ambition_objective(&db, archived_ambition.id, archived_objective.id).await?;

        let res = ActionQuery::find_all_with_linked_by_user_id(&db, user.id).await?;

        assert_eq!(res.len(), 1);

        // NOTE: Check only ids for convenience.
        let res_organized = [(res[0].id, res[0].objective_id, res[0].ambition_id)];
        let expected = [(action.id, None, None)];
        assert_eq!(res_organized[0], expected[0]);

        Ok(())
    }
}
