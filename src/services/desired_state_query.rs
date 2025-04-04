use ::types::{CustomDbErr, DesiredStateVisible, DesiredStateWithLinksQueryResult};
use entities::{action, ambition, ambitions_desired_states, desired_state, desired_states_actions};
use migration::{Alias, IntoCondition, NullOrdering::Last};
use sea_orm::{entity::prelude::*, JoinType::LeftJoin, Order::Asc, QueryOrder, QuerySelect};

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewDesiredState {
    pub name: String,
    pub user_id: uuid::Uuid,
}

pub struct DesiredStateQuery;

impl DesiredStateQuery {
    pub async fn find_all_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
        show_archived_only: bool,
    ) -> Result<Vec<DesiredStateVisible>, DbErr> {
        desired_state::Entity::find()
            .filter(desired_state::Column::UserId.eq(user_id))
            .filter(desired_state::Column::Archived.eq(show_archived_only))
            .order_by_with_nulls(desired_state::Column::Ordering, Asc, Last)
            .order_by_asc(desired_state::Column::CreatedAt)
            .into_partial_model::<DesiredStateVisible>()
            .all(db)
            .await
    }

    pub async fn find_all_with_linked_by_user_id(
        db: &DbConn,
        user_id: uuid::Uuid,
    ) -> Result<Vec<DesiredStateWithLinksQueryResult>, DbErr> {
        desired_state::Entity::find()
            .filter(desired_state::Column::UserId.eq(user_id))
            .filter(desired_state::Column::Archived.eq(false))
            .column_as(ambition::Column::Id, "ambition_id")
            .column_as(ambition::Column::Name, "ambition_name")
            .column_as(ambition::Column::Description, "ambition_description")
            .column_as(ambition::Column::CreatedAt, "ambition_created_at")
            .column_as(ambition::Column::UpdatedAt, "ambition_updated_at")
            .column_as(action::Column::Id, "action_id")
            .column_as(action::Column::Name, "action_name")
            .column_as(action::Column::Description, "action_description")
            .column_as(action::Column::CreatedAt, "action_created_at")
            .column_as(action::Column::UpdatedAt, "action_updated_at")
            .join_rev(
                LeftJoin,
                desired_states_actions::Relation::DesiredState.def(),
            )
            .join_as(
                LeftJoin,
                desired_states_actions::Relation::Action
                    .def()
                    .on_condition(|_left, right| {
                        Expr::col((right, action::Column::Archived))
                            .eq(false)
                            .into_condition()
                    }),
                Alias::new("action"),
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
            .order_by_with_nulls(desired_state::Column::Ordering, Asc, Last)
            .order_by_asc(desired_state::Column::CreatedAt)
            .order_by_with_nulls(ambition::Column::Ordering, Asc, Last)
            .order_by_with_nulls(ambition::Column::CreatedAt, Asc, Last)
            .order_by_with_nulls(action::Column::Ordering, Asc, Last)
            .order_by_with_nulls(action::Column::CreatedAt, Asc, Last)
            .into_model::<DesiredStateWithLinksQueryResult>()
            .all(db)
            .await
    }

    pub async fn find_by_id_and_user_id(
        db: &DbConn,
        desired_state_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<desired_state::Model, DbErr> {
        desired_state::Entity::find_by_id(desired_state_id)
            .filter(desired_state::Column::UserId.eq(user_id))
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
        let desired_state_0 = factory::desired_state(user.id)
            .name("desired_state_0".to_string())
            .insert(&db)
            .await?;
        let desired_state_1 = factory::desired_state(user.id)
            .name("desired_state_1".to_string())
            .description(Some("desired_state_1".to_string()))
            .ordering(Some(2))
            .insert(&db)
            .await?;
        let desired_state_2 = factory::desired_state(user.id)
            .ordering(Some(1))
            .insert(&db)
            .await?;
        let _archived_desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let res = DesiredStateQuery::find_all_by_user_id(&db, user.id, false).await?;

        let expected = [
            DesiredStateVisible {
                id: desired_state_2.id,
                name: desired_state_2.name,
                description: desired_state_2.description,
                created_at: desired_state_2.created_at,
                updated_at: desired_state_2.updated_at,
            },
            DesiredStateVisible {
                id: desired_state_1.id,
                name: desired_state_1.name,
                description: desired_state_1.description,
                created_at: desired_state_1.created_at,
                updated_at: desired_state_1.updated_at,
            },
            DesiredStateVisible {
                id: desired_state_0.id,
                name: desired_state_0.name,
                description: desired_state_0.description,
                created_at: desired_state_0.created_at,
                updated_at: desired_state_0.updated_at,
            },
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
        let _desired_state = factory::desired_state(user.id).insert(&db).await?;
        let archived_desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let res = DesiredStateQuery::find_all_by_user_id(&db, user.id, true).await?;

        let expected = [DesiredStateVisible {
            id: archived_desired_state.id,
            name: archived_desired_state.name,
            description: archived_desired_state.description,
            created_at: archived_desired_state.created_at,
            updated_at: archived_desired_state.updated_at,
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
        factory::link_ambition_desired_state(&db, ambition_0.id, desired_state_0.id).await?;
        factory::link_ambition_desired_state(&db, ambition_1.id, desired_state_0.id).await?;
        factory::link_desired_state_action(&db, desired_state_0.id, action_0.id).await?;
        factory::link_desired_state_action(&db, desired_state_0.id, action_1.id).await?;
        factory::link_desired_state_action(&db, desired_state_1.id, action_1.id).await?;

        let res = DesiredStateQuery::find_all_with_linked_by_user_id(&db, user.id).await?;

        assert_eq!(res.len(), 5);

        // NOTE: Check only ids for convenience.
        let res_organized = [
            (res[0].id, res[0].ambition_id, res[0].action_id),
            (res[1].id, res[1].ambition_id, res[1].action_id),
            (res[2].id, res[2].ambition_id, res[2].action_id),
            (res[3].id, res[3].ambition_id, res[3].action_id),
            (res[4].id, res[4].ambition_id, res[4].action_id),
        ];
        let expected = [
            (desired_state_1.id, None, Some(action_1.id)),
            (desired_state_0.id, Some(ambition_1.id), Some(action_1.id)),
            (desired_state_0.id, Some(ambition_1.id), Some(action_0.id)),
            (desired_state_0.id, Some(ambition_0.id), Some(action_1.id)),
            (desired_state_0.id, Some(ambition_0.id), Some(action_0.id)),
        ];
        assert_eq!(res_organized[0], expected[0]);
        assert_eq!(res_organized[1], expected[1]);
        assert_eq!(res_organized[2], expected[2]);
        assert_eq!(res_organized[3], expected[3]);
        assert_eq!(res_organized[4], expected[4]);

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
        let archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let _archived_desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let archived_action = factory::action(user.id).archived(true).insert(&db).await?;
        factory::link_ambition_desired_state(&db, ambition_0.id, desired_state_0.id).await?;
        factory::link_ambition_desired_state(&db, archived_ambition.id, desired_state_0.id).await?;
        factory::link_desired_state_action(&db, desired_state_0.id, action_0.id).await?;
        factory::link_desired_state_action(&db, desired_state_0.id, archived_action.id).await?;

        let res = DesiredStateQuery::find_all_with_linked_by_user_id(&db, user.id).await?;

        assert_eq!(res.len(), 4);

        // NOTE: Check only ids for convenience.
        let res_organized = [
            (res[0].id, res[0].ambition_id, res[0].action_id),
            (res[1].id, res[1].ambition_id, res[1].action_id),
            (res[2].id, res[2].ambition_id, res[2].action_id),
            (res[3].id, res[3].ambition_id, res[3].action_id),
        ];
        let expected = [
            (desired_state_0.id, Some(ambition_0.id), Some(action_0.id)),
            (desired_state_0.id, Some(ambition_0.id), None),
            (desired_state_0.id, None, Some(action_0.id)),
            (desired_state_0.id, None, None),
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
        let desired_state = factory::desired_state(user.id).insert(&db).await?;
        let archived_ambition = factory::ambition(user.id)
            .archived(true)
            .insert(&db)
            .await?;
        let archived_action = factory::action(user.id).archived(true).insert(&db).await?;
        factory::link_ambition_desired_state(&db, archived_ambition.id, desired_state.id).await?;
        factory::link_desired_state_action(&db, desired_state.id, archived_action.id).await?;

        let res = DesiredStateQuery::find_all_with_linked_by_user_id(&db, user.id).await?;

        assert_eq!(res.len(), 1);

        // NOTE: Check only ids for convenience.
        let res_organized = [(res[0].id, res[0].ambition_id, res[0].action_id)];
        let expected = [(desired_state.id, None, None)];
        assert_eq!(res_organized[0], expected[0]);

        Ok(())
    }
}
