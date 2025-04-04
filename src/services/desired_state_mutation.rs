use chrono::Utc;
use entities::{desired_state, desired_states_actions, tag};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue::NotSet;
use sea_orm::{IntoActiveModel, Set, TransactionError, TransactionTrait};

use super::desired_state_query::DesiredStateQuery;

#[derive(serde::Deserialize, Debug, serde::Serialize, Clone)]
pub struct NewDesiredState {
    pub name: String,
    pub description: Option<String>,
    pub user_id: uuid::Uuid,
}

pub struct DesiredStateMutation;

impl DesiredStateMutation {
    pub async fn create_with_tag(
        db: &DbConn,
        form_data: NewDesiredState,
    ) -> Result<desired_state::Model, TransactionError<DbErr>> {
        db.transaction::<_, desired_state::Model, DbErr>(|txn| {
            Box::pin(async move {
                let now = Utc::now();
                let desired_state_id = uuid::Uuid::new_v4();
                let created_desired_state = desired_state::ActiveModel {
                    id: Set(desired_state_id),
                    user_id: Set(form_data.user_id),
                    name: Set(form_data.name.to_owned()),
                    description: Set(form_data.description.to_owned()),
                    archived: Set(false),
                    ordering: NotSet,
                    created_at: Set(now.into()),
                    updated_at: Set(now.into()),
                }
                .insert(txn)
                .await?;
                tag::ActiveModel {
                    id: Set(uuid::Uuid::new_v4()),
                    user_id: Set(form_data.user_id),
                    ambition_id: NotSet,
                    desired_state_id: Set(Some(desired_state_id)),
                    action_id: NotSet,
                    created_at: Set(now.into()),
                }
                .insert(txn)
                .await?;

                Ok(created_desired_state)
            })
        })
        .await
    }

    pub async fn update(
        db: &DbConn,
        desired_state_id: uuid::Uuid,
        user_id: uuid::Uuid,
        name: String,
        description: Option<String>,
    ) -> Result<desired_state::Model, DbErr> {
        let mut desired_state: desired_state::ActiveModel =
            DesiredStateQuery::find_by_id_and_user_id(db, desired_state_id, user_id)
                .await?
                .into();
        desired_state.name = Set(name);
        desired_state.description = Set(description);
        desired_state.updated_at = Set(Utc::now().into());
        desired_state.update(db).await
    }

    pub async fn delete(
        db: &DbConn,
        desired_state_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        DesiredStateQuery::find_by_id_and_user_id(db, desired_state_id, user_id)
            .await?
            .delete(db)
            .await?;
        Ok(())
    }

