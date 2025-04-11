use ::types::{ActionVisible, ActionWithLinksQueryResult, CustomDbErr};
use entities::{action, ambition, ambitions_desired_states, desired_state, desired_states_actions};
use migration::{Alias, IntoCondition, NullOrdering::Last};
use sea_orm::{entity::prelude::*, JoinType::LeftJoin, Order::Asc, QueryOrder, QuerySelect};

pub struct ActionQuery;

impl ActionQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
        show_archived_only: bool,
    ) -> Result<Vec<ActionVisible>, DbErr> {
        action::Entity::find()
            .filter(action::Column::UserId.eq(user_id))
            .filter(action::Column::Archived.eq(show_archived_only))
            .order_by_with_nulls(action::Column::Ordering, Asc, Last)
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
            .column_as(desired_state::Column::Id, "desired_state_id")
            .column_as(desired_state::Column::Name, "desired_state_name")
            .column_as(
                desired_state::Column::Description,
                "desired_state_description",
            )
            .column_as(desired_state::Column::CreatedAt, "desired_state_created_at")
            .column_as(desired_state::Column::UpdatedAt, "desired_state_updated_at")
            .column_as(ambition::Column::Id, "ambition_id")
            .column_as(ambition::Column::Name, "ambition_name")
            .column_as(ambition::Column::Description, "ambition_description")
            .column_as(ambition::Column::CreatedAt, "ambition_created_at")
            .column_as(ambition::Column::UpdatedAt, "ambition_updated_at")
            .join_rev(LeftJoin, desired_states_actions::Relation::Action.def())
            .join_as(
                LeftJoin,
                desired_states_actions::Relation::DesiredState
                    .def()
                    .on_condition(|_left, right| {
                        Expr::col((right, desired_state::Column::Archived))
                            .eq(false)
                            .into_condition()
                    }),
                Alias::new("desired_state"),
            )
            .join_rev(
                LeftJoin,
                ambitions_desired_states::Relation::DesiredState.def(),
            )
            .join_as(
                LeftJoin,
                ambitions_desired_states::Relation::Ambition
                    .def()
                    .on_condition(|_left, right| {
                        Expr::col((right, ambition::Column::Archived))
                            .eq(false)
                            .into_condition()
                    }),
                Alias::new("ambition"),
            )
            .order_by_with_nulls(action::Column::Ordering, Asc, Last)
            .order_by_asc(action::Column::CreatedAt)
            .order_by_with_nulls(desired_state::Column::Ordering, Asc, Last)
            .order_by_with_nulls(desired_state::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(ambition::Column::Ordering, Asc, Last)
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
    use test_utils::{self, *};

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
            .ordering(Some(2))
            .insert(&db)
            .await?;
        let action_2 = factory::action(user.id)
            .ordering(Some(1))
            .insert(&db)
            .await?;
        let _archived_action = factory::action(user.id).archived(true).insert(&db).await?;

        let res = ActionQuery::find_all_by_user_id(&db, user.id, false).await?;

        let expected = [
            ActionVisible::from(action_2),
            ActionVisible::from(action_1),
            ActionVisible::from(action_0),
        ];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);
        assert_eq!(res[1], expected[1]);
        assert_eq!(res[2], expected[2]);

        Ok(())
    }

    #[actix_web::test]
    async fn find_all_by_user_id_show_archived_only() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let _action = factory::action(user.id).insert(&db).await?;
        let archived_action = factory::action(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let res = ActionQuery::find_all_by_user_id(&db, user.id, true).await?;

        let expected = [ActionVisible {
            id: archived_action.id,
            name: archived_action.name,
            description: archived_action.description,
            trackable: archived_action.trackable,
            color: archived_action.color,
            track_type: archived_action.track_type,
            created_at: archived_action.created_at,
            updated_at: archived_action.updated_at,
        }];

        assert_eq!(res.len(), expected.len());
        assert_eq!(res[0], expected[0]);

        Ok(())
    }

    #[actix_web::test]
    async fn find_all_with_linked_by_user_id() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let ambition_0 = factory::ambition(user.id).insert(&db).await?;
        let desired_state_0 = factory::desired_state(user.id).insert(&db).await?;
        let action_0 = factory::action(user.id).insert(&db).await?;
        let ambition_1 = factory::ambition(user.id)
            .ordering(Some(1))
            .insert(&db)
            .await?;
        let desired_state_1 = factory::desired_state(user.id)
            .ordering(Some(1))
            .insert(&db)
            .await?;
        let action_1 = factory::action(user.id)
            .ordering(Some(1))
            .insert(&db)
            .await?;
        factory::link_desired_state_action(&db, desired_state_0.id, action_0.id).await?;
        factory::link_desired_state_action(&db, desired_state_1.id, action_0.id).await?;
        factory::link_ambition_desired_state(&db, ambition_0.id, desired_state_0.id).await?;
        factory::link_ambition_desired_state(&db, ambition_1.id, desired_state_0.id).await?;

        let res = ActionQuery::find_all_with_linked_by_user_id(&db, user.id).await?;

        assert_eq!(res.len(), 4);

        // NOTE: Check only ids for convenience.
        let res_organized = [
            (res[0].id, res[0].desired_state_id, res[0].ambition_id),
            (res[1].id, res[1].desired_state_id, res[1].ambition_id),
            (res[2].id, res[2].desired_state_id, res[2].ambition_id),
            (res[3].id, res[3].desired_state_id, res[3].ambition_id),
        ];
        let expected = [
            (action_1.id, None, None),
            (action_0.id, Some(desired_state_1.id), None),
            (action_0.id, Some(desired_state_0.id), Some(ambition_1.id)),
            (action_0.id, Some(desired_state_0.id), Some(ambition_0.id)),
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
        let desired_state_0 = factory::desired_state(user.id).insert(&db).await?;
        let action_0 = factory::action(user.id).insert(&db).await?;
        let _archived_action = factory::action(user.id).archived(true).insert(&db).await?;
        let archived_desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        factory::link_desired_state_action(&db, desired_state_0.id, action_0.id).await?;
        factory::link_desired_state_action(&db, archived_desired_state.id, action_0.id).await?;
        factory::link_ambition_desired_state(&db, ambition_0.id, desired_state_0.id).await?;
        factory::link_ambition_desired_state(&db, archived_ambition.id, desired_state_0.id).await?;

        let res = ActionQuery::find_all_with_linked_by_user_id(&db, user.id).await?;

        assert_eq!(res.len(), 3);

        // NOTE: Check only ids for convenience.
        let res_organized = [
            (res[0].id, res[0].desired_state_id, res[0].ambition_id),
            (res[1].id, res[1].desired_state_id, res[1].ambition_id),
            (res[2].id, res[2].desired_state_id, res[2].ambition_id),
        ];
        let expected = [
            (action_0.id, Some(desired_state_0.id), Some(ambition_0.id)),
            (action_0.id, Some(desired_state_0.id), None),
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
        let archived_desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        factory::link_desired_state_action(&db, archived_desired_state.id, action.id).await?;
        factory::link_ambition_desired_state(&db, archived_ambition.id, archived_desired_state.id)
            .await?;

        let res = ActionQuery::find_all_with_linked_by_user_id(&db, user.id).await?;

        assert_eq!(res.len(), 1);

        // NOTE: Check only ids for convenience.
        let res_organized = [(res[0].id, res[0].desired_state_id, res[0].ambition_id)];
        let expected = [(action.id, None, None)];
        assert_eq!(res_organized[0], expected[0]);

        Ok(())
    }
}