    pub async fn archive(
        db: &DbConn,
        desired_state_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<desired_state::Model, DbErr> {
        let mut desired_state: desired_state::ActiveModel =
            DesiredStateQuery::find_by_id_and_user_id(db, desired_state_id, user_id)
                .await?
                .into();
        desired_state.archived = Set(true);
        desired_state.updated_at = Set(Utc::now().into());
        desired_state.update(db).await
    }

    pub async fn unarchive(
        db: &DbConn,
        desired_state_id: uuid::Uuid,
        user_id: uuid::Uuid,
    ) -> Result<desired_state::Model, DbErr> {
        let mut desired_state: desired_state::ActiveModel =
            DesiredStateQuery::find_by_id_and_user_id(db, desired_state_id, user_id)
                .await?
                .into();
        desired_state.archived = Set(false);
        desired_state.updated_at = Set(Utc::now().into());
        desired_state.update(db).await
    }

    pub async fn connect_action(
        db: &DbConn,
        desired_state_id: uuid::Uuid,
        action_id: uuid::Uuid,
    ) -> Result<desired_states_actions::Model, DbErr> {
        desired_states_actions::ActiveModel {
            desired_state_id: Set(desired_state_id),
            action_id: Set(action_id),
        }
        .insert(db)
        .await
    }

    pub async fn disconnect_action(
        db: &DbConn,
        desired_state_id: uuid::Uuid,
        action_id: uuid::Uuid,
    ) -> Result<(), DbErr> {
        match desired_states_actions::Entity::find()
            .filter(desired_states_actions::Column::DesiredStateId.eq(desired_state_id))
            .filter(desired_states_actions::Column::ActionId.eq(action_id))
            .one(db)
            .await
        {
            Ok(connection) => match connection {
                Some(connection) => {
                    connection.delete(db).await?;
                    Ok(())
                }
                None => Ok(()),
            },
            Err(e) => Err(e),
        }
    }

    // FIXME: Reduce query.
    pub async fn bulk_update_ordering(
        db: &DbConn,
        user_id: uuid::Uuid,
        ordering: Vec<uuid::Uuid>,
    ) -> Result<(), DbErr> {
        let desired_states = desired_state::Entity::find()
            .filter(desired_state::Column::UserId.eq(user_id))
            .filter(desired_state::Column::Id.is_in(ordering.clone()))
            .all(db)
            .await?;
        for desired_state in desired_states {
            let order = &ordering.iter().position(|id| id == &desired_state.id);
            if let Some(order) = order {
                let mut desired_state = desired_state.into_active_model();
                desired_state.ordering = Set(Some((order + 1) as i32));
                desired_state.update(db).await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use ::types::CustomDbErr;
    use test_utils::{self, *};

    use super::*;

    #[actix_web::test]
    async fn create_with_tag() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let name = "create_with_tag".to_string();
        let description = "Create with tag.".to_string();

        let form_data = NewDesiredState {
            name: name.clone(),
            description: Some(description.clone()),
            user_id: user.id,
        };
        let returned_desired_state = DesiredStateMutation::create_with_tag(&db, form_data)
            .await
            .unwrap();
        assert_eq!(returned_desired_state.name, name.clone());
        assert_eq!(
            returned_desired_state.description,
            Some(description.clone())
        );
        assert_eq!(returned_desired_state.archived, false);
        assert_eq!(returned_desired_state.user_id, user.id);

        let created_desired_state = desired_state::Entity::find_by_id(returned_desired_state.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(created_desired_state.name, returned_desired_state.name);
        assert_eq!(
            created_desired_state.description,
            returned_desired_state.description
        );
        assert_eq!(created_desired_state.archived, false);
        assert_eq!(
            created_desired_state.user_id,
            returned_desired_state.user_id
        );
        assert_eq!(
            created_desired_state.created_at,
            returned_desired_state.created_at
        );
        assert_eq!(
            created_desired_state.updated_at,
            returned_desired_state.updated_at
        );

        let created_tag = tag::Entity::find()
            .filter(tag::Column::AmbitionId.is_null())
            .filter(tag::Column::DesiredStateId.eq(returned_desired_state.id))
            .filter(tag::Column::ActionId.is_null())
            .filter(tag::Column::UserId.eq(user.id))
            .one(&db)
            .await?;
        assert!(created_tag.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn update() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;

        let new_name = "desired_state_after_update".to_string();
        let new_description = "DesiredState after update.".to_string();

        let returned_desired_state = DesiredStateMutation::update(
            &db,
            desired_state.id,
            user.id,
            new_name.clone(),
            Some(new_description.clone()),
        )
        .await?;
        assert_eq!(returned_desired_state.id, desired_state.id);
        assert_eq!(returned_desired_state.name, new_name.clone());
        assert_eq!(
            returned_desired_state.description,
            Some(new_description.clone())
        );
        assert_eq!(returned_desired_state.archived, desired_state.archived);
        assert_eq!(returned_desired_state.user_id, user.id);
        assert_eq!(returned_desired_state.created_at, desired_state.created_at);
        assert!(returned_desired_state.updated_at > desired_state.updated_at);

        let updated_desired_state = desired_state::Entity::find_by_id(desired_state.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(updated_desired_state.name, new_name.clone());
        assert_eq!(
            updated_desired_state.description,
            Some(new_description.clone())
        );
        assert_eq!(updated_desired_state.archived, desired_state.archived);
        assert_eq!(updated_desired_state.user_id, user.id);
        assert_eq!(updated_desired_state.created_at, desired_state.created_at);
        assert_eq!(
            updated_desired_state.updated_at,
            returned_desired_state.updated_at
        );

        Ok(())
    }

    #[actix_web::test]
    async fn update_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;

        let new_name = "desired_state_after_update_unauthorized".to_string();

        let error = DesiredStateMutation::update(
            &db,
            desired_state.id,
            uuid::Uuid::new_v4(),
            new_name.clone(),
            None,
        )
        .await
        .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn delete() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let (desired_state, tag) = factory::desired_state(user.id).insert_with_tag(&db).await?;

        DesiredStateMutation::delete(&db, desired_state.id, user.id).await?;

        let desired_state_in_db = desired_state::Entity::find_by_id(desired_state.id)
            .one(&db)
            .await?;
        assert!(desired_state_in_db.is_none());

        let tag_in_db = tag::Entity::find_by_id(tag.id).one(&db).await?;
        assert!(tag_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn delete_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;

        let error = DesiredStateMutation::delete(&db, desired_state.id, uuid::Uuid::new_v4())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn archive() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;

        let returned_desired_state =
            DesiredStateMutation::archive(&db, desired_state.id, user.id).await?;
        assert_eq!(returned_desired_state.id, desired_state.id);
        assert_eq!(returned_desired_state.name, desired_state.name.clone());
        assert_eq!(
            returned_desired_state.description,
            desired_state.description.clone()
        );
        assert_eq!(returned_desired_state.archived, true);
        assert_eq!(returned_desired_state.user_id, user.id);
        assert_eq!(returned_desired_state.created_at, desired_state.created_at);
        assert!(returned_desired_state.updated_at > desired_state.updated_at);

        let archived_desired_state = desired_state::Entity::find_by_id(desired_state.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(archived_desired_state.name, desired_state.name.clone());
        assert_eq!(
            archived_desired_state.description,
            desired_state.description.clone()
        );
        assert_eq!(archived_desired_state.archived, true);
        assert_eq!(archived_desired_state.user_id, user.id);
        assert_eq!(archived_desired_state.created_at, desired_state.created_at);
        assert_eq!(
            archived_desired_state.updated_at,
            returned_desired_state.updated_at
        );

        Ok(())
    }

    #[actix_web::test]
    async fn archive_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;

        let error = DesiredStateMutation::archive(&db, desired_state.id, uuid::Uuid::new_v4())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn unarchive() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let returned_desired_state =
            DesiredStateMutation::unarchive(&db, desired_state.id, user.id).await?;
        assert_eq!(returned_desired_state.id, desired_state.id);
        assert_eq!(returned_desired_state.name, desired_state.name.clone());
        assert_eq!(
            returned_desired_state.description,
            desired_state.description.clone()
        );
        assert_eq!(returned_desired_state.archived, false);
        assert_eq!(returned_desired_state.user_id, user.id);
        assert_eq!(returned_desired_state.created_at, desired_state.created_at);
        assert!(returned_desired_state.updated_at > desired_state.updated_at);

        let restored_desired_state = desired_state::Entity::find_by_id(desired_state.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(restored_desired_state.name, desired_state.name.clone());
        assert_eq!(
            restored_desired_state.description,
            desired_state.description.clone()
        );
        assert_eq!(restored_desired_state.archived, false);
        assert_eq!(restored_desired_state.user_id, user.id);
        assert_eq!(restored_desired_state.created_at, desired_state.created_at);
        assert_eq!(
            restored_desired_state.updated_at,
            returned_desired_state.updated_at
        );

        Ok(())
    }

    #[actix_web::test]
    async fn unarchive_unauthorized() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id)
            .archived(true)
            .insert(&db)
            .await?;

        let error = DesiredStateMutation::archive(&db, desired_state.id, uuid::Uuid::new_v4())
            .await
            .unwrap_err();
        assert_eq!(error, DbErr::Custom(CustomDbErr::NotFound.to_string()));

        Ok(())
    }

    #[actix_web::test]
    async fn connect_action() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;

        DesiredStateMutation::connect_action(&db, desired_state.id, action.id).await?;

        let created_connection = desired_states_actions::Entity::find()
            .filter(desired_states_actions::Column::DesiredStateId.eq(desired_state.id))
            .filter(desired_states_actions::Column::ActionId.eq(action.id))
            .one(&db)
            .await?;
        assert!(created_connection.is_some());

        Ok(())
    }

    #[actix_web::test]
    async fn disconnect_action() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let action = factory::action(user.id).insert(&db).await?;
        let desired_state = factory::desired_state(user.id).insert(&db).await?;
        factory::link_desired_state_action(&db, desired_state.id, action.id).await?;

        DesiredStateMutation::disconnect_action(&db, desired_state.id, action.id).await?;

        let connection_in_db = desired_states_actions::Entity::find()
            .filter(desired_states_actions::Column::DesiredStateId.eq(desired_state.id))
            .filter(desired_states_actions::Column::ActionId.eq(action.id))
            .one(&db)
            .await?;
        assert!(connection_in_db.is_none());

        Ok(())
    }

    #[actix_web::test]
    async fn bulk_update_ordering() -> Result<(), DbErr> {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let desired_state_0 = factory::desired_state(user.id).insert(&db).await?;
        let desired_state_1 = factory::desired_state(user.id).insert(&db).await?;
        let desired_state_2 = factory::desired_state(user.id).insert(&db).await?;

        let ordering = vec![desired_state_0.id, desired_state_1.id];

        DesiredStateMutation::bulk_update_ordering(&db, user.id, ordering).await?;

        let desired_state_in_db_0 = desired_state::Entity::find_by_id(desired_state_0.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(desired_state_in_db_0.ordering, Some(1));

        let desired_state_in_db_1 = desired_state::Entity::find_by_id(desired_state_1.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(desired_state_in_db_1.ordering, Some(2));

        let desired_state_in_db_2 = desired_state::Entity::find_by_id(desired_state_2.id)
            .one(&db)
            .await?
            .unwrap();
        assert_eq!(desired_state_in_db_2.ordering, None);

        Ok(())
    }

    #[actix_web::test]
    async fn bulk_update_ordering_no_modification_on_different_users_records() -> Result<(), DbErr>
    {
        let db = test_utils::init_db().await?;
        let user = factory::user().insert(&db).await?;
        let another_user = factory::user().insert(&db).await?;
        let another_users_desired_state =
            factory::desired_state(another_user.id).insert(&db).await?;

        let ordering = vec![another_users_desired_state.id];

        DesiredStateMutation::bulk_update_ordering(&db, user.id, ordering).await?;

        let another_users_desired_state_in_db =
            desired_state::Entity::find_by_id(another_users_desired_state.id)
                .one(&db)
                .await?
                .unwrap();
        assert_eq!(another_users_desired_state_in_db.ordering, None);

        Ok(())
    }
}
